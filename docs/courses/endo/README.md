# Course `endo`: Endocrinology (book chapter 5)

Source: `medstudy/book/internal-medicine-pg-part3.pdf (part 3: chapters 5–6)`, Alexandria Faculty of Medicine, Internal Medicine Postgraduate Lectures.
Chapter 5's printed page numbers restart at 1. `src.page` is always the printed chapter-5 page (the number in each page's footer).
Mohamed sent the book on 2026-10-29. The lecture list below comes from the book's own table of contents and the first page of each lecture.

## Files (none of these go in git)
| Path | What |
|---|---|
| `text/LNN.txt` | Text layer per printed page (`=== book page N (PDF page M) ===`). Pages with a picture (tables and figures are often images) also carry the tesseract OCR after `[OCR of the page image ...]` |
| `img/pNNN.jpg` | Page image at 200 dpi, by printed page. Check tables, figures and OCR against these |
| `verdicts/LNN.tsv` | Hand checks for `batna-tools refs` (same format as the res and git verdicts) |

## Lectures
| Id | Lecture | Printed pages | Pages | Image-only pages (OCR only) | Chunk (thread) |
|---|---|---|---:|---|---|
| endo.L01 | Multiple endocrine neoplasia | 1–12 | 12 | – | 61 (A) |
| endo.L02 | Endocrine disorders of pregnancy | 13–30 | 18 | – | 61 (A) |
| endo.L03 | Paraneoplastic syndromes | 31–42 | 12 | – | 61 (A) |
| endo.L04 | Endocrine and systemic disorders | 43–76 | 34 | – | 61 (A) |
| endo.L05 | Neuroendocrine tumours | 77–98 | 22 | – | 62 (B) |
| endo.L06 | Pitfalls in thyroid function tests | 99–110 | 12 | – | 62 (B) |
| endo.L07 | Endocrine emergencies | 111–138 | 28 | – | 62 (B) |
