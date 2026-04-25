# Recipe: Multi-Model AI Fallback Chain

> **Category:** ai
> **Stability:** stable (battle-tested ở tarot — Opus 4.6 v98 → Gemini Flash → OpenRouter)
> **Last verified:** 2026-04-25

## Mục đích

Một AI call tới Claude/Gemini có thể fail vì: timeout, rate limit, model down, prompt bị filter, region block. Single-model = single point of failure. Recipe này thiết lập **fallback chain** với timeout per-model, retry per-tier, semantic-equivalent prompts cho các tier khác nhau.

## Inputs (yêu cầu trước)

- [ ] Có ít nhất 2 provider key (VD: `ANTHROPIC_API_KEY` + `OPENROUTER_API_KEY`)
- [ ] Recipe `framework-starter/<chosen>` đã apply (cần TypeScript runtime hoặc tương đương)

## Outputs

- `lib/ai/chain.ts` — fallback orchestrator
- `lib/ai/providers/{anthropic,gemini,openrouter}.ts` — adapter cho mỗi provider
- `lib/ai/prompts/<task>.ts` — prompt versions per-tier
- Telemetry hook: log model + latency + tokens cho mỗi call

## Steps

### 1. Provider adapter interface

```typescript
// lib/ai/providers/types.ts
export type ChainContext = {
  prompt: string;
  systemPrompt?: string;
  maxTokens: number;
  temperature?: number;
  timeoutMs: number;
  signal?: AbortSignal;
};

export type ChainResult =
  | { ok: true; text: string; model: string; latencyMs: number; tokensIn: number; tokensOut: number }
  | { ok: false; error: string; provider: string; retryable: boolean };

export interface Provider {
  name: string;
  call(ctx: ChainContext): Promise<ChainResult>;
}
```

### 2. Anthropic adapter (primary tier)

```typescript
// lib/ai/providers/anthropic.ts
import Anthropic from "@anthropic-ai/sdk";
import type { Provider, ChainContext, ChainResult } from "./types";

const client = new Anthropic({ apiKey: process.env.ANTHROPIC_API_KEY! });

export const anthropicOpus: Provider = {
  name: "anthropic:opus-4-7",
  async call(ctx: ChainContext): Promise<ChainResult> {
    const start = Date.now();
    try {
      const res = await client.messages.create({
        model: "claude-opus-4-7",
        max_tokens: ctx.maxTokens,
        system: ctx.systemPrompt,
        messages: [{ role: "user", content: ctx.prompt }],
        temperature: ctx.temperature ?? 0.7,
      }, { signal: ctx.signal, timeout: ctx.timeoutMs });

      const text = res.content[0].type === "text" ? res.content[0].text : "";
      return {
        ok: true, text, model: this.name,
        latencyMs: Date.now() - start,
        tokensIn: res.usage.input_tokens,
        tokensOut: res.usage.output_tokens,
      };
    } catch (err: any) {
      const retryable = err.status === 429 || err.status === 529 || err.name === "AbortError";
      return { ok: false, error: err.message, provider: this.name, retryable };
    }
  },
};
```

### 3. OpenRouter adapter (fallback tier — universal)

```typescript
// lib/ai/providers/openrouter.ts
export const openRouterGeminiFlash: Provider = {
  name: "openrouter:gemini-2.5-flash",
  async call(ctx) {
    const start = Date.now();
    try {
      const res = await fetch("https://openrouter.ai/api/v1/chat/completions", {
        method: "POST",
        headers: {
          "Authorization": `Bearer ${process.env.OPENROUTER_API_KEY}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          model: "google/gemini-2.5-flash",
          messages: [
            ...(ctx.systemPrompt ? [{ role: "system", content: ctx.systemPrompt }] : []),
            { role: "user", content: ctx.prompt },
          ],
          max_tokens: ctx.maxTokens,
          temperature: ctx.temperature ?? 0.7,
        }),
        signal: AbortSignal.timeout(ctx.timeoutMs),
      });
      const json = await res.json();
      if (!res.ok) {
        return { ok: false, error: json.error?.message ?? `HTTP ${res.status}`, provider: this.name, retryable: res.status >= 500 || res.status === 429 };
      }
      return {
        ok: true,
        text: json.choices[0].message.content,
        model: this.name,
        latencyMs: Date.now() - start,
        tokensIn: json.usage?.prompt_tokens ?? 0,
        tokensOut: json.usage?.completion_tokens ?? 0,
      };
    } catch (err: any) {
      return { ok: false, error: err.message, provider: this.name, retryable: err.name === "AbortError" || err.name === "TimeoutError" };
    }
  },
};
```

### 4. Chain orchestrator

```typescript
// lib/ai/chain.ts
import { anthropicOpus } from "./providers/anthropic";
import { openRouterGeminiFlash } from "./providers/openrouter";
import { logAiCall } from "./telemetry";

type Tier = {
  provider: Provider;
  timeoutMs: number;
  retries: number; // số lần retry trong tier này nếu retryable error
};

const DEFAULT_CHAIN: Tier[] = [
  { provider: anthropicOpus,        timeoutMs: 30_000, retries: 1 },
  { provider: openRouterGeminiFlash, timeoutMs: 20_000, retries: 1 },
];

