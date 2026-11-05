# Course `geri`: Geriatrics (book chapter 7)

Source: `medstudy/book/internal-medicine-pg-part4-geriatrics.pdf (part 4: chapter 7)`, Alexandria Faculty of Medicine, Internal Medicine Postgraduate Lectures.
Chapter 7's printed page numbers restart at 1. `src.page` is always the printed chapter-7 page (the number in each page's footer).
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
| geri.L01 | Cardiovascular disorders in the elderly | 1–24 | 24 | 22 | 69 (A) |
| geri.L02 | Renal diseases in the elderly | 25–39 | 15 | 27, 32 | 69 (A) |
| geri.L03 | Prescribing for the elderly | 40–53 | 14 | 51 | 69 (A) |
| geri.L04 | Dementia, delirium and depression | 54–83 | 30 | 60, 66 | 70 (B) |
| geri.L05 | Cerebrovascular disorders in the elderly | 84–90 | 7 | – | 70 (B) |
| geri.L06 | Thyroid diseases in the elderly | 91–105 | 15 | – | 70 (B) |
| geri.L07 | Comprehensive geriatric assessment | 106–130 | 25 | 116, 119, 120, 121, 122, 124, 125 | 71 (C) |
| geri.L08 | Musculoskeletal disorders in the elderly | 131–154 | 24 | – | 71 (C) |
| geri.L09 | Immunosenescence and infection | 155–168 | 14 | 158 | 71 (C) |
| geri.L10 | Diffuse parenchymal lung disease in the elderly | 169–187 | 19 | 171 | 72 (D) |
| geri.L11 | Obstructive lung disease in the elderly | 188–209 | 22 | 195 | 72 (D) |
| geri.L12 | Elder abuse | 210–219 | 10 | – | 72 (D) |
| geri.L13 | Palliative care | 220–230 | 11 | – | 72 (D) |
