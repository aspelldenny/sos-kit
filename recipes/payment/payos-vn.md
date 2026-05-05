# Recipe: PayOS VN (payment integration)

> **Category:** payment
> **Stability:** stable (battle-tested ở tarot)
> **Last verified:** 2026-04-25

## Mục đích

Tích hợp PayOS làm payment gateway cho user VN — credit-based top-up. PayOS hỗ trợ QR code chuyển khoản nội địa, fee thấp hơn Stripe cho thị trường VN.

## Inputs (yêu cầu trước)

- [ ] Recipe `infra/docker-compose-postgres` đã apply (cần DB lưu transactions)
- [ ] Recipe `auth/<any>` đã apply (cần user identity)
- [ ] Tài khoản PayOS đã verify business — có `clientId`, `apiKey`, `checksumKey`
- [ ] Domain công khai (PayOS yêu cầu HTTPS callback URL)

## Outputs (sau khi apply)

- API route `/api/payment/payos/create-order`
- API route `/api/payment/payos/webhook`
- DB table `payment_transactions` + `credit_topups`
- Atomic credit deduct/add helper `lib/credits/transaction.ts`
- Pre-charge pattern (đăng ký pre-charge trước khi consume) — **bắt buộc** để tránh race

## Steps

### 1. Schema

```prisma
// prisma/schema.prisma
model PaymentTransaction {
  id           String   @id @default(cuid())
  userId       String
  orderCode    BigInt   @unique
  amount       Int
  status       PaymentStatus @default(PENDING)
  payosRef     String?
  webhookData  Json?
  createdAt    DateTime @default(now())
  completedAt  DateTime?

  user         User @relation(fields: [userId], references: [id])

  @@index([userId, status])
  @@index([orderCode])
}

enum PaymentStatus {
  PENDING
  COMPLETED
  FAILED
  REFUNDED
}

model CreditTopup {
  id              String   @id @default(cuid())
  userId          String
  transactionId   String   @unique
  creditsAdded    Int
  balanceBefore   Int
  balanceAfter    Int
  createdAt       DateTime @default(now())

  transaction     PaymentTransaction @relation(fields: [transactionId], references: [id])
  user            User @relation(fields: [userId], references: [id])

  @@index([userId, createdAt])
}
```

### 2. Atomic deduct helper

```typescript
// lib/credits/transaction.ts
import { prisma } from "@/lib/prisma";

export async function deductCreditsAtomic(
  userId: string,
  amount: number,
  reason: string,
  refId?: string
): Promise<{ ok: true; balanceAfter: number } | { ok: false; error: string }> {
  return await prisma.$transaction(async (tx) => {
    const user = await tx.user.findUnique({
      where: { id: userId },
      select: { credits: true },
    });
    if (!user) return { ok: false as const, error: "USER_NOT_FOUND" };
    if (user.credits < amount) return { ok: false as const, error: "INSUFFICIENT_CREDITS" };

    const updated = await tx.user.update({
      where: { id: userId, credits: { gte: amount } }, // race-safe via WHERE clause
      data: { credits: { decrement: amount } },
      select: { credits: true },
    });

    await tx.creditLedger.create({
      data: { userId, delta: -amount, reason, refId, balanceAfter: updated.credits },
    });

    return { ok: true as const, balanceAfter: updated.credits };
  }, { isolationLevel: "Serializable" });
}
```

### 3. Pre-charge pattern (bắt buộc)

> **Bài học từ tarot P004:** Nếu deduct credits **sau** khi user consume, race condition + crash = revenue leak. Phải **pre-charge** lúc bắt đầu, refund nếu fail.

```typescript
// Khi user start session "deep reading" tốn 60 credits:
const charge = await deductCreditsAtomic(userId, 60, "deep_reading_pre_charge", sessionId);
if (!charge.ok) throw new Error(charge.error);

try {
  // ... thực hiện work (có thể fail nhiều bước)
  await runDeepReading(sessionId);
} catch (err) {
  // refund
  await refundCreditsAtomic(userId, 60, "deep_reading_failed", sessionId);
  throw err;
}
```

### 4. PayOS create order

```typescript
// app/api/payment/payos/create-order/route.ts
import { NextResponse } from "next/server";
import crypto from "crypto";

const PAYOS_API = "https://api-merchant.payos.vn/v2/payment-requests";

export async function POST(req: Request) {
  const { amount, packageId } = await req.json();
  const session = await getSession(req); // your auth helper
  if (!session) return NextResponse.json({ error: "unauth" }, { status: 401 });

  const orderCode = Date.now(); // unique BigInt
  const description = `Topup ${packageId}`;
  const returnUrl = `${process.env.APP_URL}/topup/success`;
  const cancelUrl = `${process.env.APP_URL}/topup/cancel`;

  const data = `amount=${amount}&cancelUrl=${cancelUrl}&description=${description}&orderCode=${orderCode}&returnUrl=${returnUrl}`;
  const signature = crypto
    .createHmac("sha256", process.env.PAYOS_CHECKSUM_KEY!)
    .update(data)
    .digest("hex");

  const res = await fetch(PAYOS_API, {
    method: "POST",
    headers: {
      "x-client-id": process.env.PAYOS_CLIENT_ID!,
      "x-api-key": process.env.PAYOS_API_KEY!,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      orderCode, amount, description, returnUrl, cancelUrl, signature,
    }),
  });

  const json = await res.json();
  if (json.code !== "00") return NextResponse.json({ error: json.desc }, { status: 400 });

  await prisma.paymentTransaction.create({
    data: { userId: session.userId, orderCode: BigInt(orderCode), amount, status: "PENDING" },
  });

  return NextResponse.json({ checkoutUrl: json.data.checkoutUrl });
}
```

