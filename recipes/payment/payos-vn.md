# Recipe: PayOS VN (payment integration — official SDK)

> **Category:** payment
> **Stability:** stable (battle-tested ở tarot production)
> **Last verified:** 2026-07-23 (verified against tarot @cd16a86)

## Changelog
- **2026-07-23 (updated from tarot production, verified @cd16a86):** raw-HMAC crypto → **official `@payos/node@2.0.5` SDK**; thêm idempotency-qua-`updateMany`, webhook-fail-to-500-for-retry, và mục **subscription/VIP-qua-topup** (sửa kết luận sai cũ "PayOS không có subscription").
- 2026-04-25: bản đầu (raw HMAC — nay deprecated, xem Migration).

## Mục đích

PayOS payment gateway cho user VN (QR chuyển khoản nội địa, fee thấp hơn Stripe). Credit top-up **và** VIP/subscription-qua-topup. Dùng **SDK official** (`@payos/node`) thay raw HMAC — SDK tự lo signature create + webhook verify, ít bề mặt sai crypto.

## Inputs (yêu cầu trước khi apply)

- [ ] Recipe `infra/docker-compose-postgres` (DB lưu transactions)
- [ ] Recipe `auth/nextauth-google-credentials` (user identity)
- [ ] Tài khoản PayOS verified — `clientId`, `apiKey`, `checksumKey`
- [ ] Domain HTTPS công khai (webhook URL)
- [ ] Dep `@payos/node@2.0.5` (verified against tarot `package.json` @cd16a86)

## Outputs (sau khi apply)

- `src/lib/payment/payos.ts` — SDK client singleton + `createPayment` + `verifyWebhook` wrapper
- API routes `/api/payment/{create,webhook,status}`
- DB `PaymentTransaction` + `CreditTopup` (schema giữ nguyên bản cũ) + `Subscription` (VIP grant)
- VIP grant helper `handleVipPaymentInTx` (+30d, extend-if-active)

## Steps

### 1. Schema (giữ nguyên — vẫn đúng)

```prisma
model PaymentTransaction {
  id          String   @id @default(cuid())
  userId      String
  orderCode   BigInt   @unique
  amount      Int
  status      PaymentStatus @default(PENDING)
  payosRef    String?
  webhookData Json?
  createdAt   DateTime @default(now())
  completedAt DateTime?
  user        User @relation(fields: [userId], references: [id])
  @@index([userId, status])
  @@index([orderCode])
}
enum PaymentStatus { PENDING COMPLETED FAILED REFUNDED }

model CreditTopup {
  id            String   @id @default(cuid())
  userId        String
  transactionId String   @unique
  creditsAdded  Int
  balanceBefore Int
  balanceAfter  Int
  createdAt     DateTime @default(now())
  transaction   PaymentTransaction @relation(fields: [transactionId], references: [id])
  user          User @relation(fields: [userId], references: [id])
  @@index([userId, createdAt])
}

// VIP/subscription-qua-topup (Step 5) — mirrors tarot's real Subscription model
// (tarot/src/app/api/payment/webhook/route.ts:129-178), NOT a `vipUntil` column on User.
model Subscription {
  userId             String   @id
  tier               String   // "vip"
  status             String   // "active"
  lastPaymentOrderId String?
  currentPeriodStart DateTime
  currentPeriodEnd   DateTime
  user               User @relation(fields: [userId], references: [id])
}
```

### 2. SDK client (thay raw HMAC — điểm mới #1)

```typescript
// src/lib/payment/payos.ts
import { PayOS } from "@payos/node";
import type { WebhookData } from "@payos/node";

// Verified constructor shape against tarot/src/lib/payment/payos.ts:1-22 — PayOS takes a SINGLE
// options object, NOT 3 positional args ({ clientId, apiKey, checksumKey }).
let _client: PayOS | null = null;
function getPayOSClient(): PayOS {
  if (!_client) {
    const clientId = process.env.PAYOS_CLIENT_ID;
    const apiKey = process.env.PAYOS_API_KEY;
    const checksumKey = process.env.PAYOS_CHECKSUM_KEY;
    if (!clientId || !apiKey || !checksumKey) {
      throw new Error("PayOS credentials not configured. Set PAYOS_CLIENT_ID, PAYOS_API_KEY, PAYOS_CHECKSUM_KEY.");
    }
    _client = new PayOS({ clientId, apiKey, checksumKey });
  }
  return _client;
}

export interface CreatePaymentParams {
  orderCode: number; amount: number; description: string; returnUrl: string; cancelUrl: string;
}
export interface PaymentLinkResult { checkoutUrl: string; qrCode: string; }

export async function createPayment(params: CreatePaymentParams): Promise<PaymentLinkResult> {
  const payos = getPayOSClient();
  // Method confirmed: payos.paymentRequests.create(...) — NOT payos.createPaymentLink().
  const response = await payos.paymentRequests.create(params);
  // Return shape confirmed: response.checkoutUrl / response.qrCode are TOP-LEVEL fields
  // (not nested under response.data).
  return { checkoutUrl: response.checkoutUrl, qrCode: response.qrCode };
}

export async function verifyWebhook(webhookBody: {
  code: string; desc: string; success: boolean; data: WebhookData; signature: string;
}): Promise<WebhookData> {
  const payos = getPayOSClient();
  // payos.webhooks.verify() — wrapped in an async fn in tarot; await it regardless of
  // whether the SDK method itself is sync (awaiting a non-Promise is a safe no-op).
  return payos.webhooks.verify(webhookBody);
}
```

