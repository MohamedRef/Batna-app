# Course `neph`: Nephrology (book chapter 8)

Source: `medstudy/book/internal-medicine-pg-part5-nephrology.pdf (part 5: chapter 8, MSc)`, Alexandria Faculty of Medicine, Internal Medicine Postgraduate Lectures.
Chapter 8's printed page numbers restart at 1. `src.page` is always the printed chapter-8 page (the number in each page's footer).
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
| neph.L01 | Acid–base disorders | 1–22 | 22 | 6, 14, 15, 16, 17, 18, 19 | 73 (A) |
| neph.L02 | Chronic kidney disease | 23–67 | 45 | 28, 40, 43, 47, 51, 52, 56 | 73 (A) |
| neph.L03 | Cardiorenal syndrome | 68–79 | 12 | 72 | 74 (B) |
| neph.L04 | Hypertension | 80–97 | 18 | 92, 93, 97 | 74 (B) |
| neph.L05 | Acute kidney injury | 98–126 | 29 | 104, 109, 118, 119, 124 | 74 (B) |
| neph.L06 | Electrolyte disturbances | 127–144 | 18 | 130 | 75 (C) |
| neph.L07 | Autosomal dominant polycystic kidney disease | 145–197 | 53 | 168 | 75 (C) |
| neph.L08 | Renal clinico-pathological correlation | 198–291 | 94 | 205, 218, 229, 252, 257, 280, 286 | 76 (D) |
| neph.L09 | Renal replacement therapy | 292–336 | 45 | 306, 332 | 77 (E) |
| neph.L10 | Tubulointerstitial nephritis | 337–351 | 15 | – | 77 (E) |
| neph.L11 | Pregnancy and the kidney | 352–409 | 58 | – | 78 (F) |
| neph.L12 | Onconephrology | 410–424 | 15 | – | 79 (G) |
| neph.L13 | Diabetic kidney disease | 425–446 | 22 | 427, 437, 440, 441, 442, 443, 444 | 79 (G) |
| neph.L14 | Kidney in systemic diseases | 447–503 | 57 | 450, 467, 468, 469, 498, 499, 503 | 79 (G) |
