#!/usr/bin/env python3
"""Crop one book figure for a lesson (#983, docs/LESSONS.md § Book figures).

    python3 -I scripts/content-tools/figure-crop.py hb 3 --list
    python3 -I scripts/content-tools/figure-crop.py hb 3 content/lessons/img/hb/L01-stellate-cells.webp --image 0
    python3 -I scripts/content-tools/figure-crop.py hb 3 out.webp --box 0.30,0.60,0.70,0.80

`--list` prints the pictures embedded on the printed page with their boxes as
fractions of the page (x0,y0,x1,y1, top-left origin), plus the page's size.
`--image N` crops picture N of that list, `--box` any region (use it for
drawn diagrams, which have no embedded picture, and to add the labels a
picture carries around it). `--pad` widens the box (fraction, default 0.006).
Several `--image` values crop the box around all of them (a figure in panels).

The page is rendered at 300 dpi from the book PDF in the private sources
(BATNA_SOURCES), cropped, scaled to at most 800 px wide and saved as WebP
(quality lowered until it is under 150 KB). Look at the result before using
it: the crop has the figure's panel letters and arrows, not the printed
caption under it.
"""
import argparse
import io
import os
import re
import sys

import pdfplumber
import pypdfium2 as pdfium

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
MAX_W, MAX_KB = 800, 150


def pdf_page(course, printed):
    """The PDF page number of a printed page, from the text layer headers."""
    if course == "hb":
        texts = [os.path.join(SOURCES, "book", f) for f in os.listdir(os.path.join(SOURCES, "book")) if re.match(r"L\d+\.txt$", f)]
    else:
        d = os.path.join(SOURCES, "courses", course, "text")
        texts = [os.path.join(d, f) for f in os.listdir(d) if f.endswith(".txt")]
    pat = re.compile(rf"^=== book page {printed} \(PDF page (\d+)\) ===$", re.M)
    for t in texts:
        m = pat.search(open(t, encoding="utf-8").read())
        if m:
            return int(m.group(1))
    sys.exit(f"printed page {printed} of {course} not found in the text layer")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("course")
    ap.add_argument("page", type=int, help="printed page")
    ap.add_argument("out", nargs="?")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--image", type=int, action="append", default=[])
    ap.add_argument("--box")
    ap.add_argument("--pad", type=float, default=0.006)
    ap.add_argument("--dpi", type=int, default=300)
    a = ap.parse_args()
    if a.course not in PDF:
        sys.exit(f"no book PDF for {a.course} (research slides: crop from courses/res/slides)")
    path = os.path.join(SOURCES, PDF[a.course])
    n = pdf_page(a.course, a.page)
    with pdfplumber.open(path) as pdf:
        pg = pdf.pages[n - 1]
        W, H = float(pg.width), float(pg.height)
        imgs = [
            (i["x0"] / W, i["top"] / H, i["x1"] / W, i["bottom"] / H)
            for i in pg.images
            if (i["x1"] - i["x0"]) > 40 and (i["bottom"] - i["top"]) > 40
        ]
    if a.list or not a.out:
        print(f"printed p{a.page} = PDF page {n}, {W:.0f}x{H:.0f} pt")
        for k, b in enumerate(imgs):
            print(f"  image {k}: box {b[0]:.3f},{b[1]:.3f},{b[2]:.3f},{b[3]:.3f}")
        return
    if a.box:
        box = tuple(float(v) for v in a.box.split(","))
    elif a.image:
        sel = [imgs[k] for k in a.image]
        box = (min(b[0] for b in sel), min(b[1] for b in sel), max(b[2] for b in sel), max(b[3] for b in sel))
    else:
        sys.exit("give --image N or --box x0,y0,x1,y1")
    box = (max(0, box[0] - a.pad), max(0, box[1] - a.pad), min(1, box[2] + a.pad), min(1, box[3] + a.pad))
    doc = pdfium.PdfDocument(path)
    im = doc[n - 1].render(scale=a.dpi / 72).to_pil().convert("RGB")
    w, h = im.size
    crop = im.crop((round(box[0] * w), round(box[1] * h), round(box[2] * w), round(box[3] * h)))
    if crop.width > MAX_W:
        crop = crop.resize((MAX_W, round(crop.height * MAX_W / crop.width)), resample=3)
    for q in (82, 75, 68, 60, 52, 45):
        buf = io.BytesIO()
        crop.save(buf, "WEBP", quality=q, method=6)
        if buf.tell() <= MAX_KB * 1024:
            break
    os.makedirs(os.path.dirname(os.path.abspath(a.out)), exist_ok=True)
    open(a.out, "wb").write(buf.getvalue())
    print(f"{a.out}: {crop.width}x{crop.height}, {buf.tell() // 1024} KB (quality {q})")


if __name__ == "__main__":
    main()
