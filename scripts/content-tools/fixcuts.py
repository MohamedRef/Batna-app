import sys, re, os, csv
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import expand_quotes as X
repo, course = sys.argv[1], sys.argv[2]
cuts = {r[0] for r in csv.reader(open(sys.argv[3]), delimiter="\t")}
apply = "--apply" in sys.argv
def smap(t):
    k, ix = [], []
    for i, ch in enumerate(t):
        c = ch.lower().replace("ﬁ", "f").replace("ﬂ", "f")
        if re.match(r"[a-z0-9]", c): k.append(c); ix.append(i)
    return "".join(k), ix
ABBRDOT = re.compile(r"(?:\b(?:e\.g|i\.e|vs|et al|etc|Dr|Fig|approx|No)\.|\b[A-Z]\.)$")
def sentence(unit, s, e):
    # left: back to a sentence end
    L = s
    while L > 0:
        m = None
        j = L - 1
        if unit[j] in ".?!" and (j + 1 >= len(unit) or unit[j+1] == " ") and not ABBRDOT.search(unit[:j+1]) and not re.match(r"\d", unit[j+2:j+3] or "x"):
            break
        L -= 1
    while L < s and unit[L] == " ": L += 1
    R = e
    while R < len(unit):
        if unit[R] in ".?!" and (R + 1 >= len(unit) or unit[R+1] == " ") and not ABBRDOT.search(unit[:R+1]):
            R += 1; break
        R += 1
    return unit[L:R].strip()
lecs = sorted(x[:-5] for x in os.listdir(f"{repo}/content/{course}") if re.match(r"L\d+\.yaml$", x))
out = csv.writer(sys.stdout, delimiter="\t")
edits = {}
for lec in lecs:
    try: pg = X.pages(course, lec)
    except FileNotFoundError: continue
    for rid, page, quote, path in X.refs(repo, course, lec):
        if rid not in cuts: continue
        us = X.units(pg.get(page, []))
        q = X.sq(quote); done = False
        for k, ls in us:
            if k == "t": continue
            unit = X.fix(X.clean(k, ls))
            keys, ix = smap(unit)
            at = keys.find(q)
            if at < 0: continue
            new = sentence(unit, ix[at], ix[at + len(q) - 1] + 1)
            out.writerow([rid, page, len(quote.split()), len(new.split()), quote, new])
            edits.setdefault(path, []).append((rid, new)); done = True
            break
        if not done:
            out.writerow([rid, page, "-", "-", quote, "SPANS-UNITS"])
if apply:
    for path, lst in edits.items():
        s = open(path, encoding="utf-8").read()
        for rid, new in lst:
            if "#" in rid:
                lid, n = rid.split("#"); n = int(n)
                p = s.index(f"- id: {lid}\n")
                st = s.index("    steps:", p)
                pos = st
                for _ in range(n):
                    pos = s.index("      - kind:", pos) + 1
            else:
                pos = s.index(f"id: {rid}\n")
            m = re.compile(r"(quote: )(\"(?:[^\"\\]|\\.)*\"|'(?:[^']|'')*'|[^,}\n][^}\n]*?)(\s*(?:\}|,|$))", re.M).search(s, pos)
            s = s[:m.start()] + m.group(1) + X.yq(new) + m.group(3) + s[m.end():]
        open(path, "w", encoding="utf-8").write(s)
