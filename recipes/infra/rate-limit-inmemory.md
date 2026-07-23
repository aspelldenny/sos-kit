# Recipe: In-Memory Rate Limiting (Next.js, sliding window, zero-dep)

> **Category:** infra
> **Stability:** stable (battle-tested ở tarot production)
> **Last verified:** 2026-07-23 (mined from tarot production, verified against tarot @cd16a86)

## Mục đích

Rate-limit theo IP cho Next.js App Router route/middleware, KHÔNG cần Redis/Upstash — một `Map` module-level trong tiến trình. Đủ cho single-instance / low-to-mid scale; chấp nhận reset khi cold-start serverless. Zero-dep (chỉ dùng `next/server` types). Hai use case: (1) generic bucket limiter cho bất kỳ route nào, (2) limiter chuyên biệt cho login/credentials — chống brute-force với cửa sổ dài hơn (15 phút) và limit thấp hơn (5 lần).

**Chọn in-memory thay vì Redis vì:** solo-dev/single-VPS scale không cần distributed store; thêm Redis chỉ để rate-limit là over-engineering theo Philosophy #5 (solo-first). Nếu scale ra multi-instance, thay `Map` bằng Redis `INCR`+`EXPIRE` — interface (`rateLimitByIP`, `checkLoginRateLimit`) giữ nguyên chữ ký, chỉ đổi implementation bên trong.

## Inputs (yêu cầu trước khi apply)

- [ ] Next.js App Router (`next/server` — `NextRequest`)
- [ ] Đứng sau Cloudflare hoặc nginx set `X-Real-IP` — nếu KHÔNG có reverse proxy đáng tin, xem Discovery hook "IP header nào tin được"
- [ ] Zero deps ngoài `next` — không cần cài gì thêm

## Outputs (sau khi apply)

- `src/lib/rate-limit.ts` — `rateLimitByIP`, `getRetryAfterSeconds`, `checkLoginRateLimit`
- Không có ENV mới, không có DB migration

## Steps

### 1. Generic bucket limiter

```typescript
// src/lib/rate-limit.ts
import { NextRequest } from 'next/server';

interface RateLimitRecord {
  count: number;
  windowStart: number;
}

// In-memory store: bucket:IP → sliding window record
// Note: resets on cold start (serverless) — acceptable for current scale
const store = new Map<string, RateLimitRecord>();

/**
 * Check if request from this IP exceeds rate limit.
 * Sliding window: counts requests within the last `windowMs` milliseconds.
 *
 * IP source order matters (see Discovery hook "IP header nào tin được"):
 * prefer a header set by YOUR reverse proxy/CDN (cannot be spoofed by the client)
 * over client-supplied X-Forwarded-For.
 *
 * `bucket` param allows per-route independent counts — pass a unique bucket name
 * for stricter per-handler limits without double-counting against a middleware default.
 *
 * @returns true if rate limit exceeded (should block), false if OK
 */
export function rateLimitByIP(
  req: NextRequest,
  limit = 60,
  windowMs = 60_000,
  bucket = 'default'
): boolean {
  // Prefer a proxy-set header (Cloudflare `cf-connecting-ip`, or your reverse proxy's
  // own header, e.g. nginx setting X-Real-IP from $http_cf_connecting_ip) over the
  // client-spoofable X-Forwarded-For. Only fall back to XFF in non-production (local dev
  // has no reverse proxy in front).
  const ip =
    req.headers.get('cf-connecting-ip') ||
    req.headers.get('x-real-ip') ||
    (process.env.NODE_ENV !== 'production'
      ? req.headers.get('x-forwarded-for')?.split(',')[0]?.trim() || 'unknown'
      : 'unknown');

  const key = `${bucket}:${ip}`;

  const now = Date.now();
  const record = store.get(key);

  if (!record || now - record.windowStart > windowMs) {
    store.set(key, { count: 1, windowStart: now });
    return false;
  }

  record.count += 1;
  if (record.count > limit) {
    return true;
  }

  return false;
}

/** Returns Retry-After seconds based on the window */
export function getRetryAfterSeconds(windowMs = 60_000): number {
  return Math.ceil(windowMs / 1000);
}
```

### 2. Login attempt limiter (stricter, longer window)

```typescript
// appended to src/lib/rate-limit.ts
// Login attempt limiter: 5 attempts per IP per 15-minute window.
// Covers credentials-based login only — separate store from the generic bucket
// above so a busy API route can't exhaust the login budget or vice versa.

const LOGIN_LIMIT = 5;
const LOGIN_WINDOW_MS = 15 * 60_000; // 15 minutes

const loginStore = new Map<string, RateLimitRecord>();

/**
 * Check if IP has exceeded login attempt limit.
 * @returns true if blocked (should deny), false if OK
 */
export function checkLoginRateLimit(ip: string): boolean {
  const now = Date.now();
  const record = loginStore.get(ip);

  if (!record || now - record.windowStart > LOGIN_WINDOW_MS) {
    loginStore.set(ip, { count: 1, windowStart: now });
    return false;
  }

  record.count += 1;
  return record.count > LOGIN_LIMIT;
}
```

### 3. Usage — generic bucket in a route handler

