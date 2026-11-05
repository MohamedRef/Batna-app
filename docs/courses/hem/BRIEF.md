# Brief: Hematology content threads (course `hem`)

Read first: `/mnt/project-files/medstudy/SPLIT.md` (§1–6 rules), `README.md` beside this file,
`/mnt/project-files/medstudy/lessons/BRIEF.md` (lesson style), and in the Batna-app repo `docs/CONTENT.md`,
`docs/LESSONS.md`. Copy the shape of a finished liver lecture (`content/hb/L01.yaml`, `content/lessons/hb/L01.yaml`)
and of a finished gastro lecture (`content/git/`). The `hem.*` index entries are already on main: branch from `origin/main`.

## The source
- Book chapter 4. Text by printed page: `text/LNN.txt`; page images: `img/pNNN.jpg`.
- Printed pages restart at 1 in chapter 4. `src.page` = printed chapter-4 page. File header: `book: { chapter: 4, pages: [first, last] }`.
- Use the OCR only for tables, figures and image-only pages, and always check it against `img/pNNN.jpg` (OCR garbles numbers, drug names and arrows).
- The book never goes in git. Items paraphrase; `src.quote` ≤ 25 words. The book's answer is the exam answer; guideline differences go in `update:` with name and year.

## What each thread writes (it owns only these files)
- `content/hem/LNN.yaml`: `batna.content/v1`, ids `hem.L01.001`…, every item `status: unverified`. Same density as the liver
  lectures (≈1 card/cloze per 45 words, 1 vignette MCQ per 150 words with a reason per distractor, 2–5 cases, every table as `table`
  or clozes, every end-of-lecture question as an `essay` with `tags: [book-question]`).
- `content/lessons/hem/LNN.yaml`: Egyptian-Arabic interactive lessons (3–7 per lecture, book order, every page covered): why it matters,
  mnemonics, light jokes, drawn SVG (algorithms, anatomy, scales), question steps, a `try` step and a `practice` list. Numbered lists
  (`1. `) render as lists. Ideas for this course: the anaemia work-up as a branching algorithm (MCV first), the haemolysis lab panel as a sorting game, thalassaemia types as a ladder, the coagulation cascade drawn as two streams meeting, transfusion reactions as a timeline, the WHO/ELN leukaemia risk tables, MPN driver mutations (JAK2/CALR/MPL) as a matching game, CRAB criteria for myeloma, Ann Arbor/Lugano staging as a body map.
- Calculators: use an existing one (`docs/CALCULATORS.md`) as a `calc` item. If a score needs a new calculator, ask the thread
  "Plan Batna for the whole MSc" with the book page and the full scoring table; write it as a `table` meanwhile.
- Figures: a crop of the book's own figure may go in `content/lessons/img/hem/` with `book: true` and the printed page (docs/LESSONS.md § Images). No generated images.
- Page refs: `cargo run -p batna-tools -- refs --lecture hem.LNN --write` reads `courses/hem/text/` and writes `content/book-refs/hem/LNN.tsv`.
  Hand checks go in `courses/hem/verdicts/LNN.tsv`. Every chip carries a quote.
- `changes/hem-LNN.yaml` (one learner-visible line per lecture). Book problems: append to `medstudy/book-issues.md` under a "Hematology (chapter 4)" heading.
- Then `just check` (incl. `batna-tools validate`), play the lessons in the desktop app (lessons BRIEF), open a PR with base `main`, get CI
  green and squash-merge yourself. Finish and merge one lecture before starting the next, so Mohamed can study it right away. Rebuild the
  Anki file after each merge (`cargo run -q -p batna-tools -- anki --out /mnt/project-files/medstudy/anki/`).
- Reply to Mohamed in 3–4 short Egyptian-Arabic lines with a Markdown table of counts.

App changes (course switcher, per-course pages, Anki per course, new calculators) belong to the thread "Plan Batna for the whole MSc". Ask it rather than editing app code.

## Split
| Thread | Chunk | Lectures | Printed pages |
|---|---|---|---:|
| A | 50 | hem.L01 Hematopoiesis · hem.L02 Anaemia · hem.L03 Hemolytic anemias | 56 |
| B | 51 | hem.L04 Leukocyte disorders · hem.L05 Hemostasis · hem.L06 Blood transfusion | 47 |
| C | 52 | hem.L07 Acute leukemias · hem.L08 Chronic leukemias | 29 |
| D | 53 | hem.L09 Myeloproliferative neoplasms · hem.L10 Polycythemias | 43 |
| E | 54 | hem.L11 Multiple myeloma · hem.L12 Lymphomas | 31 |

Overlaps: the earlier lecture owns a shared fact and the later one links to it. Across courses, keep the facts consistent with the course that owns the topic (check `content/` on main before writing a shared fact).
