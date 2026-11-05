# Brief: Diabetes, Metabolism and Lipidology content threads (course `dm`)

Read first: `/mnt/project-files/medstudy/SPLIT.md` (§1–6 rules), `README.md` beside this file,
`/mnt/project-files/medstudy/lessons/BRIEF.md` (lesson style), and in the Batna-app repo `docs/CONTENT.md`,
`docs/LESSONS.md`. Copy the shape of a finished liver lecture (`content/hb/L01.yaml`, `content/lessons/hb/L01.yaml`)
and of a finished gastro lecture (`content/git/`). The `dm.*` index entries are already on main: branch from `origin/main`.

## The source
- Book chapter 6. Text by printed page: `text/LNN.txt`; page images: `img/pNNN.jpg`.
- Printed pages restart at 1 in chapter 6. `src.page` = printed chapter-6 page. File header: `book: { chapter: 6, pages: [first, last] }`.
- Use the OCR only for tables, figures and image-only pages, and always check it against `img/pNNN.jpg` (OCR garbles numbers, drug names and arrows).
- The book never goes in git. Items paraphrase; `src.quote` ≤ 25 words. The book's answer is the exam answer; guideline differences go in `update:` with name and year.

## What each thread writes (it owns only these files)
- `content/dm/LNN.yaml`: `batna.content/v1`, ids `dm.L01.001`…, every item `status: unverified`. Same density as the liver
  lectures (≈1 card/cloze per 45 words, 1 vignette MCQ per 150 words with a reason per distractor, 2–5 cases, every table as `table`
  or clozes, every end-of-lecture question as an `essay` with `tags: [book-question]`).
- `content/lessons/dm/LNN.yaml`: Egyptian-Arabic interactive lessons (3–7 per lecture, book order, every page covered): why it matters,
  mnemonics, light jokes, drawn SVG (algorithms, anatomy, scales), question steps, a `try` step and a `practice` list. Numbered lists
  (`1. `) render as lists. Ideas for this course: diagnostic cut-offs as a number ladder, the ADA treatment algorithm as a decision tree, insulin start/titration as a playable schedule, DKA vs HHS as a side-by-side, the hypoglycaemia levels, foot risk classification as a sorting game, lipoprotein classes drawn by size and density, statin intensity tables.
- Calculators: use an existing one (`docs/CALCULATORS.md`) as a `calc` item. If a score needs a new calculator, ask the thread
  "Plan Batna for the whole MSc" with the book page and the full scoring table; write it as a `table` meanwhile.
- Figures: a crop of the book's own figure may go in `content/lessons/img/dm/` with `book: true` and the printed page (docs/LESSONS.md § Images). No generated images.
- Page refs: `cargo run -p batna-tools -- refs --lecture dm.LNN --write` reads `courses/dm/text/` and writes `content/book-refs/dm/LNN.tsv`.
  Hand checks go in `courses/dm/verdicts/LNN.tsv`. Every chip carries a quote.
- `changes/dm-LNN.yaml` (one learner-visible line per lecture). Book problems: append to `medstudy/book-issues.md` under a "Diabetes, Metabolism and Lipidology (chapter 6)" heading.
- Then `just check` (incl. `batna-tools validate`), play the lessons in the desktop app (lessons BRIEF), open a PR with base `main`, get CI
  green and squash-merge yourself. Finish and merge one lecture before starting the next, so Mohamed can study it right away. Rebuild the
  Anki file after each merge (`cargo run -q -p batna-tools -- anki --out /mnt/project-files/medstudy/anki/`).
- Reply to Mohamed in 3–4 short Egyptian-Arabic lines with a Markdown table of counts.

App changes (course switcher, per-course pages, Anki per course, new calculators) belong to the thread "Plan Batna for the whole MSc". Ask it rather than editing app code.

## Split
| Thread | Chunk | Lectures | Printed pages |
|---|---|---|---:|
| A | 63 | dm.L01 Diabetes: classification, pathogenesis and diagnosis · dm.L02 ADA guidelines · dm.L03 Chronic microvascular complications · dm.L04 Cardiovascular risk reduction | 50 |
| B | 64 | dm.L05 Diabetic foot · dm.L06 Diabetic foot disease · dm.L07 Cardiac autonomic neuropathy · dm.L08 Medical nutrition therapy | 52 |
| C | 65 | dm.L09 Oral antidiabetic drugs · dm.L10 Injectable therapy · dm.L11 Starting and intensifying insulin in type 2 DM · dm.L12 Diabetes with CKD and chronic liver disease | 58 |
| D | 66 | dm.L13 Diabetes and pregnancy · dm.L14 Hypoglycemia · dm.L15 Diabetic emergencies · dm.L16 Diabetic emergencies: case reports | 53 |
| E | 67 | dm.L17 Calcium metabolism and osteoporosis · dm.L18 Uric acid metabolism and gout | 32 |
| F | 68 | dm.L19 Plasma lipoproteins · dm.L20 HDL-C therapy · dm.L21 Classification of hyperlipidemia · dm.L22 Diabetic dyslipidemia · dm.L23 Dyslipidemia management: what's new | 55 |

Overlaps: the earlier lecture owns a shared fact and the later one links to it. Across courses, keep the facts consistent with the course that owns the topic (check `content/` on main before writing a shared fact).