export async function callChain(
  prompt: string,
  opts: { systemPrompt?: string; maxTokens?: number; temperature?: number; chain?: Tier[] } = {}
): Promise<ChainResult> {
  const chain = opts.chain ?? DEFAULT_CHAIN;
  let lastError: ChainResult & { ok: false } | null = null;

  for (const tier of chain) {
    for (let attempt = 0; attempt <= tier.retries; attempt++) {
      const result = await tier.provider.call({
        prompt,
        systemPrompt: opts.systemPrompt,
        maxTokens: opts.maxTokens ?? 4096,
        temperature: opts.temperature,
        timeoutMs: tier.timeoutMs,
      });

      logAiCall({ tier: tier.provider.name, attempt, result });

      if (result.ok) return result;
      lastError = result;
      if (!result.retryable) break; // đi tier kế tiếp
      if (attempt < tier.retries) await new Promise(r => setTimeout(r, 500 * (attempt + 1))); // backoff
    }
  }

  return lastError ?? { ok: false, error: "all tiers exhausted", provider: "chain", retryable: false };
}
```

### 5. Telemetry hook

```typescript
// lib/ai/telemetry.ts
export async function logAiCall(args: {
  tier: string;
  attempt: number;
  result: ChainResult;
}) {
  await prisma.aiCallLog.create({
    data: {
      tier: args.tier,
      attempt: args.attempt,
      ok: args.result.ok,
      latencyMs: args.result.ok ? args.result.latencyMs : null,
      tokensIn: args.result.ok ? args.result.tokensIn : null,
      tokensOut: args.result.ok ? args.result.tokensOut : null,
      errorMessage: args.result.ok ? null : args.result.error,
    },
  });
}
```

```prisma
// prisma/schema.prisma
model AiCallLog {
  id          String   @id @default(cuid())
  tier        String
  attempt     Int
  ok          Boolean
  latencyMs   Int?
  tokensIn    Int?
  tokensOut   Int?
  errorMessage String?
  createdAt   DateTime @default(now())

  @@index([tier, createdAt])
  @@index([ok, createdAt])
}
```

### 6. Per-tier prompt variants (optional)

Một số task hoạt động tốt với prompt khác nhau cho model khác nhau. Pattern:

```typescript
// lib/ai/prompts/tarot-reading.ts
export function buildPrompt(opts: { tier: string; userQuery: string }) {
  if (opts.tier.startsWith("anthropic:")) return buildOpusPrompt(opts.userQuery); // dài, nuance
  if (opts.tier.startsWith("openrouter:gemini")) return buildGeminiPrompt(opts.userQuery); // ngắn, structured
  return buildGenericPrompt(opts.userQuery);
}
```

Caller:

```typescript
const result = await callChain(buildPrompt({ tier: "...", userQuery }), { ... });
// Hoặc nếu cần per-tier prompt khác hẳn → loop manual:
for (const tier of chain) {
  const prompt = buildPrompt({ tier: tier.provider.name, userQuery });
  const r = await tier.provider.call({ prompt, ... });
  if (r.ok) return r;
}
```

## Verification anchors

```bash
# 1. Provider files exist
ls lib/ai/providers/{anthropic,openrouter}.ts

# 2. Chain orchestrator có DEFAULT_CHAIN
grep "DEFAULT_CHAIN" lib/ai/chain.ts

# 3. Telemetry table migrated
grep "AiCallLog" prisma/schema.prisma

# 4. ENV keys
grep "ANTHROPIC_API_KEY\|OPENROUTER_API_KEY" .env.example

# 5. Smoke test
node -e "
  import('./lib/ai/chain').then(m =>
    m.callChain('Reply with the word OK.', { maxTokens: 10 }).then(console.log)
  );
"
# Expected: { ok: true, text: 'OK', model: '...' }
```

## Discovery hooks (chỗ dễ sai)

| Pattern | Bài học |
|---------|---------|
| Timeout per-model thiếu | Default fetch không có timeout — phải `AbortSignal.timeout()` hoặc SDK option, nếu không 1 model treo = block cả chain |
| Retry quá nhiều ở tier 1 | Opus đắt, retry 3 lần có thể tốn 3x cost mà vẫn fail. `retries: 1` là sweet spot cho production |
| Cùng 1 prompt cho mọi tier | Gemini Flash tóm tắt khác Opus — output format không match expected. Build per-tier prompt nếu output schema khắt khe |
| Telemetry sync block | `await logAiCall()` trong hot path → log fail = call fail. Nên fire-and-forget hoặc queue background |
| Region block | Một số model OpenRouter block VN region. Test trước khi chọn fallback. Tarot dùng Gemini Flash (Google, OK ở VN) |
| Streaming vs non-streaming | Non-streaming dễ implement chain. Streaming chain phức tạp — chỉ làm khi UX yêu cầu (long-form output) |
| Token cost per tier | Opus 4.7 ~$15/1M out. Gemini Flash ~$0.30/1M. Tier fallback có thể save cost 50x — đo $/request qua telemetry |

## Env vars

```bash
ANTHROPIC_API_KEY=
OPENROUTER_API_KEY=
```

## Source

- DNA: `/Users/nguyenhuuanh/tarot/lib/ai/*` (extracted, business prompts stripped)
- Bài học: `/Users/nguyenhuuanh/tarot/docs/DISCOVERIES.md` SOUL-V98-TIMEOUT entry
- Cost notes từ tarot: Opus v98 đạt ~91% cost reduction so với v94 (prompt caching + structured output)
