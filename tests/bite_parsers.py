"""Parsers of scripts/bite on captured runner output (no pytest or Xcode needed). Run by tests/bite.sh."""
import importlib.machinery, importlib.util, sys, tempfile
from pathlib import Path

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
print("\n".join(fails) or "ok")
