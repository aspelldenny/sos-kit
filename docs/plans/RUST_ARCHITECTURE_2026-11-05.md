# Kiến trúc Rust cho tool đi kèm agent — bản Kiến trúc sư 05/11/2026

> Trả lời cho kế hoạch `RUST_ALL_TOOLS_2026-11-04.md`. Đọc: code đợt 1a (`crates/sos-cli/src/{hook,sim_guard,status,kit}.rs`, `tests/adapters/run.sh`), bản cũ (`adapters/*/hook.py`, `scripts/*`, `templates/app/*`), và ngoài kit: `chan.py`, `kiem.py`, `win_heavy_guard.py`, `worker-guard.sh`, `block-unsafe-merge.sh`, `hub-audit.sh`, `harness-cong-cu.md`, `kiem-ke-tool-2026-11-04.md`. Không build, không chạy test. Chỗ chưa tự kiểm được ghi **[chưa kiểm]**.
>
> **Kết luận trước:** đây là thay đổi **NHỎ** so với kế hoạch. Giữ 5 đợt và cách chứng minh. Đổi năm điểm: (1) gộp 1c vào 1b, một bản phát hành và một lần soát; (2) kéo 2.4 `sos usage` lên ngay sau 1b (Sếp duyệt 05/11, mục 7); (3) đưa 2.1 (`test-watch` + `advise`) lên trước 1d, để hook agent hết Python trước khi làm máy luật; (4) thêm crate thư viện `sos-rules`; (5) `sos update --wiring` sửa chuỗi lệnh hook mà nó nhận ra, thay cho «chỉ nhắc». Q1 A, Q2 A, Q3 A đồng ý; hai đề xuất thêm cho Q2 (mục tiêu đo được cho Reel, tách đôi `hub`) chờ Sếp. Soát 05/11 (mục 8) làm đặc tả chặt hơn nhưng không đổi phán quyết.

## 1. Bố cục crate sau cùng

| Crate | Loại | Có gì | Phát cho app? |
|---|---|---|---|
| `crates/sos-cli` | binary `sos` | `install/update/check`, `gate`, `filter`, `role`, `hook`, và các lệnh mới `advise`, `bite`, `sim`, `sessions`, `predict`, `check-links`, `usage` | Có (binary duy nhất) |
| `crates/sos-gates` | lib | như nay: `.sos.toml` (`config`), `Report`, các cổng `text/docs/features/repo/local_secrets`, `glob` | qua `sos` |
| `crates/sos-rules` | lib, **mới** (1d) | máy luật khai báo: đọc TOML, bộ đọc lệnh shell (chuyển `lex()` từ `sim_guard.rs` sang đây), khớp, quyết định; không đọc stdin, không gọi process ngoài trừ qua trait `Runner` để test giả được | qua `sos`; crate Tổng đài không link, gọi `sos` |
| `crates/sos-dev` | binary `sos-dev`, **chỉ trong kit** (2.3) | `trust-gate`, `docs-check`, `install-hooks`; không vào `release.yml`, không vào `KIT_FILES` | Không |
| `~/code/tong-dai` (riêng tư, Q1) | binary `tong-dai` | `kiem`, `brief`, `don-rac`, sổ giao ca; gọi `sos sessions`, `sos hook rules` bằng process, không phụ thuộc Rust vào sos-kit | Không |

Mô-đun trong `sos-cli` (tách `hook.rs` hiện 372 dòng trước khi nó thành 1.000):

```
src/main.rs            clap: Cmd
src/kit.rs             install/update/check (+ --wiring, mục 3)
src/gate.rs, role.rs   như nay
src/hook/mod.rs        HookCmd, payload(), exit code; chỉ dịch payload ↔ lệnh gọi, không chính sách
src/hook/claude.rs     tool_input.file_path/notebook_path, tool_response{stdout,stderr}, agent_id
src/hook/codex.rs      apply_patch V4A, codex_block() (Windows: deny JSON + exit 0)
src/guards/env.rs      env_guard (từ hook.rs)
src/guards/sim.rs      sim_guard::check (bộ đọc lệnh chuyển sang sos-rules::shell)
src/status.rs          như nay
src/watch.rs           test-watch (2.1): streak theo runner, FAIL_RE/TEST_RE, .advise-state/
src/advise.rs          gọi claude/codex CLI, snapshot, evidence/advice/, calls.jsonl (2.1)
src/bite.rs            (2.2) worktree tạm, mutant, đọc kết quả từng test
src/sim.rs             (2.5) boot | settle | destination, dùng chung guards/sim
src/sessions.rs        (4.3, 4.4) đọc local_*.json của app Claude: list | count --max | idle
src/rules.rs           vỏ CLI của sos-rules: sos hook rules --file … [--lint|--explain|--cases]
```

Ranh giới thư viện/binary: thứ cần test đơn vị kỹ hoặc cần dùng từ chỗ khác (luật, bộ đọc shell, cấu hình, `Report`) nằm trong lib; thứ dính stdin/stdout/exit code nằm trong binary. `sos-gates` giữ tên để không đổi đường dẫn trong doc và `Cargo.toml` (luật 6 chỉ nói về bản đồ, nhưng đổi tên crate kéo theo CHANGELOG, CI, release không đáng).

Chính sách phụ thuộc: giữ như nay (`clap`, `anyhow`, `serde`, `serde_json`, `toml`, `regex`, `sha2`, `libc` trên unix). **Không** runtime async: mọi lệnh ngoài chạy bằng `std::process` + thread như `hook.rs::test_watch` đang làm; `advise` chỉ chờ một CLI, `bite` chạy tuần tự. Thời gian: `std::time` + tự đọc chuỗi ISO cho `lastActivityAt` (chrono có trong `[workspace.dependencies]` nhưng chưa crate nào dùng [chưa kiểm]; thêm chỉ khi cần múi giờ). Không thêm crate HTTP: `advise` gọi CLI, không gọi API. Mỗi dep mới ghi một dòng lý do trong CHANGELOG, vì mọi dep là bề mặt tin cậy của binary phát cho app.

## 2. Máy luật khai báo (đợt 1d) — `sos hook rules`

**Thay cho:** `chan.py` (vault), `win_heavy_guard.py` (BOOK), `worker-guard.sh` (soulsign-marketing), phần khớp lệnh của `block-unsafe-merge.sh` (tarot), `hub-audit.sh` (toàn cục). Năm script này có đúng bốn kiểu điều kiện: (a) tên tool + chương trình/đối số trong lệnh shell, (b) đường dẫn file trong/ngoài một gốc hoặc khớp glob, (c) trạng thái máy (file marker, máy Win có lên không), (d) lệnh kiểm chạy ngoài có đầu ra. Máy luật chỉ cần đủ bốn kiểu đó, không hơn (luật 2 của kit).

**Gọi:** `sos hook claude rules --file F [--file G …]` và `sos hook codex rules --file F`. Vỏ `claude|codex` chỉ để lấy đường dẫn và lệnh từ payload đúng dạng (Codex: `apply_patch`, Claude: `file_path`/`notebook_path`); payload trên stdin, một hook cho mọi `matcher` (`Bash|Edit|Write|MultiEdit|NotebookEdit|Read|Glob|PostToolUse …`), máy luật tự nhìn `tool_name` và `hook_event_name`. Không gộp vào `pre-edit`/`pre-bash` qua `.sos.toml` ở bản đầu: vault và hook toàn cục không có `.sos.toml`; làm sau nếu một repo thật cần.

**Định dạng (TOML, `deny_unknown_fields` như `.sos.toml`):**

