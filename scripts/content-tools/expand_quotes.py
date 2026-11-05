#!/usr/bin/env python3
"""Expand every book quote to its whole unit on the page (Mohamed, 2026-10-29 and
2026-11-02): a quote is the full bullet/paragraph that holds the fact, or the
whole table/list, never a cut-off line. No word cap.

Usage (run with python3 -I):
  expand_quotes.py <repo> <course> [--lecture L01] --report out.tsv [--apply]

Report columns: ref, page, status, old, new
  same      already the whole unit
  expanded  widened to the whole bullet/paragraph (auto-applied with --apply)
  table     inside a book table/list block: NEW holds the raw block; format it by
            hand (grouped list: one row per line, sub-items indented 2 spaces;
            grid: `| a | b |` rows, see crates/core/src/quote.rs) and read the
            page image for columns. Never auto-applied.
  notfound  not in the text layer (OCR/image page): check by eye.

--apply rewrites only `expanded` rows, in content/<course>/LNN.yaml and
content/lessons/<course>/LNN.yaml, keeping the rest of the file as is. Then run
`cargo run -q -p batna-tools -- refs --lecture <course>.LNN --write`.
"""
import os, re, sys, yaml

# Sources live outside git (docs/HANDOFF.md, "Sources outside git").
SOURCES = os.environ.get("BATNA_SOURCES", "/mnt/project-files/medstudy")
BOOK = os.path.join(SOURCES, "book")
COURSES = os.path.join(SOURCES, "courses")
BULLET = re.compile(r"^\s*([\u25aa\u2022\u25cf\u25a0\u27a2\u2219\u25ba\u2013\-\*\uf0b7\uf0a7\uf0d8\ue000-\uf8ff]|\d+[.)]\s|[a-z][.)]\s)")
TITLE = re.compile(r"^\s*(Table|Box)\s+\d+[A-Za-z]?\s*[:.]")
FOOT = re.compile(r"^\s*(Lecture\s+\d+\s*:.*\d+\s*$|Chapter\s+\d+\s*$|\d+\s*$)")


def sq(s):
    return re.sub(r"[^a-z0-9]", "", s.lower().replace("ﬁ", "fi").replace("ﬂ", "fl"))


def pages(course, lec):
    f = f"{BOOK}/{lec}.txt" if course == "hb" else f"{COURSES}/{course}/text/{lec}.txt"
    out, cur = {}, None
    for line in open(f, encoding="utf-8"):
        m = re.match(r"=== (?:book page|slide) (\d+)", line)
        if m:
            cur = int(m.group(1)); out[cur] = []
        elif cur is not None:
            if line.startswith("[OCR"):
                cur = None  # OCR echo of a slide image: skip, the text layer is above
                continue
            out[cur].append(line.rstrip("\n"))
    return out


def heading(l):
    t = l.strip()
    return len(t.split()) <= 6 and t[:1].isupper() and not re.search(r"[.,;:)]$", t) and not BULLET.match(l)


def units(lines):
    """[(kind, [lines])]: kind 'p' (one bullet or paragraph), 'h' (heading) or
    't' (table block). Blank lines inside a bullet don't end it (the PDF text
    breaks paragraphs at random); a bullet marker, a heading or a table does."""
    out, cur, kind, blank = [], [], "p", 0
    def flush():
        nonlocal cur
        if any(l.strip() for l in cur):
            out.append((kind, cur))
        cur = []
    for l in lines:
        if FOOT.match(l):
            continue
        if not l.strip():
            blank += 1
            if kind == "t" and blank >= 2:
                flush(); kind = "p"
            continue
        gap, blank = blank, 0
        if TITLE.match(l):
            flush(); kind = "t"; cur = [l]; continue
        if kind == "t":
            cur.append(l); continue
        if BULLET.match(l):
            flush(); kind = "p"
        elif heading(l) and (not cur or gap or re.search(r"[.:]\s*$", cur[-1])):
            flush(); out.append(("h", [l])); kind = "p"; continue
        cur.append(l)
    flush()
    return out


# unglue() from clean_quotes.py ("fibrosisfollows" -> "fibrosis follows"); that
# script runs on import, so only its helpers are loaded.
_src = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "clean_quotes.py"), encoding="utf-8").read()
_ns = {}
exec(_src[:_src.index("def clean(")], _ns)
unglue = _ns["unglue"]