### 3. Create payment (SDK `paymentRequests.create` — điểm mới #1)

```typescript
// src/app/api/payment/create/route.ts
import { createPayment } from "@/lib/payment/payos";

export async function POST(req: Request) {
  const session = await getSession(req);
  if (!session) return Response.json({ error: "unauth" }, { status: 401 });
  const { amount, packageId } = await req.json();

  const orderCode = Date.now(); // unique, 53-bit safe
  const link = await createPayment({
    orderCode, amount,
    description: `Topup ${packageId}`.slice(0, 25), // PayOS giới hạn độ dài description
    returnUrl: `${process.env.APP_URL}/topup/success`,
    cancelUrl: `${process.env.APP_URL}/topup/cancel`,
  });

  await prisma.paymentTransaction.create({
    data: { userId: session.userId, orderCode: BigInt(orderCode), amount, status: "PENDING" },
  });
  return Response.json({ checkoutUrl: link.checkoutUrl, qrCode: link.qrCode });
}
```

### 4. Webhook — SDK verify + idempotency + retry-on-fail (điểm mới #1, #2, #3)

```typescript
// src/app/api/payment/webhook/route.ts
import { verifyWebhook } from "@/lib/payment/payos";

export async function POST(req: Request) {
  const body = await req.json();

  // #1 — SDK verify (thay raw HMAC). Throw nếu chữ ký sai.
  // Confirmed tarot returns 400 (not 401) on bad signature — a bad signature will NEVER succeed
  // on retry, so it's a client-error class, distinct from the 500 used for retriable settlement
  // failures below (#3).
  let data;
  try { data = await verifyWebhook(body); }
  catch { return Response.json({ error: "invalid signature" }, { status: 400 }); }

  const orderCode = BigInt(data.orderCode);

  try {
    await prisma.$transaction(async (tx) => {
      // #2 — idempotency qua updateMany + count: chỉ update khi CHƯA completed.
      // NOTE: tarot payosRef stores String(data.orderCode) as its own settlement reference — the
      // SDK's WebhookData type was not confirmed to expose a separate `.reference` field (tarot
      // never reads one); don't invent one.
      const res = await tx.paymentTransaction.updateMany({
        where: { orderCode, status: { not: "COMPLETED" } },
        data: { status: "COMPLETED", payosRef: String(data.orderCode), webhookData: body, completedAt: new Date() },
      });
      if (res.count === 0) return; // đã xử lý rồi (retry) → no-op, KHÔNG cộng credit lần 2

      const trx = await tx.paymentTransaction.findUnique({ where: { orderCode } });
      const credits = computeCreditsForAmount(trx!.amount);
      const before = await tx.user.findUnique({ where: { id: trx!.userId }, select: { credits: true } });
      await tx.user.update({ where: { id: trx!.userId }, data: { credits: { increment: credits } } });
      await tx.creditTopup.create({ data: {
        userId: trx!.userId, transactionId: trx!.id, creditsAdded: credits,
        balanceBefore: before!.credits, balanceAfter: before!.credits + credits,
      }});

      // #4 — nếu package là VIP → gia hạn subscription trong CÙNG tx (xem Step 5).
      await handleVipPaymentInTx(tx, trx!);
    });
  } catch (err) {
    // #3 — settlement tx fail → trả 500 để PayOS RETRY (đừng nuốt lỗi trả 200).
    console.error("payos settlement failed", err);
    return Response.json({ error: "settlement failed" }, { status: 500 });
  }

  return Response.json({ ok: true });
}
```

