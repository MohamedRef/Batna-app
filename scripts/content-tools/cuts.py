import sys, re, os, csv
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import expand_quotes as X
repo = sys.argv[1]; courses = sys.argv[2:]
TERM = set('.!?:;)]"%\'')
def squash_map(text):
    keys, idx = [], []
    for i, ch in enumerate(text):
        c = ch.lower().replace("ﬁ","f").replace("ﬂ","f")
        if re.match(r"[a-z0-9]", c):
            keys.append(c); idx.append(i)
    return "".join(keys), idx
out = csv.writer(sys.stdout, delimiter="\t")
for course in courses:
    lecs = sorted(x[:-5] for x in os.listdir(f"{repo}/content/{course}") if re.match(r"L\d+\.yaml$", x))
    for lec in lecs:
        try: pg = X.pages(course, lec)
        except FileNotFoundError: continue
        cache = {}
        for rid, page, quote, path in X.refs(repo, course, lec):
            if "\n" in quote: continue
            if page not in cache:
                lines = [l for l in pg.get(page, []) if not X.FOOT.match(l)]
                text = X.fix(" ".join(l.strip() for l in lines))
                text = re.sub(r"\s+", " ", text)
                cache[page] = (text,) + squash_map(text)
            text, keys, idx = cache[page]
            q = X.sq(quote)
            if not q: continue
            at = keys.find(q)
            if at < 0: continue
            s, e = idx[at], idx[at + len(q) - 1]
            before = text[max(0, s-40):s].rstrip()
            after = text[e+1:e+60].lstrip()
            # strip trailing punctuation the quote itself includes
            qa = quote.strip()
            bad = []
            prev = before[-1:] 
            if qa[:1].islower() and prev and (prev.isalnum() or prev in ",(-"):
                bad.append("start")
            # end: next text (after any punctuation the quote already has) continues
            rest = after
            while rest[:1] and rest[:1] in TERM | {","} and qa[-1:] not in TERM:
                break
            last = qa[-1:]
            nxt = re.sub(r"^[^A-Za-z0-9]*", "", rest)[:1]
            punct_after = rest[:1]
            if last not in TERM and punct_after not in TERM and nxt.islower():
                bad.append("end")
            if bad:
                out.writerow([rid, page, "+".join(bad), before[-40:], quote[:300], after[:60]])
