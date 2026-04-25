# Recipe: <Name>

> **Category:** infra / auth / payment / ai / observability / framework-starter
> **Stability:** experimental / stable / deprecated
> **Last verified:** YYYY-MM-DD

## Mục đích

[1-2 đoạn — recipe này giải bài toán gì? Tại sao chọn cách này thay vì alternatives?]

## Inputs (yêu cầu trước khi apply)

- [ ] Recipe X đã apply (nếu phụ thuộc)
- [ ] Có account / API key gì
- [ ] Yêu cầu tech (Node version, Python version, Docker, ...)

## Outputs (sau khi apply)

- File / folder mới được tạo
- DB migration nếu có
- ENV keys mới
- Endpoint / API surface mới

## Steps

### 1. [Step name]

```language
// code or config
```

[Giải thích NẾU có nuance không obvious]

### 2. ...

## Verification anchors

```bash
# Mỗi anchor PHẢI chạy được, exit 0 = pass
grep "..." path/to/file
ls path/to/file
curl -X POST http://localhost:.../...
```

## Discovery hooks (chỗ dễ sai)

| Pattern | Bài học |
|---------|---------|
| [Tên pitfall] | [Lý do dễ sai + cách tránh] |

## Env vars

```bash
# .env.example
KEY_1=
KEY_2=
```

## Migration / interop notes

[Nếu replace recipe khác, hoặc cần coexist với recipe khác — note ở đây]

## Source

- DNA: [path tới project gốc nếu extracted]
- Official docs: [link]
- Bài học từ DISCOVERIES: [reference nếu có]