CHARS = {"―": '"', "‖": '"', "“": '"', "”": '"', "’": "'", "‘": "'", "ﬁ": "fi", "ﬂ": "fl", "\u00a0": " "}


def fix(s):
    for a, b in CHARS.items():
        s = s.replace(a, b)
    return " ".join(unglue(t) for t in s.split(" ")) if s else s


def clean(kind, ls):
    if kind == "t":
        return "\n".join(l.rstrip() for l in ls)
    if kind == "h":
        return ls[0].strip()
    s = " ".join(BULLET.sub("", l, count=1).strip() if i == 0 else l.strip() for i, l in enumerate(ls))
    return re.sub(r"\s+", " ", s).strip()


def expand(quote, page_lines):
    us = units(page_lines)
    keys = [sq(clean(k, ls)) for k, ls in us]
    q = sq(quote)
    if not q:
        return "notfound", quote
    full = "".join(keys)
    at = full.find(q)
    if at < 0:
        return "notfound", quote
    end = at + len(q)
    pos, span = 0, []
    for i, k in enumerate(keys):
        if pos < end and pos + len(k) > at:
            span.append(i)
        pos += len(k)
    kinds = {us[i][0] for i in span}
    new = ("\n" if "t" in kinds else " ").join(clean(*us[i]) for i in span)
    new = "\n".join(fix(l) for l in new.split("\n"))
    if "t" in kinds:
        return "table", new
    return ("same" if sq(new) == q else "expanded"), new


def refs(repo, course, lec):
    def load(f):
        try:
            return yaml.safe_load(open(f, encoding="utf-8")) or {}
        except yaml.YAMLError as e:  # PyYAML chokes on `?]` where the Rust parser doesn't
            print(f"{f}: PyYAML can't read it ({str(e).splitlines()[0]}); check its quotes by hand", file=sys.stderr)
            return {}
    f = f"{repo}/content/{course}/{lec}.yaml"
    if os.path.exists(f):
        for it in load(f).get("items") or []:
            s = it.get("src") or {}
            if s.get("quote"):
                yield it["id"], s.get("page"), s["quote"], f
    f = f"{repo}/content/lessons/{course}/{lec}.yaml"
    if os.path.exists(f):
        for ls in load(f).get("lessons") or []:
            for i, st in enumerate(ls.get("steps") or []):
                if st.get("quote"):
                    yield f"{ls['id']}#{i+1}", st.get("page"), st["quote"], f


def yq(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n") + '"'


QLINE = re.compile(r"(quote: )(\"(?:[^\"\\]|\\.)*\"|'(?:[^']|'')*'|[^,}\n][^}\n]*?)(\s*(?:\}|,|$))")


def apply(path, pairs):
    """Replace each quote scalar whose value is a key of pairs."""
    s = open(path, encoding="utf-8").read()
    n = 0
    def f(m):
        nonlocal n
        try:
            v = yaml.safe_load("x: " + m.group(2))["x"]
        except Exception:
            return m.group(0)
        if isinstance(v, str) and v in pairs:
            n += 1
            return m.group(1) + yq(pairs[v]) + m.group(3)
        return m.group(0)
    s = QLINE.sub(f, s)
    open(path, "w", encoding="utf-8").write(s)
    return n


def main():
    a = sys.argv[1:]
    repo, course = a[0], a[1]
    lecs = [a[a.index("--lecture") + 1]] if "--lecture" in a else sorted(
        x[:-5] for x in os.listdir(f"{repo}/content/{course}") if re.match(r"L\d+\.yaml$", x))
    rep = open(a[a.index("--report") + 1], "w", encoding="utf-8")
    rep.write("ref\tpage\tstatus\told\tnew\n")
    counts = {}
    for lec in lecs:
        try:
            pg = pages(course, lec)
        except FileNotFoundError:
            print(f"{course}.{lec}: no text", file=sys.stderr); continue
        todo = {}
        for rid, page, quote, path in refs(repo, course, lec):
            st, new = expand(quote, pg.get(page, []))
            counts[st] = counts.get(st, 0) + 1
            rep.write("\t".join([rid, str(page), st, quote.replace("\n", "\\n"), new.replace("\n", "\\n")]) + "\n")
            if st == "expanded":
                todo.setdefault(path, {})[quote] = new
        if "--apply" in a:
            for path, pairs in todo.items():
                apply(path, pairs)
    print(course, counts)


if __name__ == "__main__":
    main()
