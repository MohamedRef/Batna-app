#!/usr/bin/env python3
"""Re-encode lesson images smaller (Mohamed 2026-11-14: as small as they stay readable).

    python3 -I scripts/content-tools/shrink-images.py [file.webp ...]

With no files, goes through every content/lessons/img/**/*.webp. Each is
re-encoded as WebP at quality 50 and replaced only when that saves at least
15%, so running it again does not keep degrading the same picture. Look at a
few results before committing.
"""
import glob
import io
import os
import sys

from PIL import Image

QUALITY, MIN_SAVING = 50, 0.15


def main():
    files = sys.argv[1:] or sorted(glob.glob("content/lessons/img/**/*.webp", recursive=True))
    before = after = 0
    for f in files:
        size = os.path.getsize(f)
        im = Image.open(f)
        im.load()
        buf = io.BytesIO()
        im.save(buf, "WEBP", quality=QUALITY, method=6)
        new = buf.tell()
        before += size
        if new <= size * (1 - MIN_SAVING):
            open(f, "wb").write(buf.getvalue())
            after += new
            print(f"{f}: {size // 1024} -> {new // 1024} KB")
        else:
            after += size
    print(f"{len(files)} file(s): {before // 1024} -> {after // 1024} KB")


if __name__ == "__main__":
    main()
