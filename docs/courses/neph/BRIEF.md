# Brief: Nephrology content threads (course `neph`)

Read first: `/mnt/project-files/medstudy/SPLIT.md` (§1–6 rules), `README.md` beside this file,
`/mnt/project-files/medstudy/lessons/BRIEF.md` (lesson style), and in the Batna-app repo `docs/CONTENT.md`,
`docs/LESSONS.md`. Copy the shape of a finished liver lecture (`content/hb/L01.yaml`, `content/lessons/hb/L01.yaml`)
and of a finished gastro lecture (`content/git/`). The `neph.*` index entries are already on main: branch from `origin/main`.

## The source
- Book chapter 8. Text by printed page: `text/LNN.txt`; page images: `img/pNNN.jpg`.
- Printed pages restart at 1 in chapter 8. `src.page` = printed chapter-8 page. File header: `book: { chapter: 8, pages: [first, last] }`.
- Use the OCR only for tables, figures and image-only pages, and always check it against `img/pNNN.jpg` (OCR garbles numbers, drug names and arrows).
- The book never goes in git. Items paraphrase; `src.quote` ≤ 25 words. The book's answer is the exam answer; guideline differences go in `update:` with name and year.

## What each thread writes (it owns only these files)
- `content/neph/LNN.yaml`: `batna.content/v1`, ids `neph.L01.001`…, every item `status: unverified`. Same density as the liver
  lectures (≈1 card/cloze per 45 words, 1 vignette MCQ per 150 words with a reason per distractor, 2–5 cases, every table as `table`
  or clozes, every end-of-lecture question as an `essay` with `tags: [book-question]`).
- `content/lessons/neph/LNN.yaml`: Egyptian-Arabic interactive lessons (3–7 per lecture, book order, every page covered): why it matters,
  mnemonics, light jokes, drawn SVG (algorithms, anatomy, scales), question steps, a `try` step and a `practice` list. Numbered lists
  (`1. `) render as lists. Ideas for this course: the acid–base six steps as a playable stepper (anion gap, delta ratio, Winter formula), KDIGO CKD G/A heat map, AKI stages, electrolyte correction formulas (only with existing calculators; ask for new ones), dialysis modalities compared, glomerular disease patterns (nephritic vs nephrotic) as a sorting game, pregnancy-related kidney disease by trimester.
- Calculators: use an existing one (`docs/CALCULATORS.md`) as a `calc` item. If a score needs a new calculator, ask the thread
  "Plan Batna for the whole MSc" with the book page and the full scoring table; write it as a `table` meanwhile.
- Figures: a crop of the book's own figure may go in `content/lessons/img/neph/` with `book: true` and the printed page (docs/LESSONS.md § Images). No generated images.
- Page refs: `cargo run -p batna-tools -- refs --lecture neph.LNN --write` reads `courses/neph/text/` and writes `content/book-refs/neph/LNN.tsv`.
  Hand checks go in `courses/neph/verdicts/LNN.tsv`. Every chip carries a quote.
- `changes/neph-LNN.yaml` (one learner-visible line per lecture). Book problems: append to `medstudy/book-issues.md` under a "Nephrology (chapter 8)" heading.
- Then `just check` (incl. `batna-tools validate`), play the lessons in the desktop app (lessons BRIEF), open a PR with base `main`, get CI
  green and squash-merge yourself. Finish and merge one lecture before starting the next, so Mohamed can study it right away. Rebuild the
  Anki file after each merge (`cargo run -q -p batna-tools -- anki --out /mnt/project-files/medstudy/anki/`).
- Reply to Mohamed in 3–4 short Egyptian-Arabic lines with a Markdown table of counts.

App changes (course switcher, per-course pages, Anki per course, new calculators) belong to the thread "Plan Batna for the whole MSc". Ask it rather than editing app code.

## Split
| Thread | Chunk | Lectures | Printed pages |
|---|---|---|---:|
| A | 73 | neph.L01 Acid–base disorders · neph.L02 Chronic kidney disease | 67 |
| B | 74 | neph.L03 Cardiorenal syndrome · neph.L04 Hypertension · neph.L05 Acute kidney injury | 59 |
| C | 75 | neph.L06 Electrolyte disturbances · neph.L07 Autosomal dominant polycystic kidney disease | 71 |
| D | 76 | neph.L08 Renal clinico-pathological correlation | 94 |
| E | 77 | neph.L09 Renal replacement therapy · neph.L10 Tubulointerstitial nephritis | 60 |
| F | 78 | neph.L11 Pregnancy and the kidney | 58 |
| G | 79 | neph.L12 Onconephrology · neph.L13 Diabetic kidney disease · neph.L14 Kidney in systemic diseases | 94 |

Overlaps: the earlier lecture owns a shared fact and the later one links to it. Across courses, keep the facts consistent with the course that owns the topic (check `content/` on main before writing a shared fact).