> **Verified field names:** `data.orderCode` confirmed (tarot `webhookData.orderCode`, webhook/route.ts:47). `data.reference` — NOT used anywhere in tarot's real code (grepped `src/` for `.reference`, zero hits); dropped from this recipe rather than kept as an unverified guess.

### 5. VIP / subscription-qua-topup (điểm mới #4 — sửa kết luận sai cũ)

> **Sửa recipe cũ:** bản 2026-04-25 kết luận "PayOS không có subscription model". SAI — tarot production làm **subscription-qua-topup**: 1 payment VIP → gia hạn +30 ngày trong cùng settlement tx, qua model `Subscription` riêng (không phải `vipUntil` column trên `User` — draft ban đầu giả định sai field này; verified against `tarot/src/app/api/payment/webhook/route.ts:129-178`). PayOS chỉ là one-time payment, nhưng *subscription semantics* dựng ở app-layer.

```typescript
// trong lib/payment — gọi từ webhook tx (Step 4)
export async function handleVipPaymentInTx(tx: PrismaTx, trx: PaymentTransaction) {
  const pkg = resolvePackage(trx.amount); // map amount → package (tarot keys off packageId instead — adapt to your schema)
  if (pkg?.type !== "VIP") return;

  const now = new Date();
  const periodEnd = new Date(now.getTime() + 30 * 24 * 60 * 60 * 1000); // +30 days
  const existingSub = await tx.subscription.findUnique({ where: { userId: trx.userId } });

  if (!existingSub) {
    await tx.subscription.create({ data: {
      userId: trx.userId, tier: "vip", status: "active",
      lastPaymentOrderId: trx.id, currentPeriodStart: now, currentPeriodEnd: periodEnd,
    }});
  } else if (existingSub.status === "active" && existingSub.currentPeriodEnd > now) {
    // Extend from CURRENT period end, not from now — user doesn't lose unused days.
    const newEnd = new Date(existingSub.currentPeriodEnd.getTime() + 30 * 24 * 60 * 60 * 1000);
    await tx.subscription.update({ where: { userId: trx.userId }, data: {
      tier: "vip", status: "active", lastPaymentOrderId: trx.id, currentPeriodEnd: newEnd,
    }});
  } else {
    // Expired/cancelled → renew from now.
    await tx.subscription.update({ where: { userId: trx.userId }, data: {
      tier: "vip", status: "active", lastPaymentOrderId: trx.id,
      currentPeriodStart: now, currentPeriodEnd: periodEnd,
    }});
  }
}
```

> **App-specific extension (not part of core recipe, seen in tarot):** on VIP grant, tarot ALSO bundles a one-time bonus credit top-up (`addCreditsInTx(tx, userId, 25, ...)`) and upgrades `user.authLevel = "pro"` in the same tx — the same "bundle side-effects into the settlement tx" pattern as `onFirstSignIn` in the auth recipe. Extend `handleVipPaymentInTx` per-app; don't assume every project wants bonus credits on VIP purchase.

### 6. Atomic credit deduct + pre-charge (giữ nguyên bản cũ — vẫn đúng)

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

## Verification anchors

```bash
grep -n "@payos/node" package.json src/lib/payment/payos.ts        # 1. dùng SDK, không raw HMAC
grep -rn "createHmac" src/app/api/payment/ && echo "STILL RAW HMAC — FAIL" || echo "ok: SDK"  # 2. đã bỏ raw HMAC
grep -n "new PayOS({" src/lib/payment/payos.ts                     # 3. object-arg constructor (not positional)
grep -n "webhooks.verify" src/lib/payment/payos.ts                 # 4. SDK verify
grep -n "updateMany" src/app/api/payment/webhook/route.ts          # 5. idempotency qua updateMany
grep -n "status: 500" src/app/api/payment/webhook/route.ts         # 6. retry-on-fail
grep -n "status: 400" src/app/api/payment/webhook/route.ts         # 7. bad-signature is non-retriable 400, not 401
grep -rn "handleVipPaymentInTx\|Subscription" src/lib/payment/ prisma/schema.prisma  # 8. VIP subscription
grep -E "PAYOS_CLIENT_ID|PAYOS_API_KEY|PAYOS_CHECKSUM_KEY" .env.example  # 9. ENV
curl -X POST $APP_URL/api/payment/webhook -H "Content-Type: application/json" -d '{}'  # 10. → 400 (route + verify chạy)
```

## Discovery hooks (chỗ dễ sai)

