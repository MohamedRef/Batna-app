# Course `git`: Gastroenterology (book 1, chapter 2)

Source: the same book as Hepatology, `medstudy/book/internal-medicine-pg-part1.pdf`
(Alexandria Faculty of Medicine, Internal Medicine Postgraduate Lectures, e-book 2025, part 1).
Chapter 2 "Gastroenterology" is PDF pages 424–735. Its printed page numbers restart at 1:
**printed page p = PDF page p + 423.** `src.page` is always the printed chapter-2 page.

Found 2026-10-29: Mohamed asked for "the other subject in book 1 besides the liver"
(the coordinator first guessed hematology; the table of contents says gastroenterology).
In his course sheet GIT is a later-semester course (2 credit h), id `git` in `content/program.yaml`.

## Files (none of these go in git)
| Path | What |
|---|---|
| `text/LNN.txt` | Text layer per printed page (`=== book page N (PDF page M) ===`), then the tesseract OCR of that page's image after `[OCR of the page image ...]` |
| `img/pNNN.jpg` | Page image at 200 dpi, by printed page. Check tables, figures and OCR against these |
| `verdicts/LNN.tsv` | Hand checks for `batna-tools refs` (same format as the res verdicts) |

## Lectures
| Id | Lecture | Printed pages | Pages | Image-heavy pages (OCR only) | Chunk |
|---|---|---|---:|---|---|
| git.L01 | Acid-related disorders (GERD, PPI-refractory, H. pylori, PUD, ZES) | 1–39 | 39 | 31 | 40 |
| git.L02 | Advanced endoscopic procedures (enteroscopy, capsule, NBI, EUS, POEM) | 40–75 | 36 | 45–47, 51, 67–68 | 40 |
| git.L03 | Functional GI disorders (Rome IV) | 76–142 | 67 | 24 pages: 86–87, 95, 102–105, 109, 115, 117, 120–124, 126–128, 135, 138–142 | 41 |
| git.L04 | GI tumors | 143–184 | 42 | 149, 165 | 42 |
| git.L05 | GI polyps and polyposis syndromes | 185–205 | 21 | 196, 200–202, 204 | 42 |
| git.L06 | Inflammatory bowel disease | 206–224 | 19 | 219, 221–222 | 43 |
| git.L07 | Lower GI bleeding | 225–233 | 9 | – | 43 |
| git.L08 | Celiac disease, Whipple, bacterial overgrowth, tropical sprue, short bowel | 234–245 | 12 | – | 43 |
| git.L09 | Malabsorption | 246–268 | 23 | 255 | 43 |
| git.L10 | Esophageal motility disorders (achalasia…) | 269–298 | 30 | 297 | 44 |
| git.L11 | Gastrointestinal bleeding (non-variceal and variceal UGIB) | 299–312 | 14 | 309–311 | 44 |

"Image-heavy" = under 150 characters of text layer; the whole page is a picture.

## Book quirks
- **L09 pages 257–268 repeat L08 pages 234–245** almost word for word (celiac, Whipple, bacterial
  overgrowth, tropical sprue, short bowel). L08 owns that material. L09 items cover pages 246–256
  (physiology of absorption, approach to malabsorption) and cite L08 pages for nothing; a L09 lesson
  may recap L08 in one step with a "see L08" link instead of repeating it.
- Several lectures print their end "Questions" twice (text page, then the same page as an image).