```toml
version = 1

root = "~/VibeNotes"                  # gốc cho đường dẫn tương đối và file marker; không có thì là thư mục cwd của hook

[[rule]]
id = "vault-no-infra"
event = "pre"                          # pre (PreToolUse, mặc định) | post (PostToolUse)
tools = ["Bash"]                       # tool_name; Codex: Bash, apply_patch
command.program = ["ssh", "scp", "sftp", "rsync", "sudo", "iptables", "nft", "systemctl", "docker", "kubectl"]
decision = "block"
message = "[Cổng Tổng đài] Chặn: máy chủ / hạ tầng. Session trong VibeNotes chỉ đọc, ghi bảng và chuyển việc …"

[[rule]]                               # chan.py nhóm 2: build / cài / chạy dự án
id = "vault-no-build"
tools = ["Bash"]
command.program = ["xcodebuild", "fastlane", "make", "pnpm", "npm", "yarn", "cargo", "pip", "pip3", "uv"]
decision = "block"
message = "[Cổng Tổng đài] Chặn: build / cài đặt / chạy dự án …"

[[rule]]
id = "vault-no-simctl"
tools = ["Bash"]
command.program = ["xcrun"]
command.args = [{ pos = 0, eq = "simctl" }]          # đối số có cấu trúc: vị trí và giá trị, không nối chuỗi
decision = "block"
message = "[Cổng Tổng đài] Chặn: build / cài đặt / chạy dự án …"

[[rule]]
id = "vault-git-other-repo"
tools = ["Bash"]
command.program = ["git"]
command.args = [{ pos = 0, eq = "-C" },
                { pos = 1, path = { outside = ["~/VibeNotes"] } },     # đối số 1 là đường dẫn, chuẩn hoá như paths.*
                { pos = 2, one_of = ["add", "commit", "push", "reset", "checkout", "switch", "merge", "rebase", "restore", "stash", "clean"] }]
decision = "block"
message = "[Cổng Tổng đài] Chặn: git ghi vào repo khác ({arg.1}) …"

[[rule]]
id = "vault-edit-outside"
tools = ["Edit", "Write", "MultiEdit", "NotebookEdit", "apply_patch"]
paths.outside = ["~/VibeNotes", "~/.claude/projects"]
decision = "block"
message = "[Cổng Tổng đài] Chặn: sửa file ngoài vault ({path}) …"

[[rule]]                               # win_heavy_guard: ngoại lệ có phạm vi — chỉ miễn lệnh đơn `ssh win …`
id = "book-ssh-win-ok"
tools = ["Bash"]
command.program = ["ssh"]
command.args = [{ first_positional = "win" }]        # bỏ qua cờ `-o x`, `-p n`; host phải đúng là `win`
decision = "allow"
scope = "rule:book-heavy-on-mac"      # chỉ miễn luật này, chỉ cho lệnh đơn này; các lệnh đơn khác vẫn xét

[[rule]]
id = "book-heavy-on-mac"
tools = ["Bash"]
command.program = ["ocrmypdf", "tesseract", "pdftoppm", "pdftocairo", "ebook-convert", "gs", "magick"]
command.or = [{ program = ["hub"], args = [{ pos = 0, eq = "ocr" }] },
              { program = ["python", "python3"], args = [{ any = { basename_regex = '^(inspect_sources|qc_books|check_books)\.py$' } }] }]
command.not_raw = '^\s*MAC_FALLBACK=1\b'            # marker cũ của script; `raw` áp trên chuỗi gốc, chỉ dùng cho marker
check = { program = "ssh", args = ["-o", "ConnectTimeout=3", "-o", "BatchMode=yes", "win", "true"], timeout_secs = 6, fires_on = "success", on_error = "allow" }
decision = "block"
message = "Chặn: lệnh nặng đang chạy trên Mac trong khi máy Win đang bật …"

[[rule]]                               # worker-guard: điều kiện theo file trạng thái; thứ tự nguồn giữ như script
id = "mkt-no-canon-for-tho"
tools = ["Read", "Glob"]
when.role = { sources = [{ file = ".mkt-state/role", trim = true }, { file_exists = ".mkt-state/worker-active", value = "mkt-worker" }],
              one_of = ["tho", "data-analyst", "mkt-worker"] }      # nguồn đầu có file thì quyết định; file không đọc được → lỗi, không phải «không khớp»
paths.glob = ["**/SOUL.md", "**/PRODUCT.md", "**/NARRATIVE.md", "**/CHARACTER.md", "**/CHARACTER_*.md", "**/VOICE_SACH_CO.md", "**/tickets/TICKET_TEMPLATE.md"]
decision = "block"
message = "Vai «{when.value}» không đọc canon: {path} …"

[[rule]]                               # hub-audit: hook sau khi sửa, chạy lệnh kiểm, chặn theo đầu ra
id = "hub-fabricated-quote"
event = "post"
tools = ["Edit", "Write", "MultiEdit"]
paths.glob = ["~/Downloads/Documents/BOOK/**/*.{md,markdown}", "**/docs/research/hub-*/**/*.{md,markdown}"]
paths.exists = true
paths.content = '✔|✘|\[[^]]*(loc |p\.[0-9]|pp\.|tr\.|ch\.[0-9]|chapter |hồi |quyển )'
check = { program = "~/.local/bin/hub", args = ["audit", "{path}", "--quiet"], timeout_secs = 60,
          fires_on = { output = '(?m)^\s*✘ FABRICATED' }, on_error = "allow" }   # on_error = allow là hành vi cũ («lỗi lệnh → không chặn»), ghi rõ vì nó che lỗi thiếu hub
decision = "block"
message = "hub audit: câu trích BỊA ở {path}:\n{check.output}"
```

**Đơn vị xét: từng hành động, không phải cả payload.** Một payload Bash được tách thành các *lệnh đơn*; một payload sửa file thành các *đường dẫn* (Codex `apply_patch`: mọi header `Add/Update/Delete File` **và** `Move to`, cả nguồn và đích). Mỗi hành động được đưa qua **toàn bộ** bộ luật; payload bị chặn khi **bất kỳ** hành động nào bị chặn. Lệnh `git status; ssh win true` → `git status` qua, `ssh win true` xét riêng. Patch sửa `10_Projects/x.md` và `/etc/hosts` → chặn vì đường dẫn thứ hai. Patch `Move to` đưa file từ trong vault ra ngoài → chặn (đích ngoài).

**Mô hình khớp.** Một luật khớp một hành động khi *mọi* điều kiện có mặt đều đúng; luật không có `tools` khớp mọi tool.
- `command.*` chỉ áp cho tool có `tool_input.command` là chuỗi shell (Bash). Bộ đọc shell lấy từ `sim_guard` (quote, comment, `$(…)`, backtick, `sh -c`, nối dòng) nhưng **khác hai chỗ**: (1) wrapper **không bị bỏ** — `sudo cat /etc/hosts` là lệnh đơn có `program = sudo`, và `sim_guard` giữ cách bỏ wrapper của riêng nó khi tìm `xcodebuild` (hai hàm, một lexer); gán biến đứng trước (`A=1 cmd`) được bỏ nhưng nhớ lại cho `raw`; (2) heredoc: thân heredoc **đưa vào shell** (`sh <<EOF`, `bash <<'EOF'`, `zsh`, `dash`, hoặc qua `-s`) được lex lại thành lệnh đơn; thân heredoc đưa vào chương trình khác là dữ liệu. Cú pháp bộ đọc không hiểu (quote không đóng, `eval`, `$VAR` ở vị trí chương trình, `source`/`.` file) → hành động «không phân tích được» → **chặn** với lời «viết lệnh đơn giản hơn»; không bao giờ coi phần chưa phân tích là an toàn. `program` so với basename của từ đầu (và lệnh trong `$(…)`, trong heredoc-shell). `args` là **mảng điều kiện có cấu trúc** trên argv đã bỏ quote: `{pos, eq|one_of|regex|path}`, `{first_positional}`, `{any}`; không có dạng regex trên chuỗi nối, nên `git -C "/tmp/my repo" commit` giữ đúng ranh giới đối số. `raw`/`not_raw`: regex trên chuỗi gốc, chỉ cho marker (`MAC_FALLBACK=1`, `[security-review-skip:…]`); phạm vi của marker là luật chứa nó, không miễn cả payload.
- `paths.*`: chuẩn hoá theo thành phần: `~` mở rộng; tương đối → nối với `root` của file luật; bỏ `.`/`..` **sau** khi giải symlink của **ancestor tồn tại dài nhất** (file mới `~/VibeNotes/link/new.md` với `link` → ngoài vault bị bắt như `os.path.realpath`); containment so từng thành phần, không so tiền tố chuỗi (`~/VibeNotes2` không nằm trong `~/VibeNotes`). Hoa/thường: **không suy từ OS**; mặc định phân biệt, `paths.case_insensitive = true` khi file luật muốn (volume macOS có thể phân biệt). NUL hay path rỗng trong header → hành động không phân tích được → chặn. `glob` dùng `sos_gates::glob`; `exists` yêu cầu file có thật; `content` đọc file — lỗi đọc là **lỗi** (xem «Đóng khi hỏng»), không phải «không khớp».
- `when.*` (`role`, `file_exists`, `env`): trạng thái trên máy, đọc một lần trước khi khớp; file nêu trong `when` mà không đọc được (quyền, mã hoá) → lỗi, không phải «không khớp»; file không tồn tại là trạng thái hợp lệ («không có marker»).
- `check`: `program` + `args` **có cấu trúc**; `{path}`, `{arg.N}`, `{check.output}` được thay **vào từng đối số sau khi đã parse cấu hình**, nên tên file có khoảng trắng, quote, `;`, `$()` hay bắt đầu bằng `-` vẫn là đúng một đối số (luật nào cần `--` thì tự ghi). **Không có `run = "chuỗi"`, không nội suy vào `sh -c`**: lint từ chối `program = "sh"`/`bash` với `-c` chứa `{…}`. Chạy qua `std::process` không shell, env tối thiểu, `timeout_secs` bắt buộc, chỉ chạy khi các điều kiện khác đã khớp (ssh 3 giây không chạy cho mọi lệnh). `fires_on = "success" | "failure" | { output = regex }` (regex kiểu Rust; `(?m)` khi cần theo dòng); hết giờ, không chạy được, hoặc chương trình không tồn tại → `on_error` (`block` mặc định; `allow` phải ghi rõ và `--lint` nhắc một dòng cảnh báo).