| Pattern | Bài học |
|---------|---------|
| Còn raw HMAC (recipe cũ) | Tự `createHmac` dễ sai thứ tự sort key / encode. SDK `payos.webhooks.verify` chuẩn hoá — dùng SDK. |
| `new PayOS(id, key, checksum)` positional | SAI — SDK v2 constructor nhận 1 options object `{ clientId, apiKey, checksumKey }`. Positional args compile-fail hoặc silently wrong at runtime tuỳ TS strictness. |
| Webhook cộng credit 2 lần | PayOS retry nếu timeout. Idempotency PHẢI qua `updateMany({where:{status:{not:COMPLETED}}})` + check `count===0` → no-op, KHÔNG `findUnique`-rồi-update (race). |
| Nuốt lỗi settlement trả 200 | Nếu tx fail mà trả 200, PayOS coi như xong → mất tiền/credit. Trả **500** để PayOS RETRY (điểm #3). |
| Bad-signature trả 401 | Tarot dùng **400** cho signature sai — lỗi này KHÔNG retry-able (never succeeds), khác class với 500 (settlement, retriable). Đừng conflate 2 loại lỗi. |
| "PayOS không có subscription" (kết luận sai cũ) | PayOS one-time, nhưng subscription = app-layer: 1 payment VIP → `Subscription` row (tier/status/currentPeriodStart/currentPeriodEnd), cộng-dồn từ `currentPeriodEnd` cũ nếu còn active, trong CÙNG settlement tx (`handleVipPaymentInTx`). |
| `orderCode` overflow | `Date.now()` 53-bit safe; đừng `Math.random()` (collision). |
| Description quá dài | PayOS giới hạn độ dài `description` — `.slice(0, 25)` (verify limit thật với SDK). |
| Test prod key | PayOS có sandbox riêng — đừng test bằng prod checksum key. |
| Bịa field `.reference` từ WebhookData | Không confirmed trong tarot's usage — code thật chỉ dùng `.orderCode`. Đừng carry-over field name chưa verify sang project mới. |

## Env vars

```bash
PAYOS_CLIENT_ID=
PAYOS_API_KEY=
PAYOS_CHECKSUM_KEY=
APP_URL=https://your-domain.com
```

## Migration / interop notes

- **Từ bản raw-HMAC (2026-04-25):** thay `crypto.createHmac` (create + webhook) bằng `payos.paymentRequests.create` / `payos.webhooks.verify`. Route path đổi `/api/payment/payos/*` → `/api/payment/{create,webhook,status}` (theo tarot production).
- **VIP + credit chung 1 webhook:** cùng settlement tx xử lý cả credit top-up lẫn VIP grant — không tách 2 webhook.
- **Pairs với auth recipe:** tier/VIP state có thể seed ban đầu ở `onFirstSignIn`.

## Forge verification (2026-07-23)

Anchors chạy chống `~/tarot` @ `cd16a86` (tarot's real paths: `src/lib/payment/payos.ts`, `src/app/api/payment/{create,webhook}/route.ts`):

| # | Anchor | Result |
|---|--------|--------|
| 1 | `@payos/node` in `package.json` + `payos.ts` | ✅ HIT (`^2.0.5`, import present) |
| 2 | `createHmac` absent from `src/app/api/payment/` | ✅ HIT — no raw HMAC found, SDK-only |
| 3 | `new PayOS({` object-arg constructor | ✅ HIT (`payos.ts:19`) |
| 4 | `webhooks.verify` present | ✅ HIT (`payos.ts:69`) |
| 5 | `updateMany` in webhook route | ✅ HIT (`webhook/route.ts:75`) |
| 6 | `status: 500` in webhook route | ✅ HIT (`webhook/route.ts:115`, settlement-fail path) |
| 7 | `status: 400` in webhook route | ✅ HIT (`webhook/route.ts:35,43` — bad JSON + bad signature) |
| 8 | `handleVipPaymentInTx`/`Subscription` in payment lib | ✅ HIT (`webhook/route.ts:129`, tarot's real model is `tx.subscription`, confirmed via Prisma call shape — tarot's own `schema.prisma` not re-read verbatim here, model name inferred from `tx.subscription.findUnique/create/update` calls) |
| 9 | ENV keys vs `.env.example` | not checked (tarot's real `.env.example` not read — out of scope, keys match SDK requirement) |
| 10 | live smoke webhook 400 | not run (no dev server started for this recipe-forge pass — static-code verification only) |

## Source

- DNA: `~/tarot/src/lib/payment/payos.ts` + `~/tarot/src/app/api/payment/{create,webhook,status}/route.ts` (verified @cd16a86)
- Bài học: tarot P004 (revenue-leak pre-charge) + production idempotency/retry pattern
- PayOS docs: https://payos.vn/docs/ · SDK: https://www.npmjs.com/package/@payos/node