```typescript
// src/app/api/some-route/route.ts
import { rateLimitByIP, getRetryAfterSeconds } from '@/lib/rate-limit';

export async function POST(req: NextRequest) {
  if (rateLimitByIP(req, 30, 60_000, 'some-route')) {
    return Response.json(
      { error: 'rate limited' },
      { status: 429, headers: { 'Retry-After': String(getRetryAfterSeconds(60_000)) } }
    );
  }
  // ... handler logic
}
```

### 4. Usage — login limiter inside `authorize()` (pairs with `auth/nextauth-google-credentials`)

```typescript
// in CredentialsProvider.authorize(), BEFORE the DB lookup:
const ip = req.headers.get('cf-connecting-ip') || req.headers.get('x-real-ip') || 'unknown';
if (checkLoginRateLimit(ip)) return null; // blocked — do not even hit the DB
```

## Verification anchors

```bash
grep -n "cf-connecting-ip" src/lib/rate-limit.ts                    # 1. proxy-header priority
grep -n "x-forwarded-for" src/lib/rate-limit.ts                     # 2. XFF is fallback only, not primary
grep -n "bucket:.*ip\|\`\${bucket}:\${ip}\`" src/lib/rate-limit.ts  # 3. bucket-scoped key
grep -n "LOGIN_LIMIT = 5" src/lib/rate-limit.ts                     # 4. login limit constant
grep -n "LOGIN_WINDOW_MS = 15" src/lib/rate-limit.ts                # 5. 15-min window
grep -n "checkLoginRateLimit" src/lib/rate-limit.ts                 # 6. exported login limiter
ls src/lib/rate-limit.test.ts                                       # 7. test ported alongside
```

## Discovery hooks (chỗ dễ sai)

| Pattern | Bài học |
|---------|---------|
| IP header nào tin được (tarot vbsec HIGH-13) | `X-Forwarded-For` do CLIENT set được — dùng làm khoá rate-limit thì attacker tự đổi IP header, bypass limit vô hạn. PHẢI ưu tiên header do reverse proxy/CDN của BẠN set (không đi qua client) — ví dụ Cloudflare's `cf-connecting-ip`, hoặc nginx tự gán `X-Real-IP` từ header đó. XFF chỉ dùng làm fallback ở dev (không có proxy phía trước). |
| Rate-limit store là module-level `Map` | Reset khi cold-start serverless (mỗi instance có store riêng) — chấp nhận được ở scale nhỏ, nhưng đừng kỳ vọng limit chính xác across nhiều instance. Cần chính xác hơn → chuyển sang Redis `INCR`+`EXPIRE`, giữ nguyên chữ ký hàm. |
| Bucket key thiếu → double-count | Không truyền `bucket` riêng cho route cụ thể → route đó dùng chung counter `'default'` với middleware-level limiter, dễ tự trigger limit của chính mình. Luôn đặt `bucket` riêng cho limiter thứ 2 trở đi. |
| Login limiter check SAU DB lookup | Nếu check `checkLoginRateLimit` sau khi đã query DB, brute-force vẫn tốn DB round-trip mỗi lần — luôn check TRƯỚC lookup. |
| Test cross-contamination (module-level `Map` share giữa test) | Rate-limit store persist giữa các test trong cùng file — port kèm test PHẢI dùng unique IP mỗi test case (xem `rate-limit.test.ts` pattern `uniqueIp()`), không seq shared IP. |

## Env vars

Không có — zero-dep, không ENV.

## Migration / interop notes

- **Pairs với `auth/nextauth-google-credentials`:** login limiter (`checkLoginRateLimit`) gọi trong `authorize()` trước DB lookup — xem Discovery hook trong recipe auth "Login brute-force".
- **Scale lên multi-instance:** thay `Map` bằng Redis, giữ nguyên public API (`rateLimitByIP`, `checkLoginRateLimit`, `getRetryAfterSeconds`) để caller không đổi.

## Source

- DNA: `~/tarot/src/lib/rate-limit.ts` (verified @cd16a86) — port kèm test `~/tarot/src/lib/rate-limit.test.ts` (source cho port)
- Bài học: tarot P247 vbsec HIGH-13 (IP-spoofing qua XFF), vbsec HIGH-4 (bucket scope)

## Forge verification (2026-07-23, tarot @cd16a86)

Anchors chạy chống `~/tarot` @ `cd16a86` (`src/lib/rate-limit.ts`):

| # | Anchor | Result |
|---|--------|--------|
| 1 | `cf-connecting-ip` present | ✅ HIT (line 36) |
| 2 | `x-forwarded-for` fallback-only | ✅ HIT (line 39, gated by `NODE_ENV !== 'production'`) |
| 3 | bucket:ip key construction | ✅ HIT (line 44, `` `${bucket}:${ip}` ``) |
| 4 | `LOGIN_LIMIT = 5` | ✅ HIT (line 70) |
| 5 | `LOGIN_WINDOW_MS = 15 * 60_000` | ✅ HIT (line 71) |
| 6 | `checkLoginRateLimit` exported | ✅ HIT (line 79) |
| 7 | test file exists | ✅ HIT — `src/lib/rate-limit.test.ts`, 116 lines, 11 test cases incl. sliding-window expiry + unknown-bucket + x-real-ip precedence |