**Quyết định và thông điệp.** `decision = "allow" | "block" | "ask"`. Với mỗi hành động: duyệt luật theo thứ tự file (các `--file` nối theo thứ tự dòng lệnh); luật `allow` **phải có `scope`** (`rule:<id>` hoặc `group:<tên>`) và chỉ miễn hành động đó khỏi các luật trong phạm vi ấy, không dừng duyệt; luật `block` đầu tiên khớp (và không bị miễn) → hành động bị chặn. Payload bị chặn khi có hành động bị chặn; thông điệp gom theo hành động. Không hành động nào bị chặn → exit 0. `block` → stderr, exit 2; Codex trên Windows → deny JSON + exit 0 như `codex_block()`. `ask`: fixture và code của kit chỉ cho thấy `permissionDecision: "deny"`; chưa có bằng chứng agent nhận `ask` nên bản đầu **từ chối file luật có `ask`** («chưa hỗ trợ»), không lặng lẽ coi là block [chưa kiểm]. Thông điệp thay `{path}`, `{program}`, `{arg.N}`, `{check.output}`; luật `block` thiếu `message` là lỗi file (luật 1 của kit).

**Đóng khi hỏng.** Ba lớp, mọi lớp đều chặn:
- *Payload*: không phải JSON, không phải object → chặn (như 1a). JSON hợp lệ nhưng tool được bảo vệ có **schema sai** (`tool_input.command` không phải chuỗi, `apply_patch` không rút được header nào, `file_path` không phải chuỗi) → chặn «không đọc được đầu vào của <tool>»; chỉ tool **không có luật nào nhắc tới** mới được qua khi thiếu trường.
- *File luật*: `--file` **bắt buộc** cho hook đã cài (mẫu settings luôn ghi `--file`); file nêu mà không có, TOML sai, khóa lạ, regex sai, `allow` thiếu `scope`, `check` thiếu `timeout_secs` → chặn mọi lệnh với «sửa file luật: <lỗi>; `sos hook rules --file F --lint`». «Chưa có luật» chỉ tồn tại ở dạng `sos hook rules --lint`/`--cases` chạy tay không `--file`; không có đường «thiếu file → cho qua». Repo chứa file luật đặt `--lint` vào pre-commit để không tự khoá.
- *Khi khớp*: lỗi đọc marker/`content`, lỗi chuẩn hoá path (ancestor không tồn tại ngay cả `root`) → chặn với lời «không kiểm được <điều kiện> của luật <id>». `check` hỏng → `on_error`.

**Chỗ để file luật.** Repo: `.sos-rules.toml` ở gốc (cạnh `.sos.toml`, file của dự án, `sos update` không đụng), settings của repo ghi `--file .sos-rules.toml` tường minh. Vault: `~/VibeNotes/10_Projects/tong-dai/chan.toml`, gọi từ `~/VibeNotes/.claude/settings.json` thay `chan.py`. Toàn cục: `~/.claude/sos-rules.toml` cho hub-audit. Mọi đường dẫn cá nhân nằm trong các file đó, ngoài kit (luật 3).

**Kiểm.** Ba lớp: (1) test đơn vị Rust trong `sos-rules` cho bộ đọc shell (wrapper giữ lại, heredoc-shell, cú pháp không hiểu → không phân tích được), khớp `program`/`args`/`paths` (chuẩn hoá symlink ancestor, `..`, containment theo thành phần, hoa/thường), thứ tự luật và `scope`, `Runner` giả cho `check` (đối số có khoảng trắng/quote/`;`/`$()`/bắt đầu bằng `-` phải tới chương trình nguyên một đối số, không có tác dụng phụ); (2) `sos hook rules --file F --cases cases.jsonl`: mỗi dòng `{payload, want: "allow"|"block", stderr_has}`, bộ chạy nằm trong `sos` nên repo nào cũng kiểm được không cần bash. **Bắt buộc có** trong mỗi bộ cases: lệnh ghép (`git status; ssh win true`), patch nhiều file (một trong, một ngoài), patch `Move to` ra ngoài, nhiều `--file`, payload schema sai, file luật thiếu. **Quy định:** mỗi ca phải có **kết quả bản cũ ghi sẵn** (chạy script Python/Bash cũ với cùng payload, ghi `want` từ đó) trước khi dịch luật; ca nào bản mới cố ý khác bản cũ ghi `note` và lý do; (3) `--explain` in từng hành động, luật nào khớp, luật nào miễn và vì sao. Bộ cases của kit ở `tests/rules/` dùng payload thật trong `tests/adapters/fixtures/`; bộ cases của vault nằm cạnh `chan.toml`.

**Bảng bảo toàn hành vi năm script** (ca Người soát nêu 05/11, mỗi ca vào `cases.jsonl` của repo đó với kết quả bản cũ trước):

| Script | Ca phải giữ |
|---|---|
| `chan.py` | `sudo cat /etc/hosts` chặn (wrapper không bị bỏ); `npm install`, `cargo build`, `xcrun simctl list` chặn (luật `vault-no-build`, `vault-no-simctl`); `git -C "/tmp/my repo" commit` chặn, `git -C ~/VibeNotes commit` qua; `sh <<'EOF'\nssh win true\nEOF` chặn (heredoc-shell); sửa `~/.claude/projects/...` qua; `~/VibeNotes/link/new.md` với `link` → ngoài chặn |
| `win_heavy_guard.py` | `hub ocr x.pdf`, `python3 inspect_sources.py`, `qc_books.py`, `check_books.py` chặn khi Win lên; `ssh win 'tesseract …'` qua; **`ssh other true; tesseract a.png` chặn** (ngoại lệ chỉ cho lệnh đơn `ssh win`); `MAC_FALLBACK=1 tesseract …` qua; Win không lên → qua (`on_error = allow` là hành vi cũ) |
| `worker-guard.sh` | chỉ có `.mkt-state/worker-active` (không có `role`) → vẫn chặn như `mkt-worker`; `role` chứa `tho\n` → trim rồi khớp; `role` = `marketing` → qua; `CHARACTER_CHI_HA.md` chặn; `Glob pattern` cũng xét |
| `block-unsafe-merge.sh` | helper `scripts/merge-check <N>` trong tarot **giữ đủ bốn nhánh**: pattern security surface (kể cả `.env` không phải `.env.example`), `gh pr diff` lỗi/rỗng → chặn, chưa có review → chặn, verdict **mới nhất** trong sentinel; marker `[security-review-skip:…]` chỉ miễn luật merge, không miễn lệnh khác trong cùng payload; `on_error = "block"` |
| `hub-audit.sh` | `.markdown` cũng xét; đầu ra có dòng thông tin trước `✘ FABRICATED` vẫn chặn (`(?m)`); file ngoài BOOK/hub-* qua; file không tồn tại qua; `hub` gọi bằng đường dẫn tuyệt đối trong file luật như script cũ (PATH của hook có thể thiếu `~/.local/bin`); `on_error = allow` ghi rõ là che lỗi thiếu `hub` |

