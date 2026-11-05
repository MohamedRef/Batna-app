# Course `dm`: Diabetes, Metabolism and Lipidology (book chapter 6)

Source: `medstudy/book/internal-medicine-pg-part3.pdf (part 3: chapters 5–6)`, Alexandria Faculty of Medicine, Internal Medicine Postgraduate Lectures.
Chapter 6's printed page numbers restart at 1. `src.page` is always the printed chapter-6 page (the number in each page's footer).
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
| dm.L01 | Diabetes: classification, pathogenesis and diagnosis | 1–17 | 17 | – | 63 (A) |
| dm.L02 | ADA guidelines | 18–26 | 9 | 24 | 63 (A) |
| dm.L03 | Chronic microvascular complications | 27–36 | 10 | – | 63 (A) |
| dm.L04 | Cardiovascular risk reduction | 37–50 | 14 | 39, 43 | 63 (A) |
| dm.L05 | Diabetic foot | 51–65 | 15 | – | 64 (B) |
| dm.L06 | Diabetic foot disease | 66–75 | 10 | – | 64 (B) |
| dm.L07 | Cardiac autonomic neuropathy | 76–88 | 13 | – | 64 (B) |
| dm.L08 | Medical nutrition therapy | 89–102 | 14 | 97, 98 | 64 (B) |
| dm.L09 | Oral antidiabetic drugs | 103–124 | 22 | – | 65 (C) |
| dm.L10 | Injectable therapy | 125–129 | 5 | – | 65 (C) |
| dm.L11 | Starting and intensifying insulin in type 2 DM | 130–141 | 12 | – | 65 (C) |
| dm.L12 | Diabetes with CKD and chronic liver disease | 142–160 | 19 | – | 65 (C) |
| dm.L13 | Diabetes and pregnancy | 161–178 | 18 | – | 66 (D) |
| dm.L14 | Hypoglycemia | 179–184 | 6 | – | 66 (D) |
| dm.L15 | Diabetic emergencies | 185–195 | 11 | – | 66 (D) |
| dm.L16 | Diabetic emergencies: case reports | 196–213 | 18 | – | 66 (D) |
| dm.L17 | Calcium metabolism and osteoporosis | 214–232 | 19 | – | 67 (E) |
| dm.L18 | Uric acid metabolism and gout | 233–245 | 13 | – | 67 (E) |
| dm.L19 | Plasma lipoproteins | 246–251 | 6 | – | 68 (F) |
| dm.L20 | HDL-C therapy | 252–259 | 8 | – | 68 (F) |
| dm.L21 | Classification of hyperlipidemia | 260–270 | 11 | – | 68 (F) |
| dm.L22 | Diabetic dyslipidemia | 271–282 | 12 | – | 68 (F) |
| dm.L23 | Dyslipidemia management: what's new | 283–300 | 18 | 284, 285, 292, 293 | 68 (F) |

## Book quirks
- Part 1 is diabetology (lectures 1–18), part 2 lipidology (its lectures 1–5 are `dm.L19`–`dm.L23`). A divider page (PDF 389) has no number.
- The printed numbers skip 281–282: page 280 is followed by 283, where lipidology lecture 5 starts.
