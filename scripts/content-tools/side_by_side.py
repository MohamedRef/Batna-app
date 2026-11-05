#!/usr/bin/env python3
"""Print each content item next to the book text of its src.page.

Usage: python3 -I side_by_side.py content/hb/L01.yaml /mnt/project-files/medstudy/book/L01.txt [item-id-prefix]
Chunk 19 (accuracy checker) helper. Read-only.
"""
import re, sys, yaml

def pages(book):
    out, cur = {}, None
    for line in open(book, encoding="utf-8"):
        m = re.match(r"=== book page (\d+)", line)
        if m:
            cur = int(m.group(1)); out[cur] = []
        elif cur is not None:
            out[cur].append(line.rstrip())
    return {k: "\n".join(l for l in v if l.strip()) for k, v in out.items()}

def main():
    doc = yaml.safe_load(open(sys.argv[1], encoding="utf-8"))
    bp = pages(sys.argv[2])
    pref = sys.argv[3] if len(sys.argv) > 3 else ""
    for it in doc.get("items", []):
        if not it["id"].startswith(pref):
            continue
        body = {k: v for k, v in it.items() if k not in ("id", "kind", "status", "src", "topic")}
        p = (it.get("src") or {}).get("page")
        print(f"\n##### {it['id']}  [{it['kind']}]  p.{p}  status={it.get('status')}")
        print(yaml.safe_dump(body, allow_unicode=True, sort_keys=False, width=110).rstrip())
        q = (it.get("src") or {}).get("quote")
        if q and p in bp and q.lower().split()[0] not in bp[p].lower():
            print(f"!! quote not obviously on p.{p}: {q}")
        if p not in bp:
            print(f"!! page {p} not in book text")

if __name__ == "__main__":
    main()
