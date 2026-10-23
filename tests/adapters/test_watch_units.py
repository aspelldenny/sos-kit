"""scripts/test-watch.py pieces without a hook payload: the Python < 3.11 TOML reader and the
match of the project's own test commands. Usage: python3 tests/adapters/test_watch_units.py <test-watch.py>"""
import sys

src = open(sys.argv[1], encoding="utf-8").read().rsplit("\ntry:\n    main()", 1)[0]
ns = {"__file__": sys.argv[1]}
exec(compile(src, sys.argv[1], "exec"), ns)
bad = []
for text, want in {
    '[advisor] # note\ntests = ["a b"]\n': ["a b"],
    '[advisor]\ntests = [ # "not this"\n  "x ] y", \'lit # z\' ]\n': ["x ] y", "lit # z"],
    '[advisor]\ntests = ["a\\"b"]\n': ['a"b'],
    '[advisor]\nafter = 2\n[state]\ntests = ["no"]\n': [],
    '[advisor]\ntests = ["unclosed"\n': [],
}.items():
    got = ns["toml_string_array"](text, "advisor", "tests")
    if got != want:
        bad.append(f"toml {text!r}: {got!r} != {want!r}")
own = [["node", "scripts/check.mjs"], ["node", "scripts/my check.mjs"]]
for cmd, want in [
    ("node scripts/check.mjs", "node scripts/check.mjs"),
    ("CI=1 timeout 60 node scripts/check.mjs --full", "node scripts/check.mjs"),
    ("cd x && node scripts/check.mjs", "node scripts/check.mjs"),
    ('node "scripts/check.mjs --other"', None),
    ('node "scripts/my check.mjs"', "node 'scripts/my check.mjs'"),
    ("node scripts/check.mjsx", None),
]:
    got = ns["test_segment"](cmd, own)
    if got != want:
        bad.append(f"match {cmd!r}: {got!r} != {want!r}")
print("\n".join(bad) or "ok")
