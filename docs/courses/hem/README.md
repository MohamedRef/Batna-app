# Course `hem`: Hematology (book chapter 4)

Source: `medstudy/book/internal-medicine-pg-part2.pdf (part 2: chapters 3–4)`, Alexandria Faculty of Medicine, Internal Medicine Postgraduate Lectures.
Chapter 4's printed page numbers restart at 1. `src.page` is always the printed chapter-4 page (the number in each page's footer).
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
| hem.L01 | Hematopoiesis | 1–8 | 8 | 2 | 50 (A) |
| hem.L02 | Anaemia | 9–17 | 9 | – | 50 (A) |
| hem.L03 | Hemolytic anemias | 18–56 | 39 | 41 | 50 (A) |
| hem.L04 | Leukocyte disorders | 57–69 | 13 | 60 | 51 (B) |
| hem.L05 | Hemostasis | 70–83 | 14 | – | 51 (B) |
| hem.L06 | Blood transfusion | 84–103 | 20 | – | 51 (B) |
| hem.L07 | Acute leukemias | 104–115 | 12 | – | 52 (C) |
| hem.L08 | Chronic leukemias | 116–132 | 17 | 127 | 52 (C) |
| hem.L09 | Myeloproliferative neoplasms | 133–155 | 23 | 139, 145, 146 | 53 (D) |
| hem.L10 | Polycythemias | 156–175 | 20 | 163, 169, 175 | 53 (D) |
| hem.L11 | Multiple myeloma | 176–189 | 14 | – | 54 (E) |
| hem.L12 | Lymphomas | 190–206 | 17 | 201, 202 | 54 (E) |
