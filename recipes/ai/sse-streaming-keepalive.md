# Recipe: SSE Streaming Keepalive (idle-disconnect guard)

> **Category:** ai
> **Stability:** stable (battle-tested ở tarot production)
> **Last verified:** 2026-07-23 (mined from tarot production, verified against tarot @cd16a86)

## Mục đích

Chống browser/proxy tự đóng kết nối SSE khi model im lặng quá lâu (ví dụ LLM đang ở "thinking"/reasoning phase, không emit `data:` chunk nào trong ~10s+). Wrap `ReadableStreamDefaultController` với ticker gửi comment-line `: ping\n\n` mỗi khi idle vượt ngưỡng — comment line không được SSE client parser hiểu là data event, chỉ giữ connection alive. Zero-dep, dùng `setInterval` polling (không dùng `setTimeout` reset-per-chunk — đơn giản hơn để reason về multiple-stop-call safety).

**Bài học gốc (tarot P222):** route reasoning models (OpenRouter Anthropic 1P thinking) có thể emit `delta.content=''` (rỗng) trong lúc suy luận — server-side code coi đó là "không có nội dung thật" nên skip enqueue, nhưng với client/proxy đang chờ byte tiếp theo, khoảng lặng ~10s+ đủ để timeout-disconnect giữa chừng response.

## Inputs (yêu cầu trước khi apply)

- [ ] Route SSE streaming đã có (`ReadableStream` + `TextEncoder`) — recipe này wrap thêm keepalive, không tự tạo stream
- [ ] Client-side SSE parser PHẢI skip mọi line không bắt đầu bằng `data: ` (comment lines theo SSE spec bắt đầu bằng `:` bị ignore bởi `EventSource` chuẩn — nếu dùng custom parser tự viết, verify nó cũng skip non-`data:` lines)
- [ ] Zero deps — chỉ Web Streams API built-in

## Outputs (sau khi apply)

- `src/lib/ai/sse-keepalive.ts` — `startSseKeepalive(controller, encoder, options)` → `{ markEnqueue, stop }`
- Không có ENV mới, không có DB thay đổi

## Steps

### 1. Keepalive util

```typescript
// src/lib/ai/sse-keepalive.ts
// SSE keepalive util — wraps a streaming controller with idle-detection ping.
// Replaces inline "lastEnqueueTime per-chunk polling" pattern with a reusable helper.
// V2 note: uses setInterval polling twice per intervalMs, NOT setTimeout-reset-per-chunk.

export interface SseKeepaliveOptions {
  intervalMs?: number // default 3000ms
}

export interface SseKeepaliveHandle {
  /** Call when a real `data:` chunk is enqueued. Resets idle timer. */
  markEnqueue: () => void
  /** Stop keepalive (call in finally block after stream complete/error). */
  stop: () => void
}

/**
 * Wrap a streaming controller with SSE keepalive comment lines (`: ping\n\n`).
 * When no real chunk is enqueued within `intervalMs`, emit a comment to
 * prevent SSE idle disconnect during silent phases (e.g. LLM reasoning/thinking
 * tokens the server intentionally doesn't forward to the client, ~10s+ gaps).
 *
 * Client parser MUST skip lines not prefixed with `data: ` (standard EventSource
 * behavior already does this — comment lines starting with `:` are spec-ignored;
 * verify if you wrote a custom SSE parser instead of using EventSource).
 *
 * Usage:
 *   const keepalive = startSseKeepalive(controller, encoder, { intervalMs: 3000 })
 *   try {
 *     for await (const chunk of stream) {
 *       if (chunkHasContent) {
 *         controller.enqueue(encoder.encode(`data: ${JSON.stringify(...)}\n\n`))
 *         keepalive.markEnqueue()
 *       }
 *     }
 *   } finally {
 *     keepalive.stop()
 *   }
 */
export function startSseKeepalive(
  controller: ReadableStreamDefaultController<Uint8Array>,
  encoder: TextEncoder,
  options: SseKeepaliveOptions = {}
): SseKeepaliveHandle {
  const intervalMs = options.intervalMs ?? 3000
  let lastEnqueueTime = Date.now()
  let stopped = false

  const ticker = setInterval(() => {
    if (stopped) return
    const idleMs = Date.now() - lastEnqueueTime
    if (idleMs >= intervalMs) {
      try {
        controller.enqueue(encoder.encode(`: ping\n\n`))
        lastEnqueueTime = Date.now()
      } catch {
        // Controller already closed - silent fail, ticker will be stopped next tick
      }
    }
  }, Math.floor(intervalMs / 2)) // poll twice per interval for accurate timing

  return {
    markEnqueue: () => {
      lastEnqueueTime = Date.now()
    },
    stop: () => {
      stopped = true
      clearInterval(ticker)
    },
  }
}
```

### 2. Wiring into an existing SSE route

```typescript
// src/app/api/some-stream-route/route.ts
import { startSseKeepalive } from '@/lib/ai/sse-keepalive';

export async function POST(req: Request) {
  const encoder = new TextEncoder();
  const stream = new ReadableStream<Uint8Array>({
    async start(controller) {
      const keepalive = startSseKeepalive(controller, encoder, { intervalMs: 3000 });
      try {
        for await (const chunk of modelStream) {
          if (chunk.hasContent) {
            controller.enqueue(encoder.encode(`data: ${JSON.stringify(chunk)}\n\n`));
            keepalive.markEnqueue();
          }
          // silent reasoning chunks: do nothing — keepalive ticker covers the gap
        }
      } finally {
        keepalive.stop(); // ALWAYS stop in finally — leaked ticker keeps the request alive server-side
        controller.close();
      }
    },
  });
  return new Response(stream, { headers: { 'Content-Type': 'text/event-stream' } });
}
```

