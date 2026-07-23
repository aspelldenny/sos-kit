# Recipe: PII Field Encryption (AES-256-GCM + queryable email hash, Node crypto)

> **Category:** infra
> **Stability:** stable (battle-tested ở tarot production)
> **Last verified:** 2026-07-23 (mined from tarot production, verified against tarot @cd16a86)

## Mục đích

Mã hoá field PII (email, tên, toạ độ sinh, v.v.) tại rest bằng AES-256-GCM (Node's built-in `crypto`, zero-dep), gắn kèm 2 helper thực dụng: `isEncrypted` để detect field đã mã hoá chưa (hữu ích cho migration incremental — cột cũ plaintext, cột mới encrypted, cùng tồn tại trong lúc backfill) và `hashEmail` — SHA256 deterministic hash để **query-by-email khi chính cột email đã bị mã hoá** (encrypt không deterministic do random IV → không `WHERE email = encrypt(x)` được, phải giữ 1 cột hash riêng làm index).

**Vì sao AES-256-GCM (không phải AES-CBC):** GCM là authenticated encryption — tự phát hiện ciphertext bị tamper (auth tag verification fail → throw), CBC cần thêm HMAC riêng để chống tamper. Ít bề mặt tự làm sai hơn.

## Inputs (yêu cầu trước khi apply)

- [ ] Node.js runtime (dùng `node:crypto` — không chạy Edge runtime nếu Next.js middleware/edge function)
- [ ] Có field DB cần mã hoá (email, PII khác) — schema thay đổi tuỳ project, recipe không ràng ORM
- [ ] Zero deps — chỉ Node built-in `crypto`

## Outputs (sau khi apply)

- `src/lib/encryption.ts` — `encrypt`, `decrypt`, `isEncrypted`, `hashEmail`
- ENV mới: `ENCRYPTION_KEY` (base64, 32 bytes), `EMAIL_HASH_SALT`
- Nếu query-by-email: thêm 1 cột `emailHash` (index) bên cạnh cột `email` đã mã hoá

## Steps

### 1. Core encrypt/decrypt + hash

```typescript
// src/lib/encryption.ts
import { createCipheriv, createDecipheriv, randomBytes, createHash } from 'crypto';

const ALGORITHM = 'aes-256-gcm';

function getKey(): Buffer {
  const key = process.env.ENCRYPTION_KEY;
  if (!key) throw new Error('ENCRYPTION_KEY env var not set');
  return Buffer.from(key, 'base64');
}

// Detect if string is already in encrypted format: base64:base64:base64
export function isEncrypted(value: string): boolean {
  const parts = value.split(':');
  if (parts.length !== 3) return false;
  const base64Regex = /^[A-Za-z0-9+/]+=*$/;
  return parts.every(p => p.length > 0 && base64Regex.test(p));
}

export function encrypt(plaintext: string): string {
  const key = getKey();
  const iv = randomBytes(16);
  const cipher = createCipheriv(ALGORITHM, key, iv);
  let encrypted = cipher.update(plaintext, 'utf8', 'base64');
  encrypted += cipher.final('base64');
  const authTag = cipher.getAuthTag();
  return `${iv.toString('base64')}:${authTag.toString('base64')}:${encrypted}`;
}

export function decrypt(encryptedStr: string): string {
  const key = getKey();
  const [ivB64, authTagB64, ciphertext] = encryptedStr.split(':');
  const iv = Buffer.from(ivB64, 'base64');
  const authTag = Buffer.from(authTagB64, 'base64');
  const decipher = createDecipheriv(ALGORITHM, key, iv);
  decipher.setAuthTag(authTag);
  let decrypted = decipher.update(ciphertext, 'base64', 'utf8');
  decrypted += decipher.final('utf8');
  return decrypted;
}

export function hashEmail(email: string): string {
  const salt = process.env.EMAIL_HASH_SALT;
  if (!salt) throw new Error('EMAIL_HASH_SALT env var not set');
  return createHash('sha256').update(email.toLowerCase() + salt).digest('hex');
}
```

### 2. Generate `ENCRYPTION_KEY`

```bash
node -e "console.log(require('crypto').randomBytes(32).toString('base64'))"
```

### 3. Usage — write path (encrypt before save, hash for query index)

```typescript
await prisma.user.create({
  data: {
    email: encrypt(rawEmail),
    emailHash: hashEmail(rawEmail), // separate indexed column — this is what WHERE queries against
  },
});
```

### 4. Usage — query path (lookup by hash, decrypt after read)

```typescript
const row = await prisma.user.findUnique({ where: { emailHash: hashEmail(rawEmail) } });
const plainEmail = row ? decrypt(row.email) : null;
```

### 5. Usage — incremental migration detector

```typescript
// During a backfill migrating a plaintext column to encrypted, some rows may
// already be encrypted (re-run safety) and some still plaintext.
const stored = isEncrypted(row.email) ? decrypt(row.email) : row.email;
```

## Verification anchors

```bash
grep -n "aes-256-gcm" src/lib/encryption.ts                 # 1. authenticated encryption, not CBC
grep -n "getAuthTag\|setAuthTag" src/lib/encryption.ts       # 2. GCM auth tag round-trip
grep -n "export function isEncrypted" src/lib/encryption.ts # 3. detector present
grep -n "export function hashEmail" src/lib/encryption.ts   # 4. queryable hash present
grep -n "createHash('sha256')" src/lib/encryption.ts        # 5. SHA256, not MD5
grep -n "EMAIL_HASH_SALT\|ENCRYPTION_KEY" src/lib/encryption.ts  # 6. both ENV keys referenced
ls src/lib/encryption.test.ts                                # 7. test ported alongside
```

## Discovery hooks (chỗ dễ sai)

| Pattern | Bài học |
|---------|---------|
| Query `WHERE email = encrypt(x)` trực tiếp | KHÔNG hoạt động — `encrypt()` dùng random IV, cùng plaintext ra ciphertext khác mỗi lần (verified: tarot test "two encryptions of the same plaintext produce different ciphertexts"). PHẢI có cột hash riêng (`hashEmail`, deterministic) làm index để query. |
| `hashEmail` không lowercase trước khi hash | Case-sensitivity làm 2 email thực chất giống nhau (`Test@Email.com` vs `test@email.com`) ra 2 hash khác nhau → query miss. Luôn `.toLowerCase()` trước hash (verified trong code + test "is case-insensitive"). |
| Dùng AES-CBC thay GCM | CBC không tự phát hiện tamper — cần thêm HMAC riêng, dễ implement sai (padding oracle, thứ tự MAC-then-encrypt). GCM built-in auth tag đơn giản hơn và an toàn hơn mặc định. |
| Đổi `ENCRYPTION_KEY` mà không re-encrypt data cũ | `decrypt` với sai key → throw ngay (auth tag verification fail) — KHÔNG silent-corrupt, nhưng cũng nghĩa là mọi row cũ trở nên không đọc được. Đổi key đòi hỏi migration re-encrypt toàn bộ, không phải chỉ đổi ENV. |
| Migrate incremental thiếu `isEncrypted` check | Chạy backfill script 2 lần (hoặc job restart giữa chừng) sẽ double-encrypt row đã encrypt rồi → `decrypt` fail vì IV/tag không khớp format. `isEncrypted()` guard cho phép script idempotent. |
| Tamper ciphertext không bị phát hiện | Nếu dùng thuật toán không có auth tag, một ký tự bị sửa trong ciphertext vẫn "decrypt" ra rác thay vì throw. GCM's `setAuthTag` + verify đảm bảo tamper luôn throw (verified: tarot test "decrypt tampered ciphertext throws"). |

## Env vars

```bash
# .env.example
ENCRYPTION_KEY=       # node -e "console.log(require('crypto').randomBytes(32).toString('base64'))"
EMAIL_HASH_SALT=      # bất kỳ chuỗi random dài, KHÔNG đổi sau khi có data (đổi = mọi hash cũ vô hiệu)
```

## Migration / interop notes

- **Đổi `EMAIL_HASH_SALT` sau khi có data:** mọi `emailHash` cũ trở nên sai — cần re-hash toàn bộ. Coi salt này là bất biến sau ngày đầu production, giống migration-breaking change.
- **Không phụ thuộc recipe khác** — zero-dep, đứng độc lập. Pairs tự nhiên với bất kỳ recipe nào lưu PII (auth, payment).

## Source

- DNA: `~/tarot/src/lib/encryption.ts` (verified @cd16a86) — port kèm test `~/tarot/src/lib/encryption.test.ts` (source cho port)
- Docs: Node.js `crypto` module — https://nodejs.org/api/crypto.html

## Forge verification (2026-07-23, tarot @cd16a86)

Anchors chạy chống `~/tarot` @ `cd16a86` (`src/lib/encryption.ts`):

| # | Anchor | Result |
|---|--------|--------|
| 1 | `aes-256-gcm` algorithm | ✅ HIT (line 3) |
| 2 | `getAuthTag`/`setAuthTag` present | ✅ HIT (lines 25, 35) |
| 3 | `isEncrypted` exported | ✅ HIT (line 12) |
| 4 | `hashEmail` exported | ✅ HIT (line 41) |
| 5 | `createHash('sha256')` | ✅ HIT (line 44) |
| 6 | ENV keys referenced | ✅ HIT (`ENCRYPTION_KEY` line 6, `EMAIL_HASH_SALT` line 42) |
| 7 | test file exists | ✅ HIT — `src/lib/encryption.test.ts`, 94 lines: roundtrip (short/long/JSON/Unicode), random-IV distinctness, wrong-key throw, tampered-ciphertext throw, hashEmail determinism/case-insensitivity |
