# Course `rheum`: Rheumatology and Clinical Immunology (book chapter 3)

Source: `medstudy/book/internal-medicine-pg-part2.pdf (part 2: chapters 3–4)`, Alexandria Faculty of Medicine, Internal Medicine Postgraduate Lectures.
Chapter 3's printed page numbers restart at 1. `src.page` is always the printed chapter-3 page (the number in each page's footer).
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
| rheum.L01 | Cytokines | 1–22 | 22 | – | 55 (A) |
| rheum.L02 | Gout and pseudogout | 23–51 | 29 | – | 55 (A) |
| rheum.L03 | Hypersensitivity reactions | 52–67 | 16 | 57, 62, 64 | 55 (A) |
| rheum.L04 | Antiphospholipid syndrome | 68–76 | 9 | – | 56 (B) |
| rheum.L05 | Autoimmunity | 77–82 | 6 | – | 56 (B) |
| rheum.L06 | Basic immunology | 83–104 | 22 | 84, 99, 102 | 56 (B) |
| rheum.L07 | Complement system | 105–122 | 18 | 108, 110, 113, 118 | 56 (B) |
| rheum.L08 | Vasculitis: an approach | 123–163 | 41 | 143, 144 | 57 (C) |
| rheum.L09 | Idiopathic inflammatory myopathies | 164–172 | 9 | – | 57 (C) |
| rheum.L10 | Immune-based therapy | 173–179 | 7 | – | 57 (C) |
| rheum.L11 | Immune deficiency disorders | 180–196 | 17 | – | 57 (C) |
| rheum.L12 | Juvenile idiopathic arthritis | 197–210 | 14 | – | 58 (D) |
| rheum.L13 | Musculoskeletal ultrasound and nailfold capillaroscopy | 211–220 | 10 | – | 58 (D) |
| rheum.L14 | Osteoporosis | 221–232 | 12 | – | 58 (D) |
| rheum.L15 | Osteoarthritis update | 233–240 | 8 | – | 58 (D) |
| rheum.L16 | Rheumatoid arthritis | 241–278 | 38 | 248, 256, 261, 267 | 59 (E) |
| rheum.L17 | Seronegative spondyloarthritis I | 279–300 | 22 | – | 59 (E) |
| rheum.L18 | Sjögren syndrome | 301–308 | 8 | – | 60 (F) |
| rheum.L19 | Systemic lupus erythematosus | 309–325 | 17 | – | 60 (F) |
| rheum.L20 | Seronegative spondyloarthritis II: psoriatic arthritis | 326–344 | 19 | – | 60 (F) |
| rheum.L21 | Systemic sclerosis | 345–353 | 9 | – | 60 (F) |