Ngoài sức máy luật, để ở repo: toàn bộ logic `gh` của `block-unsafe-merge.sh` thành một helper nhỏ trong tarot mà luật gọi qua `check = { program = "scripts/merge-check", args = ["{arg.N}"], fires_on = "failure", on_error = "block" }`. Không nhét grammar «lấy comment, lấy dòng cuối» vào TOML.

## 3. Hook gọi binary và đổi chỗ gọi (đợt 1b)

**Nguyên tắc: không bỏ fail-closed để đổi lấy «hết Python».** Guard trước sửa file và trước lệnh shell phải chặn trong mọi ca khởi chạy hỏng: `sos` thiếu, `sos` cũ (0.3.13 không biết `hook`), `sos` không chạy được, npm shim có mà binary chưa tải. Một guard **không bao giờ tự tải binary**: `bin/sos-npm` hiện chỉ miễn tải cho `gate|filter` (dòng 61–69), `hook` phải vào cùng nhánh đó — thiếu binary thì shim in lỗi và thoát ≠ 0, launcher bên ngoài biến cái đó thành chặn.

**Launcher.** Mỗi agent/hệ có một *launcher* nhỏ, cố định trong mẫu, biến **mọi** kết quả không phải «sos chạy và trả 0» thành chặn đúng giao thức agent đó:
- Claude (mọi hệ; Claude chạy hook qua shell, mẫu hiện tại đã dùng `$CLAUDE_PROJECT_DIR`, chạy trên POD-agent qua Git Bash): 
  ```
  sos hook claude pre-edit --protocol 1; rc=$?; [ "$rc" = 0 ] || { [ "$rc" = 2 ] || echo "BLOCKED: SOS Kit hook could not run (sos missing, too old, or failed: exit $rc). Install/upgrade sos: https://github.com/aspelldenny/sos-kit#quick-start" >&2; exit 2; }
  ```
  Không `exec` (để còn xử lý mã thoát); 127 (thiếu), 1 (shim npm chưa tải, crash), 2 của clap (binary cũ) đều ra exit 2. `session-start`/`post-bash` gọi trần, không launcher (thiếu thì im).
- Codex macOS/Linux: cùng launcher.
- Codex Windows: shell chạy hook chưa rõ, deny phải là JSON + exit 0; một dòng shell không làm được trong cả cmd/PowerShell/bash. **Giữ dạng `python -c` của `codex_hooks_any_shell()`**, chỉ đổi thân: gọi `sos hook codex <event> --protocol 1` thay `hook.py`, mọi lỗi (không tìm thấy, mã ≠ 0, timeout) → `d()` deny JSON như hôm nay. Python ở đây là launcher 1 dòng, không phải adapter; bỏ nó chỉ khi có cách khác đã **thử thật trong Codex Windows** với bốn ca: `sos` mới, cũ, thiếu, không chạy được. Ca thử này là điều kiện phát hành mẫu 1b (Sếp hoặc máy Win qua `may-win chay`), nối tiếp việc mở từ 23/10.
- `sos hook … --protocol 1`: cờ khai phiên bản giao thức hook; binary mới nhận và chạy; binary cũ (0.3.13) từ chối cờ lạ bằng clap exit 2 → launcher chặn với lời «sos quá cũ». Khi giao thức đổi (tên event, dạng JSON) thì tăng số; binary chỉ chạy số nó biết. Đây là cách xử lý «binary cũ» mà kỷ luật thứ tự cài không bảo đảm được.
- Test launcher (`tests/hooks/launcher.sh`): PATH không có `sos`; `sos` là script thoát 1; `sos` là bản giả 0.3.13 (clap không biết `hook`); npm shim thật với `installed()` giả trả rỗng và tải thất bại — bốn ca đều phải cho exit 2 (Claude) / deny JSON (Codex Windows, chạy trong CI `windows-2022` qua cmd và PowerShell). Thêm: kiểm stdout **tách** stderr và parse **toàn bộ** stdout là đúng một JSON (điểm 8 còn mở của review 1a).

**Lệch phiên bản.** Kỷ luật thứ tự giữ nguyên (phát hành có `sos hook` → cài mọi máy → đổi settings), nhưng không còn là lớp bảo vệ duy nhất: `--protocol` + launcher chặn khi binary cũ. `sos hook claude session-start` trả exit 2 thì Claude làm gì chưa rõ [chưa kiểm] — vì thế session-start không có launcher. Từ đây **`sos hook <agent> <event>` là hợp đồng tương thích**: tên lệnh con, event và `--protocol` chỉ thêm, không đổi; đổi logic bên trong tự do.

**`sos update`/`sos check` với file của dự án.** Hôm nay chỉ nhắc; hậu quả thật đã có: 6 repo nâng 0.3.13 đều thiếu `pre-bash` và phải gộp tay (kiểm kê 04/11), 11 repo sắp phải đổi `hook.py` → `sos hook`. Đề nghị `sos update --wiring` (và `sos check` gợi ý lệnh đó): trong `.claude/settings.json` và `.codex/hooks.json` chỉ (a) thay những chuỗi `command` khớp **nguyên văn** một mẫu cũ của kit (`python3 "$CLAUDE_PROJECT_DIR/harness-lite/adapters/claude/hook.py" pre-edit`, biến thể `python`, biến thể `python -c` Windows) bằng chuỗi mới, (b) thêm event của kit còn thiếu (`pre-bash`) vào mảng tương ứng, (c) không đụng entry lạ (money-guard của POD-agent, hook riêng của tarot). Sửa **theo cấu trúc JSON** (event → matcher → hooks[].command), không thay chuỗi trên văn bản; giữ nguyên khóa lạ và metadata; chạy lặp không sinh bản sao; ghi qua file tạm rồi đổi tên; in diff trước khi ghi, có `--dry-run`. Quyền sở hữu không đổi: file vẫn của dự án, `update` thường không chạm; `--wiring` là hành động người gọi chủ động — lời hứa «never overwritten» trong README/`kit.rs` sửa thành «never overwritten by `sos update`; `--wiring` edits the kit's own hook commands in place, shown first». `sos check` đọc được cả hai dạng (`hook.py` cũ và `sos hook`) và cảnh báo dạng cũ: «app còn gọi hook.py».

**Khi nào bỏ adapter Python.** Bản N phát hành mẫu mới; bản N+1 rút `adapters/*/hook.py`, `env-guard.sh`, `sim-guard`, `status.sh` khỏi `KIT_FILES`. Trạng thái fleet (11 repo) chỉ là tín hiệu để chọn thời điểm, **không phải điều kiện an toàn**: sos-kit công khai, người ngoài có thể nhảy thẳng từ bản cũ lên N+1. Vì thế `sos update` kiểm **tại repo đang cập nhật**: trước khi xoá một file «dropped upstream», đọc `.claude/settings.json` và `.codex/hooks.json`; còn `command` nào tham chiếu file đó (hoặc file settings không parse được, tức không xác định được wiring) → **không xoá**, giữ file dưới nhãn mới `Act::KeepReferenced` («kept: .claude/settings.json still calls it — run `sos update --wiring`»), và `update` thoát ≠ 0 như với conflict. Ca test: nâng từ bản trước N thẳng lên N+1; settings có lệnh tuỳ biến chứa `hook.py`. Chi tiết `kit.rs` hiện tại: file bỏ upstream mà app đã sửa tay đi qua `Act::KeepModified` («keep (dropped upstream, but you edited it)»), **không** sinh `.sos-new` — đoạn trước của bản này ghi sai, sửa lại ở đây. Bản cũ chuyển vào `archive/` của kit (lịch sử công khai) chứ không vào `~/.local/share/tool-archive` — đó là chỗ cho tool ngoài kit. `tests/adapters/run.sh` lúc đó chỉ còn nhánh `SOS_HOOK_BIN`, đổi tên thành `tests/hooks/run.sh`.

