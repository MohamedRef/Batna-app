import sys, re, os, yaml, csv
repo = sys.argv[1]; courses = sys.argv[2:]
apply = "--apply" in courses; courses = [c for c in courses if not c.startswith("--")]
g = yaml.safe_load(open(f"{repo}/content/glossary.yaml"))
ab = {}
for e in g["entries"]:
    if e.get("kind") == "abbr":
        for t in [e["term"]] + (e.get("aliases") or []):
            ab[t] = e["text"]
NOEXP = {"GIV","G2A1","A3F3","NS5B","RII","XXX","DRβ","NY","DRβ71","IU","CO2","NO","DNA","RNA","mRNA","QT","pH","IV","II","III","VI","VII","VIII","IX","XI","XII","US","O2","H2O","NaCl","KCl"}
KEEP = {"text", "why", "caption"}
SKIP = {"quote", "svg", "id", "kind", "status", "items", "topics", "pages", "page", "alt"}
def strings(x, k=None):
    if isinstance(x, str):
        if k in KEEP: yield x
    elif isinstance(x, list):
        for y in x: yield from strings(y, k)
    elif isinstance(x, dict):
        for kk, v in x.items():
            if kk not in SKIP: yield from strings(v, kk)
words = lambda s: set(re.findall(r"[a-z]{3,}", s.lower()))
pat = re.compile(r"(?<![A-Za-z0-9\-])(" + "|".join(sorted(map(re.escape, ab), key=len, reverse=True)) + r")(?![A-Za-z0-9\-])")
LOWER = set()
for e in g["entries"]:
    for w in re.findall(r"(?<=\s)[a-z][a-z'\-]+", " " + e.get("text", "")):
        LOWER.add(w)
US_SP = [("oesophag","esophag"),("Oesophag","Esophag"),("haem","hem"),("Haem","Hem"),("aemia","emia"),("tumour","tumor"),("Tumour","Tumor"),("ischaem","ischem"),("oestrogen","estrogen"),("faecal","fecal"),("diarrhoea","diarrhea"),("anaesth","anesth"),("colour","color"),("isation","ization"),("apnoea","apnea"),("leukaemia","leukemia"),("uraem","urem"),("oedema","edema"),("paediatr","pediatr"),("leucocyte","leukocyte")]
COURSE = [None]
def expansion0(a):
    t = re.split(r";|, also", ab[a])[0].strip()
    t = re.sub(r"\s*\([^)]*\)", "", t).strip()
    w = t.split(" ")
    f = w[0]
    PROPER = {"Janus", "Mycobacterium", "Wegener's", "Toll-like", "Roux-en-Y", "Simian", "Tegafur–uracil"}
    caps2 = len(w) > 1 and w[1][:1].isupper() and len(w[1]) > 2 and any(ch.islower() for ch in w[1])
    if f == "Von": return "von" + t[3:]
    namedash = re.match(r"^[A-Z][a-z']+–[A-Z]", f)
    if (f[:1].isupper() and f[1:] == f[1:].lower() and not namedash and f not in PROPER
            and not re.match(r"^[A-Za-z]-", f) and not caps2):
        t = t[:1].lower() + t[1:]
    return t
def expansion(a):
    t = expansion0(a)
    if COURSE[0] != "hem":
        for x, y in US_SP: t = t.replace(x, y)
    return t
log = csv.writer(sys.stdout, delimiter="\t")
for c in courses:
    COURSE[0] = c
    d = f"{repo}/content/lessons/{c}"
    for f in sorted(os.listdir(d)):
        path = f"{d}/{f}"
        raw = open(path, encoding="utf-8").read()
        try: doc = yaml.safe_load(raw)
        except Exception: log.writerow([f"{c}/{f}", "-", "SKIP-YAML", ""]); continue
        seen = set(); edits = []
        for ls in doc.get("lessons") or []:
            lpos = raw.find(f"- id: {ls['id']}\n")
            for st in ls.get("steps") or []:
                if st.get("kind") == "cloze": 
                    for s in strings(st):
                        for m in pat.finditer(s): seen.add(m.group(1))
                    continue
                for s in strings(st):
                    for m in pat.finditer(s):
                        a = m.group(1)
                        if a in seen or a in NOEXP: seen.add(a); continue
                        seen.add(a)
                        before, after = s[:m.start()].rstrip("*=_ "), s[m.end():].lstrip("*=_")
                        if before.endswith("(") or before.endswith("/") or after.startswith(" (") or after.startswith("("): continue
                        exp = words(ab[a].split("(")[0]); ctx = words(before[-120:])
                        if exp and len(exp & ctx) >= max(1, len(exp) // 2): continue
                        if "also" in ab[a] or ";" in ab[a]:
                            log.writerow([ls["id"], a, "AMBIGUOUS", s[max(0,m.start()-50):m.end()+30].replace("\n"," ")]); continue
                        if a.lower() in expansion(a).lower() or expansion(a).lower().startswith("the ") or re.match(r"^[A-Z]\d+[A-Z]$", a): continue
                        if a == "ACE" and after.lower().startswith(" inhibitor"): continue
                        # locate in raw: snippet on one line
                        line_start = s.rfind("\n", 0, m.start()) + 1
                        snip = s[max(line_start, m.start() - 30):m.end()]
                        after1 = s[m.end():m.end()+1]
                        pos = raw.find(snip + after1, lpos) if after1 else raw.find(snip, lpos)
                        if pos < 0 or len(snip) < len(a) + 4:
                            log.writerow([ls["id"], a, "NOTFOUND", snip]); continue
                        at = pos + len(snip) - len(a)
                        edits.append((at, a))
                        log.writerow([ls["id"], a, "FIX", expansion(a) + f" ({a})", s[max(0,m.start()-50):m.end()+30].replace("\n"," ")])
        if apply and edits:
            for at, a in sorted(edits, reverse=True):
                assert raw[at:at+len(a)] == a
                raw = raw[:at] + expansion(a) + " (" + a + ")" + raw[at+len(a):]
            open(path, "w", encoding="utf-8").write(raw)