### 5. PayOS webhook (idempotent)

```typescript
// app/api/payment/payos/webhook/route.ts
export async function POST(req: Request) {
  const body = await req.json();

  // Verify signature
  const expected = crypto
    .createHmac("sha256", process.env.PAYOS_CHECKSUM_KEY!)
    .update(JSON.stringify(body.data))
    .digest("hex");
  if (expected !== body.signature) {
    return NextResponse.json({ error: "invalid signature" }, { status: 401 });
  }

  const orderCode = BigInt(body.data.orderCode);

  // Idempotent — check if already processed
  const existing = await prisma.paymentTransaction.findUnique({ where: { orderCode } });
  if (!existing) return NextResponse.json({ error: "unknown order" }, { status: 404 });
  if (existing.status === "COMPLETED") return NextResponse.json({ ok: true }); // already done

  // Atomic: mark completed + add credits
  await prisma.$transaction(async (tx) => {
    const tx1 = await tx.paymentTransaction.update({
      where: { orderCode, status: "PENDING" }, // race-safe
      data: { status: "COMPLETED", payosRef: body.data.reference, webhookData: body, completedAt: new Date() },
    });

    const credits = computeCreditsForAmount(tx1.amount); // your pricing logic
    const userBefore = await tx.user.findUnique({ where: { id: tx1.userId }, select: { credits: true } });

    await tx.user.update({
      where: { id: tx1.userId },
      data: { credits: { increment: credits } },
    });

    await tx.creditTopup.create({
      data: {
        userId: tx1.userId,
        transactionId: tx1.id,
        creditsAdded: credits,
        balanceBefore: userBefore!.credits,
        balanceAfter: userBefore!.credits + credits,
      },
    });
  });

  return NextResponse.json({ ok: true });
}
```

## Verification anchors

```bash
# 1. Schema migrated
grep "PaymentTransaction" prisma/schema.prisma

# 2. Webhook endpoint registered
curl -X POST $APP_URL/api/payment/payos/webhook -H "Content-Type: application/json" -d '{}'
# Expected: 401 invalid signature (chứng tỏ route exist + verify chạy)

# 3. Atomic helper compile
grep "deductCreditsAtomic" lib/credits/transaction.ts

# 4. Pre-charge pattern dùng đúng nơi (không ai consume trước khi charge)
grep -rn "deductCreditsAtomic" app/api/ src/

# 5. ENV keys đầy đủ
grep "PAYOS_CLIENT_ID\|PAYOS_API_KEY\|PAYOS_CHECKSUM_KEY" .env.example
```

## Discovery hooks (chỗ dễ sai)

| Pattern | Bài học |
|---------|---------|
| Webhook signature mismatch | PayOS gửi `data` JSON đã sort key alphabet — phải verify TRÊN data raw, không trên object đã parse-rồi-stringify lại |
| `orderCode` overflow | PayOS chấp nhận `BigInt` nhưng JS `Date.now()` an toàn 53-bit. Đừng dùng `Math.random()` — collision risk |
| Webhook chạy 2 lần | PayOS retry nếu webhook timeout >5s. Idempotent check qua `where: { orderCode, status: "PENDING" }` mới race-safe |
| Pre-charge refund quên | Nếu try/catch không bao trùm full work → user bị trừ mà không nhận service. Phải refund trong catch + có cron scan "PENDING" >1h auto-refund |
| Test với sandbox | PayOS có sandbox env riêng — đừng test với prod key. Sandbox: `https://api-merchant.payos.vn/v2/...` (cùng domain, account khác) |

## Env vars

```bash
# .env.example
PAYOS_CLIENT_ID=
PAYOS_API_KEY=
PAYOS_CHECKSUM_KEY=
APP_URL=https://your-domain.com
```

## Migration từ Stripe (nếu có)

Nếu sếp đã có Stripe integration: PayOS không thay thế 1-1, vì PayOS không có subscription model. Pattern khuyên: PayOS cho top-up credit (one-time), Stripe (nếu cần) cho subscription tier. Tarot dùng pure credit → chỉ PayOS.

## Source

- DNA: `~/<your-app>/app/api/payment/payos/*` (extracted, business logic stripped)
- Bài học: `~/<your-app>/docs/DISCOVERIES.md` P004 entry (revenue leak post-mortem)
- PayOS docs: https://payos.vn/docs/api/
