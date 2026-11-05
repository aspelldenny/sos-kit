"""Parsers of bite on captured runner output (no pytest or Xcode needed). Run by tests/bite.sh.

`bite_parsers.py scripts/bite` loads the Python version; `bite_parsers.py --sos path/to/sos` asks
`sos bite --internal …` the same questions, so both versions answer the same samples."""
import importlib.machinery, importlib.util, json, subprocess, sys, tempfile
from pathlib import Path


class Sos:
    """The functions this file calls, answered by the Rust version."""

    def __init__(self, exe):
        self.exe = exe

    def _ask(self, kind, data, *extra):
        r = subprocess.run([self.exe, "bite", "--internal", kind, *extra], input=data.encode(), capture_output=True)
        if r.returncode:
            raise SystemExit(f"sos bite --internal {kind}: {r.stderr.decode(errors='replace')}")
        return json.loads(r.stdout.decode())

    def results(self, out):
        return self._ask("results", out)["results"]

    def has_verdicts(self, out):
        return self._ask("results", out)["has_verdicts"]

    def any_failed(self, out):
        return self._ask("results", out)["any_failed"]

    def test_names(self, lines, root):
        return set(self._ask("test-names", json.dumps(lines), "--internal-dir", str(root)))

    def mutants(self, lines, limit, root):
        got = self._ask("mutants", json.dumps(lines), "--internal-dir", str(root), "--mutants", str(limit))
        return [tuple(m) for m in got]


if sys.argv[1] == "--sos":
    b = Sos(sys.argv[2])
else:
    loader = importlib.machinery.SourceFileLoader("bite", sys.argv[1])
    b = importlib.util.module_from_spec(importlib.util.spec_from_loader("bite", loader))
    loader.exec_module(b)

OUT = """test_a (tests.t.C.test_a) ... ok
test_b (tests.t.C)
Docstring line ... FAIL
tests/test_p.py::TestP::test_x PASSED
tests/test_p.py::test_top[1] PASSED
tests/test_p.py::test_top[2] FAILED
Test Case '-[App.Feed testLoads]' passed (0.1 seconds).
Test case '-[AppTests.Sync testRetry()]' failed on 'iPhone' (0.2 seconds)
Test Case 'Store.testSave' passed (0.0 seconds)
"""
SRC = """import XCTest

@MainActor
final class FeedTests: XCTestCase {
    func testLoads() {
        XCTAssertTrue(true)
    }

    override func setUp() {
        super.setUp()
    }
}

def test_top():
    assert 1
"""
fails = []
got = sorted(b.results(OUT).items())
want = [(".test_top", False), ("C.test_a", True), ("C.test_b", False), ("Feed.testLoads", True),
        ("Store.testSave", True), ("Sync.testRetry", False), ("TestP.test_x", True)]
if got != want:
    fails.append(f"results: {got}")
d = Path(tempfile.mkdtemp())
(d / "t.swift").write_text(SRC)
for line, expect in ((6, {"FeedTests.testLoads"}), (10, set()), (15, {".test_top"})):
    names = b.test_names({"t.swift": [(line, "")]}, d)
    if names != expect:
        fails.append(f"test_names line {line}: {names} (want {expect})")

SPEC = """▶ group
  ✔ works (0.296917ms)
  ✔ same (0.056ms)
✔ group (0.884208ms)
▶ other
  ✖ same (0.101958ms)
  ✖ fails here (1.0ms)
✖ other (0.5385ms)
✖ t/bad.test.mjs (116.112ms)
ℹ tests 4

✖ failing tests:

test at t/a.test.mjs:3:27
✖ fails here (1.0ms)
"""
TAP = """TAP version 13
# Subtest: group
    # Subtest: works
    ok 1 - works
      ---
      type: 'test'
      ...
    ok 2 - tap skipped # SKIP no db
    not ok 3 - tap fails
      ---
      type: 'test'
      ...
    1..3
not ok 1 - group
  ---
  type: 'suite'
  ...
# Subtest: t/bad.test.mjs
not ok 2 - t/bad.test.mjs
  ---
  type: 'test'
  ...
"""
OTHER = """ ✓ src/a.test.ts > P569 path > allows the exact route 3ms
 × src/a.test.ts > P569 path > redirects a lookalike 0ms
    ✓ jest title (3 ms)
    ✕ jest broken (12 ms)
test tests::rounds_down ... ok
test store::tests::rejects_empty ... FAILED
test a::tests::same ... ok
test b::tests::same ... FAILED
test result: FAILED. 1 passed; 1 failed
"""
for name, out, want in (
        ("node spec", SPEC, {".works": True, ".fails here": False}),
        ("node tap", TAP, {".works": True, ".tap fails": False}),
        ("vitest/jest/cargo", OTHER, {".allows the exact route": True, ".redirects a lookalike": False, ".jest title": True,
                                      ".jest broken": False, ".rounds_down": True, ".rejects_empty": False})):
    got = b.results(out)
    if got != want:
        fails.append(f"{name}: {got} (want {want})")