**Gộp 1c vào 1b.** Vỏ git hook của app đã là 15 dòng gọi `sos gate all` rồi `exec *.local`. Chuyển logic đó thành `sos gate pre-commit|pre-push` (chạy gate rồi `.local`, xử lý «có file mà không chạy được» cho cả Windows không có exec bit) làm vỏ còn 3 dòng — đáng, nhưng chỉ vì nó cùng chạm `templates/app/hooks/` với 1b: một bản phát hành, một lần Người soát độc lập (luật 4), một lần `trust-gate rebaseline`. Tách riêng thì hai lần soát cho hai thay đổi nhỏ. Hook của chính kit (`hooks/pre-commit` chạy `cargo run`, trust gate, docs-check) **không** theo đường này: nó là dev tooling → `sos-dev` ở 2.3.

## 4. Code dính vault để đâu (Q1)

sos-kit công khai; vault chứa tên dự án, đường dẫn máy, sổ giao ca. Ranh giới: **chung = không biết gì về bảng `10_Projects/README.md`**.

| Phần | Vào `sos` (công khai) | Vào `~/code/tong-dai` (riêng) |
|---|---|---|
| Máy luật | `sos hook rules` | `chan.toml` + `cases.jsonl` (trong vault, không phải trong crate) |
| Đếm/canh session | `sos sessions list|count --max N|idle --than 24h` đọc `~/Library/Application Support/Claude/claude-code-sessions/**/local_*.json` (`lastActivityAt`, `isArchived`, `cwd`, `title` — đã kiểm trong harness-cong-cu), `--dir` cho máy khác, in JSON; không biết cột «Đầu mối» | `kiem` gọi `sos sessions idle --json`, ghép với cột Đầu mối của bảng, in mục ĐỨNG IM |
| Tối đa 3 session con (4.4) | luật trong `chan.toml`: `tools = ["mcp__ccd_session__start_session"]` (tên tool trong payload cần bắt một lần cho chắc [chưa kiểm]) + `check = { run = "sos sessions count --max 3", fires_on = "failure" }` | — |
| Tin ≤100 chữ (4.6) | `sos hook word-limit --max 100` (hook Stop; đếm chữ, không biết nội dung) | — |
| Bảng, mốc, mục đích, sổ giao ca (4.2, 4.5), brief (4.7), don-rac (2.9) | — | toàn bộ |

Crate riêng không `depend` Rust vào sos-kit (không git-dep, không publish): nó gọi `sos` qua process như mọi app, đúng luật 5 «CLI first», và sos-kit đổi nội bộ không làm nó vỡ. Cái giá của Q1-A: binary Mac phải build ở đâu đó, và giới hạn hiện hành của kế hoạch là **không build trên Mac 8 GB** — bản này giữ giới hạn đó. Cách làm trong giới hạn: repo GitHub riêng tư cho `tong-dai` với `release.yml` chép từ sos-kit (job `macos-14` build và đính binary; Actions cho repo riêng có 2.000 phút/tháng miễn phí), Mac chỉ tải binary về như với `sos`. Việc pre-commit của sos-kit đang `cargo run` trên Mac là chuyện có sẵn, không phải cớ để nới luật; nếu Sếp muốn nới thì đó là một quyết định riêng, ghi vào kế hoạch gốc.

## 5. Ý kiến về Q1–Q3

- **Q1 — A, đồng ý.** Lý do ở mục 4. Thêm: nếu để trong workspace công khai thì mỗi commit bảng/sổ thành lịch sử công khai không rút lại được; và `trust-gate` của kit sẽ phải canh thêm một bề mặt chạy tự động trên máy Sếp.
- **Q2 — đồng ý A (Reel và `hub` đổi sau cùng, dự án riêng). Hai điều dưới đây là ĐỀ XUẤT chờ Sếp, không phải quyết định:** (i) Reel: 407 test Python là chuẩn so sánh hiếm có; đổi thành test mức HTTP/CLI rồi chạy song song trên VPS là đường đúng; em *đề xuất* bắt đầu sau đợt 4 và ghi vào BACKLOG của Reel một mục tiêu đo được (RAM VPS, lỗi kiểu dữ liệu, deploy) để biết port xong có đạt không — động cơ Sếp chốt (đồng bộ, bảo trì) vẫn là lý do làm, mục tiêu đo chỉ là thước. (ii) `hub` (9.000 dòng, 40 file Python): em *đề xuất* tách đôi — Rust cho `hub audit|verify|search` (khớp chữ với kho), giữ Python cho dây chuyền OCR/convert/đọc PDF chạy qua `ssh win` vì thư viện Rust tương đương chưa chín [chưa kiểm từng thư viện]; port phần đó là viết lại công cụ, không phải đổi ngôn ngữ. Nếu Sếp chọn đổi toàn bộ, kiến trúc này không cản; brief cho Quản đốc BOOK ghi lựa chọn Sếp chọn.
- **Q3 — A, đồng ý.** `pre-commit.local` là keo gọi `xcodebuild`/`npm` của chính app; bản Rust vẫn gọi y lệnh đó, chậm hơn khi sửa (build) và Thợ của app đọc Bash 10 dòng dễ hơn một crate. Điều kiện giữ: `.local` không chứa logic quyết định (chỉ gọi lệnh và trả exit code); logic như INV của tarot (2.7) đi vào luật hoặc Rust trong repo đó.

## 6. Vấn đề thấy ở đợt 1a và thứ tự đợt

Đợt 1a đúng hướng (dịch 1-1, test hai bản cùng bộ, khác biệt có chủ ý ghi rõ). Mấy điểm cần sửa hoặc canh:

1. **`hook.rs` gộp dịch payload và chính sách.** Python tách adapter/`scripts/` (triage §2.3a); bản Rust gộp vào một file. Chưa sai, nhưng 1d và 2.1 sẽ thêm vào đúng file này. Tách như mục 1 trước khi làm 2.1 (một commit cơ học, test hiện có đủ bảo vệ).
2. **`post-bash` vẫn spawn `python3`** — nên Windows vẫn cần Python, `sos check` vẫn kiểm interpreter. Đây là lý do chính để **2.1 đi trước 1d**: xong 2.1, hook agent không còn Python, `hook_python()`/`hook_interpreters()` trong `kit.rs` xóa được, và README bỏ «needs python3» cho hook (bite, trust-gate còn cần đến 2.2/2.3).
3. **`sim_guard` dùng `ps -axo`**: khi không có `ps` (Windows) `running_tests()` trả rỗng, tức coi như không có xcodebuild đang chạy (đã đọc, dòng 374–377). Chấp nhận được vì Windows không có simulator, nhưng chưa có test nào ghim hành vi đó; thêm một ca.
4. **Mã thoát chưa thống nhất**: `sos gate` 1 = chặn, 2 = cấu hình sai; `sos hook` 2 = chặn (theo Claude). Giữ cả hai nhưng ghi vào `main.rs` doc: `gate` nói với git/người, `hook` nói với agent; `sos hook rules --lint` theo kiểu `gate` (1 lỗi luật) để pre-commit dùng.
5. **Hợp đồng CLI của `sos hook`** chưa được ghi là «chỉ thêm, không đổi» — thêm vào `CLAUDE.md` của kit một dòng khi 1b vào (tên lệnh nằm trong file của 11 dự án).
6. **`run.sh` kiểm Rust bằng cách so với Python**: khi Python bị bỏ (N+1), các ca «same status» byte-đối-byte mất chuẩn so. Trước khi bỏ, chốt đầu ra mong đợi thành fixture tĩnh (`tests/hooks/expected/status-*.txt`).