## Verification anchors

```bash
grep -n "startSseKeepalive" src/lib/ai/sse-keepalive.ts       # 1. main export
grep -n "markEnqueue\|stop" src/lib/ai/sse-keepalive.ts       # 2. handle shape
grep -n ": ping" src/lib/ai/sse-keepalive.ts                  # 3. comment-line format, not data: event
grep -n "setInterval" src/lib/ai/sse-keepalive.ts             # 4. polling ticker, not setTimeout-per-chunk
grep -n "intervalMs ?? 3000" src/lib/ai/sse-keepalive.ts      # 5. default 3000ms
ls src/lib/ai/sse-keepalive.test.ts                            # 6. test ported alongside
```

## Discovery hooks (chỗ dễ sai)

| Pattern | Bài học |
|---------|---------|
| Idle-disconnect trong reasoning phase (tarot P222) | Model reasoning (ví dụ OpenRouter Anthropic 1P thinking) có thể emit content rỗng nhiều giây liên tục — nếu server chỉ enqueue khi có content thật, gap ~10s+ đủ để browser/proxy timeout-drop connection giữa chừng response. Keepalive ticker lấp gap này bằng comment-line vô hại. |
| Quên `keepalive.stop()` trong `finally` | Ticker (`setInterval`) không tự dừng khi stream đóng bình thường — nếu chỉ gọi `stop()` ở happy-path (không có `finally`), lỗi giữa chừng (throw trong `for await`) sẽ leak interval, giữ request handler sống vô thời hạn phía server. |
| Comment-line bị client parser coi là data event | `: ping\n\n` PHẢI bắt đầu bằng dấu `:` (SSE comment syntax) — nếu code gõ nhầm thành `data: ping\n\n`, client sẽ cố `JSON.parse` nó như 1 event thật và crash. Dùng `EventSource` chuẩn thì tự động skip comment lines; parser tự viết phải verify riêng. |
| `controller.enqueue` throw sau khi stream đã close | Ticker có thể fire NGAY SAU KHI controller đã đóng (race giữa `stop()` chưa kịp clear và tick đang chạy) — code PHẢI try/catch quanh `enqueue`, không để ticker throw ra ngoài phá vỡ response (verified: tarot test "controller close mid-stream → keepalive silently no-ops"). |
| `intervalMs` quá lớn (> proxy timeout) | Nếu reverse proxy/CDN có idle timeout ngắn hơn `intervalMs`, ping đến trễ hơn khi proxy đã cắt. Đặt `intervalMs` thấp hơn timeout ngắn nhất trong chain (thường Cloudflare free ~100s, nhưng số nhỏ hơn thường an toàn hơn — tarot dùng 3000ms). |
| Gọi `stop()` nhiều lần | Idempotency — code test xác nhận multiple `stop()` không throw (dùng `let stopped` guard + `clearInterval` vốn đã idempotent tự nhiên trong Node/Web). Không cần thêm guard riêng khi port. |

## Env vars

Không có — zero-dep.

## Migration / interop notes

- **Interop với `ai/multi-model-fallback`:** khi fallback chain đổi model giữa chừng stream (timeout tier-1 → tier-2), `keepalive.stop()` PHẢI chạy trước khi bắt đầu request tier tiếp theo, rồi `startSseKeepalive` lại mới cho tier mới — mỗi `ReadableStreamDefaultController` chỉ nên có 1 active ticker tại một thời điểm. Nếu multi-model-fallback wrap toàn bộ trong 1 stream duy nhất (không tạo controller mới per-tier), gọi `markEnqueue()` khi tier mới bắt đầu gửi content để tránh ping giả ngay lúc chuyển tier.
- Không phụ thuộc recipe khác để hoạt động độc lập — chỉ hữu ích khi đã có 1 SSE streaming route.

## Source

- DNA: `~/tarot/src/lib/ai/sse-keepalive.ts` (verified @cd16a86, extracted từ P222 hotfix inline logic) — port kèm test `~/tarot/src/lib/ai/sse-keepalive.test.ts` (source cho port)
- Bài học: tarot P222 (reasoning-phase idle disconnect), P235 (test suite)
- Docs: MDN Server-Sent Events — https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events

## Forge verification (2026-07-23, tarot @cd16a86)

Anchors chạy chống `~/tarot` @ `cd16a86` (`src/lib/ai/sse-keepalive.ts`):

| # | Anchor | Result |
|---|--------|--------|
| 1 | `startSseKeepalive` exported | ✅ HIT (line 38) |
| 2 | `markEnqueue`/`stop` handle shape | ✅ HIT (lines 61, 64) |
| 3 | `: ping` comment-line format | ✅ HIT (line 52) |
| 4 | `setInterval` polling ticker | ✅ HIT (line 47, fires at `intervalMs/2`) |
| 5 | default `intervalMs ?? 3000` | ✅ HIT (line 43) |
| 6 | test file exists | ✅ HIT — `src/lib/ai/sse-keepalive.test.ts`, 120 lines, 6 test cases incl. idle-emit, markEnqueue-reset, stop-idempotent, controller-close-silent-noop, default-3000ms |
