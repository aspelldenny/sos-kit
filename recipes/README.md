# Recipes — Atomic, Composable Building Blocks

> **Triết lý:** sos-kit không có "scaffolds cứng" theo combo (Next+Prisma+Postgres+Docker). Recipes là **đơn vị nhỏ nhất** — 1 file = 1 recipe — chọn theo nhu cầu của dự án.

## Tại sao recipes thay scaffolds

| | Scaffold cứng | Recipe library |
|---|---|---|
| Đơn vị | Cả stack | 1 mảnh ghép |
| Combo lạ | Phải tạo scaffold mới | Mix recipe có sẵn |
| Maintain | N×M×K (tổ hợp nổ) | Tuyến tính |
| Project mới | Phụ thuộc combo có sẵn | Mix; thiếu thì viết recipe mới từ code đã ship |

## Categories

```
recipes/
├── infra/              docker-compose, vps-bootstrap, nginx, postgres, redis...
├── auth/               nextauth, supabase, jwt-custom, clerk...
├── payment/            payos-vn, stripe-checkout, lemonsqueezy...
├── ai/                 multi-model-fallback, credit-atomic-deduct, embeddings...
├── observability/      sentry, umami, canary-github-actions, uptime-monitor...
└── framework-starter/  nextjs, sveltekit, flask, fastapi, tauri... (minimal scaffold only)
```

## Format mỗi recipe

Recipe = 1 file Markdown duy nhất, có structure:

```
# Recipe: <Name>

> Category, Stability, Last verified date

## Mục đích — 1 đoạn what + why
## Inputs — recipe nào phải apply trước
## Outputs — sau khi apply có gì
## Steps — code blocks + commands cụ thể
## Verification anchors — bash commands để check apply thành công
## Discovery hooks — chỗ dễ sai trên thực tế (DNA từ project trước)
## Env vars — list keys cần thêm vào .env.example
## Source — pointer tới project DNA gốc + docs official
```

Template chi tiết: `recipes/_TEMPLATE.md`

## Cách dùng

Trong brief giao cho Thợ, ghi recipe cần áp (ví dụ `payment/payos-vn`). Thợ dùng skill `apply` (`skills/apply/SKILL.md`): đọc recipe, đối chiếu đầu vào với code thật, làm các bước cho khớp dự án, chạy hết verification anchors, báo bằng chứng. Mỗi recipe một commit hoặc một PR.

## Thêm recipe mới

Chỉ thêm recipe đã chạy thật trong một dự án đã ship. Dùng `recipes/_TEMPLATE.md`; bắt buộc có verification anchors chạy được và ít nhất một discovery hook. Recipe là thay đổi của sos-kit, đi qua PR như mọi thay đổi khác.

## Recipe đã có

### Stable (battle-tested)
- `payment/payos-vn` — Tích hợp PayOS VN (SDK official) với pre-charge + atomic deduct + VIP-qua-topup (DNA tarot)
- `auth/nextauth-google-credentials` — NextAuth v4 Google OAuth + Credentials (email/password bcrypt), JWT strategy (DNA tarot)
- `ai/multi-model-fallback` — Opus → Gemini → OpenRouter chain với timeout per-tier (DNA tarot)
- `infra/rate-limit-inmemory` — Sliding-window rate limit theo IP (zero-dep `Map`) + login-attempt limiter, IP-spoofing-safe header priority (DNA tarot)
- `infra/pii-encryption` — AES-256-GCM field encryption + queryable email hash (Node `crypto`, zero-dep) (DNA tarot)
- `ai/sse-streaming-keepalive` — SSE idle-disconnect guard cho streaming routes trong lúc model reasoning im lặng (zero-dep) (DNA tarot)

### TODO (priority cao theo experience tarot)
- `infra/docker-compose-postgres` — Postgres 16 self-host + Prisma init
- `infra/docker-compose-nginx` — Nginx reverse proxy + Cloudflare cert
- `infra/vps-bootstrap-ubuntu` — UFW + fail2ban + SSH hardening + certbot
- `observability/sentry-nextjs` — Sentry SDK + source maps + tracing
- `observability/umami-self-host` — Umami analytics docker
- `observability/canary-github-actions` — Post-deploy health check workflow
- `framework-starter/nextjs-15-app-router` — Next 15 minimal scaffold (no DB, no auth — pure framework)