**Thứ tự đợt đề nghị:** 1a (xong) → **1b+1c** (một PR, một bản phát hành N, cài Mac/Win, rồi đổi settings từng repo bằng `sos update --wiring`) → **2.4 `sos usage`** (Sếp duyệt 05/11: đo trước khi chỉnh model/effort/fork; mục 7) → **2.1** (`watch`, `advise`; tách `hook/`) → **1d** (`sos-rules`, vault `chan.toml` trước vì có chủ sở hữu rõ và cases dễ viết; rồi BOOK, soulsign-marketing, tarot, hub-audit) → 2.2 `bite` → 2.3 `sos-dev` → 2.5 `sim` → đợt 3 → đợt 4 (`sessions`, `word-limit` trong sos; crate tong-dai) → đợt 5 theo điều kiện ở Q2. Các mục 2.4, 2.6, 3.4 (bỏ/giữ) làm xen bất cứ lúc nào vì không chạm code chung.

**Phán quyết (sau soát 05/11): vẫn là thay đổi NHỎ so với kế hoạch.** Không bỏ đợt, không đổi Q1–Q3 (Q2 có hai đề xuất chờ Sếp). Năm chỉnh: gộp 1c vào 1b, kéo 2.4 `sos usage` lên ngay sau 1b (lệnh của Sếp), đưa 2.1 lên trước 1d, thêm crate `sos-rules`, thêm `sos update --wiring`. Chín điểm soát làm đặc tả **chặt hơn** (launcher fail-closed, đơn vị xét theo hành động, `check` có cấu trúc, bộ đọc shell giữ wrapper, path theo thành phần, schema payload, bỏ adapter kiểm tại repo) nhưng không đổi crate, đợt hay chỗ gọi; cái giá là 1b giữ một launcher `python -c` cho Codex Windows cho tới khi thử thật, và 1d lớn hơn ước lượng ban đầu (bộ đọc shell riêng cho luật, không dùng nguyên `sim_guard`).

## 7. `sos usage` — đo token theo vai trước khi chỉnh (2.4, ngay sau 1b)

**Vì sao trước:** Sếp đang cân bốn cách tiết kiệm (model theo vai, ít fork, subagent trả ngắn, hạ effort thợ). Số đếm thô đã có (`querySource`: sdk ≈33,5k lượt, `agent:custom:tho` 12,7k, worker 2,3k…) nhưng *lượt gọi* không phải *token*: một lượt của Quản đốc với 45k cache read khác một lượt thợ. Sửa đúng chỗ cần bảng token theo vai × model. `tally/tools/agent-usage.py` (99 dòng) đã làm một phần cho một repo; `sos usage` làm cho mọi repo và thêm ba góc nhìn (fork, kết quả subagent, ước tính tiền).

**Trường thật trong transcript Claude** (đọc 05/11 trên `~/.claude/projects/**`, 1.023 file, 13 GB; Codex ở dưới):

| Trường | Ở bản ghi | Dùng làm gì |
|---|---|---|
| `message.usage.{input_tokens, cache_creation_input_tokens, cache_read_input_tokens, output_tokens}`, `output_tokens_details.thinking_tokens`, `cache_creation.{ephemeral_5m,ephemeral_1h}_input_tokens` | `type = "assistant"` | con số cộng. `thinking_tokens` không có ở mọi bản (file 09/2026 thiếu) → cột «thinking» ghi `—` khi vắng, không ghi 0 |
| `message.id`, `message.model`, `requestRef`, `requestedModel`, `effort`, `timestamp`, `cwd`, `entrypoint`, `isSidechain`, `agentId` (chỉ file subagent) | `assistant` | khử trùng: một tin stream thành nhiều dòng cùng `message.id`, **giữ dòng có `output_tokens` lớn nhất** (đúng cách của agent-usage.py; đã thấy 75 dòng → 41 tin). `cwd` = dự án; `effort` để so hạ effort |
| `querySource` (`sdk`, `agent:custom:<tên>`, `agent:builtin:<tên>`), `params.model`, `id`, `agentId` | `type = "api-request"` (một dòng mỗi lần gọi API; **không** nằm trên `assistant`) | vai. Nối với `assistant` qua `assistant.requestRef == api-request.id` (75/75 nối được trong file thử; `requestId` thì không). Trong file subagent `querySource` cũng nằm ở `api-request` của chính file đó |
| `agentType`, `description`, `spawnDepth`, `model`, `effort`, `parentAgentId`, `requestShape` (`background`/`foreground`), `worktreePath` | `<session>/subagents/agent-<id>.meta.json` | vai dự phòng khi file subagent không có `api-request` (bản 09/2026 không có `querySource`); `spawnDepth` để thấy subagent của subagent |

**Vai (Claude).** Vai = `querySource` nếu có; không thì `agentType` của meta.json **chuẩn hoá** về cùng không gian tên (`worker` → `agent:custom:worker`, `general-purpose`/`Explore` → `agent:builtin:…`, file chính → `sdk`); không có cả hai → `unknown`. Hai nguồn cho cùng một lần gọi so **sau** chuẩn hoá; lệch thì `--explain` in ra, không đoán.

**Một request tính một lần — chính sách trùng.** Khóa khử trùng là `message.id` (dự phòng `requestRef`, rồi `requestId`); map khóa → `(output_tokens, usage, model, role)` giữ **theo phiên (`sessionId`), không theo file**: lịch sử resume ghi lại trong cùng file hay sao chép sang file khác của cùng phiên đều chỉ tính một lần (Người soát chưa thấy trùng xuyên file trong 12 file kiểm; chính sách này để tổng không đổi theo cách app lưu). `isSidechain = true` trong file chính: **bỏ** nếu tồn tại file `subagents/agent-<agentId>.jsonl` (file subagent là chủ sở hữu), **giữ và gắn vai từ meta/`unknown`** nếu không có file đó — khác agent-usage.py (bỏ hết sidechain), ghi là khác biệt có chủ ý. `model = "<synthetic>"` hoặc không có `usage` → bỏ, đếm vào `meta.skipped.synthetic`. Compaction summary không là một lần gọi mới; usage thật của lần gọi tạo summary vẫn tính như mọi `assistant`. Khóa thiếu hoàn toàn → tính theo số dòng, đếm vào `meta.skipped.no_id`.

**Fork và kết quả subagent** — chỉ báo cái đo được, đặt tên đúng đại lượng:
- *Fork*: trong 80 file gần nhất không thấy `subagent_type: "fork"` ở tool call `Agent`, meta.json không có khóa fork [chưa kiểm toàn bộ 13 GB]. **Không** có cột «chi phí fork» và **không** dùng ngưỡng 80% (subagent độc lập có prompt lớn cũng vượt; fork không cache lại không vượt; cha gần nhất theo giờ chưa chắc là cha spawn). Quan hệ cha–con lấy từ `meta.toolUseId` ↔ `tool_use.id` của tool call `Agent` trong file cha, và `meta.parentAgentId`; không có hai trường đó → `parent = unknown`. Chỉ khi có trường fork thật (hoặc Sếp chỉ ra cách app ghi fork) mới có cột fork. Trong lúc đó `--explain` in một **heuristic có nhãn** `large_initial_context`: subagent có lần gọi đầu với `input + cache_creation + cache_read` lớn hơn X (mặc định 50k), kèm số của cha spawn nếu nối được — là gợi ý đi xem, không phải số tiền.
- *`subagent_result_text_chars`*: số ký tự **văn bản** trong `tool_result.content` (chuỗi, hoặc nối các block `text` của mảng) của **kết quả cuối** của tool call `Agent`, nối theo `tool_use_id` xuyên mọi bản ghi `user` của file cha (không phải «bản ghi kế»); bản ghi có `toolUseResult.isAsync = true` là **spawn acknowledgment**, đếm riêng (`spawn_acks`), kết quả cuối của nó là `tool_result` sau cùng `tool_use_id`; lỗi (`is_error`) đếm riêng. Nó đo **lượng văn bản trả về một lần**, không đo token cha đọc lại qua các request, không trừ phần cache hay bị compaction cắt; token ước ≈ ký tự/4 chỉ nằm trong `estimates`. Cộng theo vai của subagent (từ `subagent_type` của tool call). Đây là số để kiểm ý «subagent trả ≈15 dòng, chi tiết ra file»; đo chi phí đọc lại cần phương pháp khác, chưa làm.
- Không chép `agent_summary`/`CLAUDE_CODE_AGENT_PROGRESS_SUMMARIES` (Desktop tắt, transcript không có).