for out in ("✔ works (1ms)\n✖ test/my file.test.mjs (1ms)\n", "✔ works (1ms)\n✖ my file.test.mjs (1ms)\n"):
    if b.results(out) != {".works": True} or b.any_failed(out):
        fails.append(f"a test file that did not load is not a failed test: {out!r}")
DUP = "▶ a\n  ✔ same (1ms)\n▶ b\n  ✖ same (1ms)\n"
if b.results(DUP) or not b.has_verdicts(DUP) or not b.any_failed(DUP):
    fails.append("titles that are all shared: no verdicts, but per-test output and a failure are still seen")
(d / "a.test.mjs").write_text("""import test from 'node:test';
describe('clamp', () => {
  it('keeps "low" end', () => {
    assert.equal(clamp(-1), 0);
  });

  test.only(`one liner`, () => assert.ok(1));
});
""")
for line, expect in ((4, {'.keeps "low" end'}), (7, {".one liner"}), (2, set())):
    names = b.test_names({"a.test.mjs": [(line, "")]}, d)
    if names != expect:
        fails.append(f"js test_names line {line}: {names} (want {expect})")
(d / "lib.rs").write_text("""fn helper() -> u8 { 1 }

#[cfg(test)]
mod tests {
    #[test]
    fn rounds_down() {
        assert_eq!(super::helper(), 1);
    }

    #[tokio::test]
    async fn waits() {}
}
""")
for line, expect in ((1, set()), (7, {".rounds_down"}), (11, {".waits"})):
    names = b.test_names({"lib.rs": [(line, "")]}, d)
    if names != expect:
        fails.append(f"rust test_names line {line}: {names} (want {expect})")
(d / "g.ts").write_text("const s = new Set<string>(); let f = (a) => a\n")
if b.mutants({"g.ts": [(1, "const s = new Set<string>(); let f = (a) => a")]}, 9, d):
    fails.append("generics and arrows must not be mutated as comparisons")
(d / "m.ts").write_text("if (a !== 'x') f()\nif (a === 1) f()\nif (a == b) f()\nif (a != b) f()\n")
got = [new for _, _, _, new in b.mutants({"m.ts": [(i, l) for i, l in enumerate((d / "m.ts").read_text().split("\n")[:4], 1)]}, 9, d)]
if got != ["if (a === 'x') f()", "if (a !== 1) f()", "if (a != b) f()", "if (a == b) f()"]:
    fails.append(f"equality mutants: {got}")
# Python's \w is a letter, number or _: a combining mark ends a word, ² does not (review 2.2 round 1).
(d / "u.ts").write_text("export const tru\u0065\u0301 = x;\nlet n = a\u00b2 + 1\n")
got = [new for _, _, _, new in b.mutants({"u.ts": [(1, "export const tru\u0065\u0301 = x;"), (2, "let n = a\u00b2 + 1")]}, 9, d)]
if got != ["export const fals\u0065\u0301 = x;", "let n = a\u00b2 - 1"]:
    fails.append(f"unicode word edges: {got!r}")
UNI = "test_x\u00b2 (mod.C.test_x\u00b2) ... FAIL\n"
if b.results(UNI) != {"C.test_x\u00b2": False} or not b.any_failed(UNI):
    fails.append(f"unittest name with \u00b2: {b.results(UNI)}")
for tap, want in (("not ok 1 - a # SKIP\u00e9 later\n", {".a # SKIP\u00e9 later": False}),
                  ("not ok 1 - a # SKIP\u0301 later\n", {}), ("ok 1 - a # SKIP\nok 2 - b\n", {".b": True})):
    if b.results(tap) != want:
        fails.append(f"TAP # SKIP word edge {tap!r}: {b.results(tap)} (want {want})")
print("\n".join(fails) or "ok")
