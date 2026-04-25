# Recipes — Atomic, Composable Building Blocks

> **Triết lý:** sos-kit không có "scaffolds cứng" theo combo (Next+Prisma+Postgres+Docker). Recipes là **đơn vị nhỏ nhất** — 1 file = 1 recipe — Kiến trúc sư mix theo blueprint của project.

## Tại sao recipes thay scaffolds

| | Scaffold cứng | Recipe library |
|---|---|---|
| Đơn vị | Cả stack | 1 mảnh ghép |
| Combo lạ | Phải tạo scaffold mới | Mix recipe có sẵn |
| Maintain | N×M×K (tổ hợp nổ) | Tuyến tính |
| Project mới | Phụ thuộc combo có sẵn | Mix + forge nếu thiếu |

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

## Cách Kiến trúc sư dùng

Trong `phieu/P000-genesis.md > 3. Tech Commitments > Recipes to apply`:

```
1. infra/vps-bootstrap-ubuntu
2. infra/docker-compose-postgres
3. framework-starter/nextjs-15-app-router
4. auth/nextauth-google-email
5. payment/payos-vn
6. ai/multi-model-fallback
7. observability/sentry-nextjs
```

Thợ chạy `sos apply <name>` lần lượt — mỗi recipe = 1 phiếu sub-genesis (P000.1, P000.2, ...) với verification anchors riêng.

## Cách forge recipe mới

Khi blueprint cần combo mà library thiếu:

```bash
sos recipe new <category>/<name>
# Hoặc invoke skill /forge trong Claude Code
```

Skill `/forge` dẫn Kiến trúc sư qua: nghiên cứu official docs → write steps → verify trên test project → save vào `recipes/<category>/<name>.md` → commit.

**Quy tắc forge:** Mọi recipe mới PHẢI có Verification anchors chạy được + ít nhất 1 Discovery hook (anticipate failure mode). Không recipe nào ship "untested".

## Recipe đã có

### Stable (battle-tested)
- `payment/payos-vn` — Tích hợp PayOS VN với pre-charge + atomic deduct (DNA tarot)
- `ai/multi-model-fallback` — Opus → Gemini → OpenRouter chain với timeout per-tier (DNA tarot)

### TODO (priority cao theo experience tarot)
- `infra/docker-compose-postgres` — Postgres 16 self-host + Prisma init
- `infra/docker-compose-nginx` — Nginx reverse proxy + Cloudflare cert
- `infra/vps-bootstrap-ubuntu` — UFW + fail2ban + SSH hardening + certbot
- `auth/nextauth-google-email` — NextAuth Google OAuth + Email magic link
- `observability/sentry-nextjs` — Sentry SDK + source maps + tracing
- `observability/umami-self-host` — Umami analytics docker
- `observability/canary-github-actions` — Post-deploy health check workflow
- `framework-starter/nextjs-15-app-router` — Next 15 minimal scaffold (no DB, no auth — pure framework)
