# Recipe: NextAuth — Google + Credentials (Next.js App Router)

> **Category:** auth
> **Stability:** stable (battle-tested ở tarot production)
> **Last verified:** 2026-07-23 (mined from tarot production, verified against tarot @cd16a86)

## Mục đích

Auth cho Next.js 15 App Router, 2 đường song song: **Google OAuth** (1-click) + **Credentials** (email/password bcrypt). NextAuth v4, JWT strategy (no adapter table). Business-logic (referral/tier) tách thành hook-point `onFirstSignIn` để tái dùng. Chọn v4 (không v5/Auth.js beta) vì tarot ổn định trên `next-auth@4.24.14`.

## Inputs (yêu cầu trước khi apply)

- [ ] Next.js 15 App Router (`app/` dir)
- [ ] User model có `passwordHash` (Credentials) + email identity — recipe không ràng ORM (ví dụ Prisma)
- [ ] Google Cloud OAuth client (redirect URI `<origin>/api/auth/callback/google`)
- [ ] Deps `next-auth@4.24.14`, `bcryptjs@3.0.3`, `zod@4.4.3` (verified against tarot `package.json` @cd16a86)

## Outputs (sau khi apply)

- `src/lib/auth.ts` — `authOptions` (providers + callbacks + events)
- `src/app/api/auth/[...nextauth]/route.ts` — handler GET+POST
- ENV: `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `NEXTAUTH_SECRET`, `NEXTAUTH_URL`
- Hook-point `onFirstSignIn(user)`

## Steps

### 1. `authOptions`

```typescript
// src/lib/auth.ts
import type { NextAuthOptions } from "next-auth";
import GoogleProvider from "next-auth/providers/google";
import CredentialsProvider from "next-auth/providers/credentials";
import bcrypt from "bcryptjs";
import { z } from "zod";
import { prisma } from "@/lib/prisma";

const credentialsSchema = z.object({ email: z.string().email(), password: z.string().min(1) });