**Codex** (`~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`, 175 file, 982 MB). Số đo của Người soát 05/11 quyết định cách cộng: rollout 27/10 có 17 `token_count` cho 16 response, cộng `last_token_usage` dư **78.217** token; rollout 29/10 có compaction **246.204** token nằm ở `token_usage_record` (và lặp trong `compacted.latest_token_usage_record`) nhưng **không** nằm trong `token_count` cuối. Vì thế:
- **Nguồn chính:** `token_usage_record.payload.usage`, khử trùng theo `response_id` (map theo thread, xóa khi hết file); compaction tính **đúng một lần** từ bản ghi gốc, bản sao trong `compacted.*` bỏ qua.
- **Fallback** khi phiên không có `token_usage_record` nào: hiệu của `total_token_usage` cộng dồn giữa hai `token_count` liên tiếp (âm hoặc reset về 0 → coi là counter reset, lấy giá trị mới, đếm `meta.codex.counter_resets`); phiên có cả hai nguồn **không** cộng cả hai; `--explain` in nguồn dùng cho từng phiên.
- Model: `turn_context.payload.model`, gán cho các response của turn đó (đổi model giữa phiên thì theo turn), không tìm một chuỗi model cho cả file.
- Vai/loại phiên: `originator` chỉ là **nguồn chạy** (phiên chính và subagent đều có `originator = "Codex Desktop"`); loại phiên từ `session_meta.payload.source`: chuỗi (`exec`, …) hay object có `subagent.thread_spawn.{parent_thread_id, depth, agent_path, agent_role}`; vai = `agent_role` khi có, `unknown` khi null, `main` khi không phải subagent; `parent_thread_id` dựng cha–con. Bảng Codex in riêng (`cached_input` ≈ cache read; `cache_write_input_tokens`; `reasoning_output_tokens` là phần của output, không cộng thêm).

**Lệnh:**

```
sos usage [--since 2026-11-01] [--until …] [--project <đường dẫn|tên>] [--by role|model|project|day|session]
          [--agent claude|codex|all] [--prices <file.toml>] [--json] [--explain] [--dir <projects dir>]
```

Mặc định: `--since` 30 ngày, `--by role`, mọi dự án, Claude. Bảng (cột căn phải, số có dấu phẩy nghìn, như agent-usage.py để so mắt):

```
role                model              calls   input  cache write   cache read   output  thinking  result_chars   est $    share
sdk (main)          claude-opus-5-5    1,204   2,410   38,120,440  412,009,120  901,220   120,400     1,802,300  1,234.5   61%
agent:custom:tho    claude-sonnet-5-5  6,410  …
unknown             —                  …
total
```

`--json`: `{meta: {since, until, files, sessions, dedup_dropped, skipped: {synthetic, no_id, oversize_lines, bad_json}, coverage_warnings: […]}, rows: […], estimates: {subagent_result_tokens_est, large_initial_context: […], prices_version}}`; `rows` chỉ có số đếm được; mọi ước lượng và heuristic nằm trong `estimates`; thiếu dữ liệu (dòng bỏ, phiên không có nguồn usage) nằm trong `coverage_warnings` và in dưới bảng.

**Ước tính tiền và đề xuất.** Bảng giá là file TOML `[prices."claude-opus-5-5"] input = …, cache_write_5m = …, cache_write_1h = …, cache_read = …, output = …` (USD/1M token). Nguồn mặc định nhúng trong binary với ngày lấy giá (`prices_version`), `--prices` hoặc `~/.config/sos/prices.toml` ghi đè; `sos usage` không tải giá từ mạng. Giá đổi và Sếp dùng gói thuê bao chứ không trả theo token, nên cột `est $` là **đơn vị so sánh**, không phải hóa đơn — dòng chú thích dưới bảng nói rõ. **Công thức, không cộng trùng:** `$ = input·p_in + ephemeral_5m·p_w5m + ephemeral_1h·p_w1h + cache_read·p_read + output·p_out`; khi thiếu `cache_creation.*` thì `cache_creation_input_tokens·p_w5m` (ghi giả định); **`thinking_tokens` đã nằm trong `output_tokens`** nên không cộng thêm (cột thinking chỉ để xem); `ephemeral_5m + ephemeral_1h = cache_creation_input_tokens`, chỉ dùng một trong hai cách; Codex: `cached_input` theo giá cache read, `input − cached_input` theo giá input [chưa kiểm định nghĩa `input_tokens` của Codex có bao gồm cached hay không — kiểm trên rollout thật: nếu `input ≥ cached` luôn thì bao gồm]; model không có trong bảng giá → `est $ = —`, không phải 0. Đề xuất in sau bảng, mỗi dòng một phép tính kiểm được và **giữ nguyên lượng token để so giá**: «`agent:custom:tho` chiếm 23% token; 31% số lượt chạy opus — cùng lượng token đó theo giá sonnet: ≈ −Z $ (so giá, không dự báo chi phí hay chất lượng sau khi đổi; bảng giá ngày …)». Chỉ gợi ý đổi model và trả kết quả ngắn; không gợi ý về effort (transcript ghi `effort` nhưng không có số đo token-theo-effort tách được), không gợi ý về fork (chưa đo được).

**Chỗ trong code.** `crates/sos-cli/src/usage/{mod.rs, claude.rs, codex.rs, prices.rs, table.rs}`; phần đọc/khử trùng/cộng là hàm thuần nhận iterator dòng, nên test đơn vị không cần file. **Không dùng chung gì với `sos hook`**: hook đọc một payload JSON từ stdin, không chạm transcript; chỗ chung duy nhất là `serde_json` và thư viện ngày tháng. `sos sessions` (đợt 4) đọc `local_*.json` của app, cũng là bộ khác. Đọc **streaming từng dòng** (`BufReader::read_line` vào buffer tái dùng → `serde_json::from_str`; dòng hỏng bỏ qua và đếm `bad_json`). Trạng thái giữ lại chỉ là ID, số và metadata ngắn: map khử trùng theo phiên, map `tool_use_id → (subagent_type, ký tự đã thấy, isAsync)` của file cha, map `response_id` của Codex — **không giữ prompt/content** (độ dài `tool_result` tính rồi bỏ). Ngân sách đo được, không hứa suông: file lớn nhất hiện 1,01 GB, dòng lớn nhất ≈4,14 MB (số của Người soát); giới hạn dòng `--max-line-bytes` mặc định 16 MiB, dòng vượt → bỏ, đếm `oversize_lines` và in `coverage_warning` (không im); mục tiêu RSS < 200 MB trên fixture «nhiều ID + dòng rất lớn», đo bằng `/usr/bin/time -l` trong `tests/usage.sh` và ghi số vào CHANGELOG — cho tới khi đo, con số là dự kiến. Bỏ qua sớm file có `mtime` trước `--since`. Không spawn process, không ghi gì, không gửi gì. Lọc `--project` so với `cwd` trong bản ghi (tên thư mục mã hoá của `~/.claude/projects/` chỉ dùng để bỏ qua sớm; hai đường dẫn cùng basename là hai dự án).

