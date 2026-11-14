#!/usr/bin/env python3
"""Find the book's figures in one lecture, for its inventory (#983).

    python3 -I scripts/content-tools/figure-candidates.py hem.L03 [--sheets DIR]

Reads the lecture's printed pages from content/index.yaml and, from the
private sources (BATNA_SOURCES, default /mnt/project-files/medstudy):

- the text layer (`book/LNN.txt` for hb, `courses/<c>/text/LNN.txt` for the
  rest) for the PDF page of each printed page and any "Figure"/"Fig." words;
- `pdfimages -list` for the pictures embedded on that PDF page, leaving out
  small ones and the running header images that repeat on most pages;
- with --sheets, contact sheets of the page images (`courses/<c>/img/pNNN.jpg`,
  6 pages a sheet) to look through by eye, because drawn diagrams and tables
  set as vector art have no embedded image.

Prints one line per page: page, PDF page, embedded pictures (w x h), figure
words. It is a worklist, not the inventory: look at every page image before
writing `content/book-figures/<course>/LNN.yaml`. Nothing it writes goes in git.
"""
import collections
import os
import re
import subprocess
import sys

import yaml

SOURCES = os.environ.get("BATNA_SOURCES", "/mnt/project-files/medstudy")
PDF = {
    "hb": "book/internal-medicine-pg-part1.pdf",
    "git": "book/internal-medicine-pg-part1.pdf",
    "hem": "book/internal-medicine-pg-part2.pdf",
    "rheum": "book/internal-medicine-pg-part2.pdf",
    "endo": "book/internal-medicine-pg-part3.pdf",
    "dm": "book/internal-medicine-pg-part3.pdf",
    "geri": "book/internal-medicine-pg-part4-geriatrics.pdf",
    "neph": "book/internal-medicine-pg-part5-nephrology.pdf",
}
MIN_SIDE = 90  # pixels; smaller pictures are icons, bullets and rules
FIG = re.compile(r"\b(fig(?:ure)?s?\.?\s*\d+[a-z]?)", re.I)


def text_path(course, num):
    if course == "hb":
        return os.path.join(SOURCES, "book", f"{num}.txt")
    return os.path.join(SOURCES, "courses", course, "text", f"{num}.txt")


def pages_of(text):
    """printed page -> (pdf page, page text)."""
    out = {}
    parts = re.split(r"^=== book page (\d+) \(PDF page (\d+)\) ===$", text, flags=re.M)
    for i in range(1, len(parts) - 2, 3):
        out[int(parts[i])] = (int(parts[i + 1]), parts[i + 2])
    return out


def embedded(pdf, first, last):
    """pdf page -> [(w, h, object id)] for real pictures."""
    rows = subprocess.run(
        ["pdfimages", "-list", "-f", str(first), "-l", str(last), pdf],
        capture_output=True, text=True, check=True,
    ).stdout.splitlines()[2:]
    per_page = collections.defaultdict(list)
    for r in rows:
        f = r.split()
        if len(f) < 12 or f[2] != "image":
            continue
        page, w, h, obj = int(f[0]), int(f[3]), int(f[4]), f[10]
        if min(w, h) >= MIN_SIDE:
            per_page[page].append((w, h, obj))
    # Running headers: the same object on more than a third of the pages.
    seen = collections.Counter(o for imgs in per_page.values() for _, _, o in set(imgs))
    span = max(1, last - first + 1)
    common = {o for o, n in seen.items() if n > max(2, span // 3)}
    return {p: [i for i in imgs if i[2] not in common] for p, imgs in per_page.items()}


def sheets(course, pages, out):
    from PIL import Image  # only needed for --sheets

    os.makedirs(out, exist_ok=True)
    img_dir = os.path.join(SOURCES, "courses", course, "img")
    for start in range(0, len(pages), 6):
        group = pages[start:start + 6]
        thumbs = []
        for p in group:
            f = os.path.join(img_dir, f"p{p:03d}.jpg")
            if os.path.exists(f):
                im = Image.open(f)
                im.thumbnail((620, 880))
                thumbs.append((p, im))
        if not thumbs:
            continue
        sheet = Image.new("RGB", (3 * 630, 2 * 890), "white")
        for k, (p, im) in enumerate(thumbs):
            sheet.paste(im, ((k % 3) * 630, (k // 3) * 890))
        name = os.path.join(out, f"p{group[0]:03d}-p{group[-1]:03d}.jpg")
        sheet.save(name, quality=70)
        print("sheet", name)


def research(num, first, last):
    """Research decks: pictures per slide and contact sheets of the slide images."""
    pdf = os.path.join(SOURCES, "courses", "res", "slides", f"res-{num}.pdf")
    pics = embedded(pdf, first, last)
    for p in range(first, last + 1):
        imgs = pics.get(p, [])
        print(f"slide {p}\t{len(imgs)} pic(s) {' '.join(f'{w}x{h}' for w, h, _ in imgs)}")
    if "--sheets" in sys.argv:
        from PIL import Image

        out = sys.argv[sys.argv.index("--sheets") + 1]
        os.makedirs(out, exist_ok=True)
        img_dir = os.path.join(SOURCES, "courses", "res", "img")
        for start in range(first, last + 1, 9):
            group = [p for p in range(start, min(start + 9, last + 1))]
            sheet = Image.new("RGB", (3 * 640, 3 * 490), "white")
            for k, p in enumerate(group):
                f = os.path.join(img_dir, f"{num}-s-{p:02d}.jpg")
                if os.path.exists(f):
                    im = Image.open(f)
                    im.thumbnail((630, 480))
                    sheet.paste(im, ((k % 3) * 640, (k // 3) * 490))
            name = os.path.join(out, f"s{group[0]:03d}-s{group[-1]:03d}.jpg")
            sheet.save(name, quality=70)
            print("sheet", name)


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    lecture = sys.argv[1]
    course, num = lecture.split(".")
    index = yaml.safe_load(open("content/index.yaml"))
    entry = next(e for e in index["lectures"] if e["id"] == lecture)
    first, last = entry["pages"]
    if course == "res":
        return research(num, first, last)
    pages = pages_of(open(text_path(course, num), encoding="utf-8").read())
    pdf_pages = [pages[p][0] for p in range(first, last + 1) if p in pages]
    pics = {}
    if course in PDF and pdf_pages:
        pics = embedded(os.path.join(SOURCES, PDF[course]), min(pdf_pages), max(pdf_pages))
    for p in range(first, last + 1):
        pdf, text = pages.get(p, (None, ""))
        imgs = pics.get(pdf, [])
        words = sorted(set(m.group(1) for m in FIG.finditer(text)))
        ocr = "OCR" if "[OCR of the page image" in text else ""
        print(f"p{p}\tPDF {pdf}\t{len(imgs)} pic(s) {' '.join(f'{w}x{h}' for w, h, _ in imgs)}\t{ocr}\t{'; '.join(words)}")
    if "--sheets" in sys.argv:
        sheets(course, list(range(first, last + 1)), sys.argv[sys.argv.index("--sheets") + 1])


if __name__ == "__main__":
    main()
