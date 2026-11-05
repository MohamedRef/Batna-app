"""Shared helpers for the Batna page-reference audit (read-only on the repo).

Book text: /mnt/project-files/medstudy/book/LNN.txt (printed pages), plus
Tesseract OCR of the image-only pages in pagerefs/ocr/<page>.txt (merged in).
"""
import re, math, yaml, glob, os
# Sources live outside git (docs/HANDOFF.md, "Sources outside git").
SOURCES = os.environ.get("BATNA_SOURCES", "/mnt/project-files/medstudy")
BOOK = os.path.join(SOURCES, "book")
OCR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ocr")
# Tesseract OCR of EVERY page (2026-10-29): catches figure/table text on pages that also have prose.
OCR_ALL = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ocr-all")
STOP = set("""the and for with that this from are was were has have had not but can its into than then they them their there which who whom
what when where while will would should could may might also more most less such other some any all each both only very over under about after
before between during through per via due one two three use used using usually often mainly most patients patient disease liver hepatic cases
case include includes including seen common commonly cause causes caused associated based level levels high low increased decreased normal""".split())

def norm(s):
    for a, b in (("‐", "-"), ("‑", "-"), ("–", "-"), ("—", "-"), ("’", "'"), ("‘", "'"),
                 ("“", '"'), ("”", '"'), ("―", '"'), ("‖", '"'), ("ﬁ", "fi"), ("ﬂ", "fl")):
        s = s.replace(a, b)
    return s

def toks(s):
    s = norm(s).lower()
    return [t for t in re.findall(r"[a-z][a-z0-9\-]+|\d+(?:\.\d+)?", s) if t not in STOP and (len(t) >= 3 or t[0].isdigit())]

def words(s):
    """Normalised word stream used for quote matching (case, punctuation and line breaks ignored)."""
    return re.findall(r"[a-z0-9]+", norm(s).lower())

def pages(lec, ocr=True):
    """{printed page: text} for lecture 'L01'..'L20'."""
    out, cur = {}, None
    for line in open(f"{BOOK}/{lec}.txt", encoding="utf-8"):
        m = re.match(r"=== book page (\d+)", line)
        if m:
            cur = int(m.group(1)); out[cur] = []
        elif cur is not None:
            out[cur].append(line)
    out = {k: "".join(v) for k, v in out.items()}
    if ocr:
        for p in out:
            for d in (OCR, OCR_ALL):
                f = os.path.join(d, f"{p}.txt")
                if os.path.exists(f):
                    out[p] += "\n[OCR]\n" + open(f, encoding="utf-8").read()
    return out

SKIP = ("id", "kind", "status", "src", "topic", "tags", "page", "quote", "image", "pages")
def strings(x):
    if isinstance(x, str): yield x
    elif isinstance(x, dict):
        for k, v in x.items():
            if k not in SKIP: yield from strings(v)
    elif isinstance(x, list):
        for v in x: yield from strings(v)

def refs(repo, lec):
    """Yield (ref_id, kind, page, quote, text) for every page reference of one lecture:
    items in content/hb/LNN.yaml and lesson steps (id 'hb.LNN.lessonK#S', S counted from 1)."""
    f = f"{repo}/content/hb/{lec}.yaml"
    for it in yaml.safe_load(open(f))["items"]:
        s = it.get("src") or {}
        yield it["id"], it["kind"], s.get("page"), s.get("quote"), " \n".join(strings(it))
    f = f"{repo}/content/lessons/hb/{lec}.yaml"
    if os.path.exists(f):
        for ls in yaml.safe_load(open(f))["lessons"]:
            for i, st in enumerate(ls["steps"]):
                if st.get("page") is None and not st.get("quote"):
                    continue
                yield f"{ls['id']}#{i+1}", "step:" + st["kind"], st.get("page"), st.get("quote"), " \n".join(strings(st))

def quote_on(quote, text):
    """Verbatim up to case, punctuation, spacing (the PDF text often glues words: 'factorsinclude')."""
    return "".join(words(quote)) in "".join(words(text))