**Test.** `tests/usage/fixtures/` (ẩn danh tự dựng, giữ cấu trúc thật): Claude — phiên 09/2026 không `querySource` + subagent với meta.json (có `toolUseId`); phiên 11/2026 có `api-request`, tin stream 3 dòng (output 8 → 40.261 như mẫu thật), tool call `Agent` foreground, một `Agent` background với `isAsync` ack rồi kết quả cuối sau nhiều tool xen kẽ, một `Agent` lỗi, subagent độc lập có ngữ cảnh ban đầu lớn (để heuristic không gọi nó là fork), sidechain **có** và **không có** file subagent, `<synthetic>`, dòng thiếu ID, resume lặp lịch sử trong cùng file và bản sao sang file thứ hai cùng `sessionId`, dòng cuối ghi dở, dòng 20 MB (vượt `--max-line-bytes`), thinking thiếu một phần, hai dự án cùng basename, bản ghi quanh nửa đêm UTC. Codex — rollout có `token_count` lặp cùng tổng (tái hiện dư 78.217), rollout có compaction trong `token_usage_record` + bản sao `compacted` (tái hiện 246.204), phiên chỉ có `token_count` (fallback), counter reset, main và child cùng `originator`, `agent_role` null, đổi model giữa turn.
`tests/usage.sh`: (1) **oracle Python chỉ cho phần giao nhau**: chép `agent-usage.py` vào `tests/usage/agent-usage.py` (ghi nguồn), chạy cả hai trên cùng fixture, cùng `--since`/dự án, với mapping vai minh bạch ghi trong `tests/usage/role-map.toml` (`main (Quản đốc)` ↔ `sdk`, `agentType` ↔ `agent:custom:…`/`agent:builtin:…`), **bỏ sidechain không có file subagent** ở cả hai bên; so từng ô bốn cột token + `calls`, và so **tổng theo model**; (2) phần Python không có — `querySource`, Codex, compaction, `subagent_result_text_chars`, `$`, coverage — so với `expected.json` **tính độc lập** (bằng tay từ fixture, ghi phép tính cạnh số); (3) **privacy**: fixture đặt chuỗi bí mật giả (`SOS-SECRET-7f3a…`) trong prompt, `meta.description`, `tool_result` và một dòng JSON hỏng; bảng, `--json`, `--explain` và stderr đều phải không chứa chuỗi đó; (4) bộ nhớ: `/usr/bin/time -l` trên fixture lớn, RSS dưới ngân sách. Thêm một lần chạy thật trên máy Sếp (`sos usage --since 2026-10-01` so với `agent-usage.py` trên `~/code/tally`, phần giao nhau) ghi vào CHANGELOG như 1a đã ghi 178/178.

**Sửa theo soát mục 7 (05/11, Người soát Codex, SOUND-WITH-CHANGES, 5 chặn + 2 nên sửa):**

| Điểm | Chỗ sửa |
|---|---|
| 1. Codex cộng `last_token_usage` dư 78.217; compaction 246.204 chỉ ở `token_usage_record` | «Codex»: nguồn chính `token_usage_record` khử trùng `response_id`, compaction một lần, bỏ bản sao `compacted`; fallback hiệu `total_token_usage` với xử lý reset; không cộng hai nguồn; fixture tái hiện cả hai số |
| 2. «Kết quả subagent phình cha» đặt tên quá đại lượng; `isAsync` | Đổi thành `subagent_result_text_chars`: nối theo `tool_use_id` xuyên bản ghi, tách spawn ack (`isAsync`)/kết quả cuối/lỗi, chỉ đo văn bản trả về một lần; token chỉ trong `estimates`; cột bảng đổi tên `result_chars` |
| 3. Ngưỡng 80% làm «chi phí fork» | Bỏ cột fork và ngưỡng; cha–con từ `meta.toolUseId`/`parentAgentId`, không có → `unknown`; chỉ còn heuristic có nhãn `large_initial_context` trong `--explain`/`estimates`; đề xuất không nói về fork |
| 4. `originator` không phải vai Codex | «Codex»: loại phiên từ `session_meta.payload.source.subagent.thread_spawn`, vai = `agent_role` hoặc `unknown`, model từ `turn_context.payload.model` theo turn |
| 5. Oracle Python không hợp lệ cho từng ô theo vai | «Test» (1)–(2): oracle chỉ cho phần giao nhau với `role-map.toml`, cùng phạm vi, so thêm tổng theo model; phần còn lại `expected.json` tính độc lập; chuẩn hoá `agentType` trước khi báo lệch |
| 6. Chính sách trùng file/resume/sidechain/synthetic | «Một request tính một lần»: khử trùng theo phiên không theo file; sidechain thuộc file subagent nếu có, giữ nếu không; `<synthetic>` bỏ và đếm; compaction summary không là lần gọi mới; fixture tương ứng |
| 7. Ngân sách bộ nhớ chưa kiểm được | «Chỗ trong code»: trạng thái chỉ ID/số, `--max-line-bytes` + `oversize_lines` + `coverage_warning`, RSS đo bằng `/usr/bin/time -l` trong test, ghi CHANGELOG |
| Privacy, tiền | «Test» (3) chuỗi bí mật giả; «Công thức»: thinking không cộng lại vào output, TTL không cộng lại vào cache write, Codex cached input theo giá cache read, model lạ → `—`, đề xuất «giữ lượng token để so giá» |

## 8. Sửa theo soát 05/11 (Người soát Codex, SOUND-WITH-CHANGES, 9 điểm chặn + 1 nên sửa)

| Điểm | Chỗ sửa trong bản này |
|---|---|
| 1. Codex Windows bỏ fail-closed | Mục 3 «Nguyên tắc» + «Launcher»: giữ `python -c` làm launcher gọi `sos hook codex … --protocol 1`; bốn ca thử thật trong Codex Windows là điều kiện phát hành 1b |
| 2. Vỏ POSIX chưa bao npm shim | Mục 3: `hook` vào nhánh miễn tải của `bin/sos-npm`; launcher không `exec`, mọi mã ≠ 0 → exit 2; test với shim npm thật và tải giả thất bại; `--protocol` chặn binary cũ |
| 3. Luật đầu tiên quyết định cả payload | Mục 2 «Đơn vị xét»: từng lệnh đơn / từng path (kể cả `Move to`) qua toàn bộ luật; `allow` bắt buộc có `scope`, không dừng duyệt; ca ghép bắt buộc trong `cases.jsonl` |
| 4. `check.run` nội suy chuỗi | Mục 2 `check`: `program` + `args` có cấu trúc, thay biến vào từng đối số sau parse, cấm `{…}` trong `sh -c`; test đối số khoảng trắng/quote/`;`/`$()`/`-` |
| 5. Bộ đọc sim-guard làm bộ phân tích chung | Mục 2 «Mô hình khớp» `command.*`: wrapper giữ lại (`sudo cat` chặn), heredoc-shell lex lại, cú pháp không hiểu → không phân tích được → chặn; `args` là mảng điều kiện có cấu trúc thay regex trên chuỗi nối |
| 6. Chuẩn hoá path | Mục 2 `paths.*`: `root` của file luật, symlink ancestor tồn tại dài nhất, `..` theo thành phần, containment theo thành phần, hoa/thường không suy từ OS (`case_insensitive` tường minh), NUL/rỗng → chặn; Codex giữ Add/Update/Delete/Move to |
| 7. Payload đọc được / thiếu file luật | Mục 2 «Đóng khi hỏng»: schema theo tool, lỗi trích xuất của tool có luật → chặn; `--file` bắt buộc cho hook đã cài, không có đường «thiếu file → qua»; lỗi I/O marker/content là lỗi, không phải «không khớp» |
| 8. Ví dụ luật mất hành vi | Mục 2 TOML (luật `vault-no-build`, `vault-no-simctl`, `book-ssh-win-ok` có `scope`, `hub ocr` + 3 script Python, `when.role` hai nguồn + trim, `.markdown`, `(?m)`, `hub` đường dẫn tuyệt đối) + «Bảng bảo toàn hành vi năm script»; quy định mỗi ca có kết quả bản cũ trước khi dịch |
| 9. Xoá adapter theo fleet | Mục 3 «Khi nào bỏ adapter»: kiểm tại repo đang `update` (settings còn tham chiếu hoặc không parse được → `KeepReferenced`, không xoá, thoát ≠ 0); sửa chi tiết `KeepModified` không sinh `.sos-new` |
| 10. Thẩm quyền | Mục 4: giữ giới hạn «không build trên Mac», `tong-dai` build qua Actions repo riêng; Mục 5 Q2: Reel «mục tiêu đo được» và `hub` tách đôi ghi rõ là ĐỀ XUẤT chờ Sếp |
| `--wiring` (không chặn) | Mục 3: sửa theo cấu trúc JSON, giữ khóa lạ, chạy lặp không sinh bản sao, ghi an toàn, `--dry-run`; sửa lời hứa «never overwritten» cùng implementation |
| Review 1a mục 8 còn mở (bằng chứng JSON stdout) | Mục 3 «Test launcher»: tách stdout/stderr, parse toàn bộ stdout là đúng một JSON ở event cần JSON |