export const authOptions: NextAuthOptions = {
  session: { strategy: "jwt" },
  secret: process.env.NEXTAUTH_SECRET,
  pages: { signIn: "/login", error: "/login" }, // optional — tarot routes NextAuth error codes back to /login?error=
  providers: [
    GoogleProvider({
      clientId: process.env.GOOGLE_CLIENT_ID!,
      clientSecret: process.env.GOOGLE_CLIENT_SECRET!,
      // GOTCHA tarot P146: Google session ở accounts.google.com không clear khi user
      // signOut khỏi app — default sẽ auto-pick account đang active trên browser.
      // prompt=select_account ép Google show account chooser mỗi lần.
      authorization: { params: { prompt: "select_account" } },
    }),
    CredentialsProvider({
      name: "Credentials",
      credentials: { email: { label: "Email", type: "email" }, password: { label: "Password", type: "password" } },
      async authorize(raw) {
        // NOTE: tarot's real authorize() does a plain `!credentials?.email || !credentials?.password`
        // null-check (no zod) + a login-rate-limit check keyed by IP before the DB lookup — zod here
        // is a recipe-level hardening on top, not a literal tarot mining. Keep it; it's cheap insurance.
        const parsed = credentialsSchema.safeParse(raw);
        if (!parsed.success) return null;
        const { email, password } = parsed.data;
        const user = await prisma.user.findUnique({ where: { email } });
        if (!user?.passwordHash) return null; // Google-only account
        const ok = await bcrypt.compare(password, user.passwordHash);
        if (!ok) return null;
        return { id: user.id, email: user.email, name: user.name };
      },
    }),
  ],
  callbacks: {
    async signIn({ user, account }) { await onFirstSignIn({ user, account }); return true; },
    // GOTCHA (confirmed in tarot/src/lib/auth.ts:70-119): WITHOUT a DB adapter, the `user` object
    // the jwt() callback receives on first sign-in for an OAuth provider carries the PROVIDER's id
    // (Google's `sub`), NOT your app's DB user id. Credentials provider is fine (authorize() already
    // returns your DB id) — Google is the trap. onFirstSignIn must upsert the DB user BEFORE jwt()
    // runs (signIn() always fires first in the same request), then jwt() re-resolves by email and
    // overwrites token.sub with the real DB id.
    async jwt({ token, user }) {
      if (user) {
        const dbUser = await prisma.user.findUnique({ where: { email: user.email! } });
        if (dbUser) token.sub = dbUser.id;
      }
      return token;
    },
    async session({ session, token }) { if (session.user && token.sub) (session.user as any).id = token.sub; return session; },
  },
  events: {
    // Best-effort — never throw inside signOut, NextAuth doesn't awaits-and-fails gracefully here.
    // tarot uses this hook to clear an app-specific secondary cookie (`lite_session`, P111) that
    // isn't part of the NextAuth token — generalize to "any side-channel cookie/cache your app sets
    // outside NextAuth's own session cookie".
    async signOut(_message) {
      try {
        // await clearYourSideChannelCookie();
      } catch {
        // Best-effort: never block signOut.
      }
    },
  },
};
```

### 2. Hook-point `onFirstSignIn` (generic — thay chỗ tarot business logic)

```typescript
// src/lib/auth-hooks.ts
// Chỗ tarot đặt grantReferralReward + authLevel upgrade (auth.ts:70-119, signIn callback).
// Recipe generic → app tự impl; PHẢI upsert DB user ở đây trước khi jwt() chạy (xem gotcha ở Step 1).
export async function onFirstSignIn({ user, account }: { user: any; account: any }) {
  // 1. upsert DB user by email (create if not exists — this is what makes jwt()'s re-lookup work)
  // 2. detect first login (row just created) → seed profile / apply referral / grant welcome credits
}
```

**Đã GỠ (tarot-specific, confirmed at `auth.ts:70-119`):** `grantReferralReward(referrer.id, newUser.id)` (ref_code cookie → referral credit) + `authLevel` upgrade (`"lite"` → `"registered"`) inline in the `signIn` callback. Both moved into `onFirstSignIn` — the recipe keeps only the generic "detect first login, do X" shape.

### 3. App Router handler

```typescript
// src/app/api/auth/[...nextauth]/route.ts
import NextAuth from "next-auth";
import { authOptions } from "@/lib/auth";
const handler = NextAuth(authOptions);
export { handler as GET, handler as POST };
```
(Verified verbatim against `tarot/src/app/api/auth/[...nextauth]/route.ts` — 0 diff.)

## Verification anchors

```bash
grep -n "GoogleProvider\|CredentialsProvider" src/lib/auth.ts        # 1. cả 2 provider
grep -n "select_account" src/lib/auth.ts                             # 2. gotcha P146
grep -n "bcrypt.compare" src/lib/auth.ts                             # 3. không so plaintext
grep -n "as GET, handler as POST" "src/app/api/auth/[...nextauth]/route.ts"  # 4. App Router export
grep -E '"next-auth"|"bcryptjs"|"zod"' package.json                  # 5. deps version
grep -E "GOOGLE_CLIENT_ID|GOOGLE_CLIENT_SECRET|NEXTAUTH_SECRET|NEXTAUTH_URL" .env.example  # 6. ENV
curl -s http://localhost:3000/api/auth/providers | grep -o '"google"\|"credentials"'       # 7. live smoke → cả 2
```

## Discovery hooks (chỗ dễ sai)

| Pattern | Bài học |
|---------|---------|
| `prompt=select_account` thiếu (tarot P146) | Google auto-login account đã login trên browser → user kẹt account cũ, không đổi được. `authorization.params.prompt="select_account"` ép chooser mỗi lần. |
| Dùng `NoAdapter` + JWT strategy nhưng quên override `token.sub` cho OAuth | Không có adapter → `jwt()` callback nhận `user.id` = ID của provider (Google `sub`), không phải DB id. PHẢI re-lookup DB user by email và ghi đè `token.sub`. Credentials provider không dính lỗi này (authorize() đã trả DB id). |
| `authorize` return thiếu `id` | NextAuth cần `id` build token; thiếu → `session.user.id` undefined downstream. |
| So password plaintext | PHẢI `bcrypt.compare`. Google-only user không có `passwordHash` → return null sớm. |
| Business logic nhét thẳng callback | callback chạy MỌI login → khó tách + chạy lặp. Tách `onFirstSignIn`, detect "first" qua flag DB. |
| `NEXTAUTH_SECRET` thiếu prod | dev auto-gen, prod bắt buộc set (JWT sign) — thiếu → session vỡ silently sau restart. |
| `NEXTAUTH_URL` sai origin | callback redirect dùng URL này — sai → `redirect_uri_mismatch`. Khớp origin thật + Google Console. |
| Login brute-force (tarot: `checkLoginRateLimit(ip)`) | tarot rate-limits Credentials `authorize()` by IP (5 attempts/15min) trước khi hit DB — recipe không bắt buộc nhưng khuyến nghị nếu app public. |

## Env vars

```bash
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=
NEXTAUTH_SECRET=          # openssl rand -base64 32
NEXTAUTH_URL=http://localhost:3000
```

## Migration / interop notes

- **Pairs với payment recipe:** `onFirstSignIn` là chỗ seed credit/tier ban đầu — nhưng logic ở app, không ở recipe.
- **Google-only vs Credentials:** 1 email có thể chỉ có Google (no `passwordHash`) → `authorize` return null khi thiếu hash.
- **NextAuth v5/Auth.js:** recipe này v4. Nâng v5 đổi import path + `route.ts` shape — không cover.

## Forge verification (2026-07-23)

Anchors chạy chống `~/tarot` @ `cd16a86` (tarot's real paths: `src/lib/auth.ts`, `src/app/api/auth/[...nextauth]/route.ts`):

| # | Anchor | Result |
|---|--------|--------|
| 1 | `GoogleProvider\|CredentialsProvider` in `auth.ts` | ✅ HIT (both, lines 15/25) |
| 2 | `select_account` in `auth.ts` | ✅ HIT (line 22, P146 comment intact) |
| 3 | `bcrypt.compare` in `auth.ts` | ✅ HIT (line 49) |
| 4 | `as GET, handler as POST` in `route.ts` | ✅ HIT (verbatim) |
| 5 | deps regex vs `package.json` | ✅ HIT — `next-auth@^4.24.14`, `bcryptjs@^3.0.3`, `zod@^4.4.3` |
| 6 | ENV keys vs `.env.example` | not checked (tarot's real `.env.example` not read — out of scope, keys are standard NextAuth) |
| 7 | live smoke `/api/auth/providers` | not run (no dev server started for this recipe-forge pass — static-code verification only) |

## Source

- DNA: `~/tarot/src/lib/auth.ts` (verified @cd16a86) + `.../api/auth/[...nextauth]/route.ts` (`grantReferralReward`/`authLevel` stripped → `onFirstSignIn`)
- Bài học: tarot P146 (`prompt=select_account`), P111 (signOut side-channel cookie), rate-limit pattern
- Docs: https://next-auth.js.org/ (v4)
