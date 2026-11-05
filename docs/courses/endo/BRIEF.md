# Brief: Endocrinology content threads (course `endo`)

Read first: `/mnt/project-files/medstudy/SPLIT.md` (§1–6 rules), `README.md` beside this file,
`/mnt/project-files/medstudy/lessons/BRIEF.md` (lesson style), and in the Batna-app repo `docs/CONTENT.md`,
`docs/LESSONS.md`. Copy the shape of a finished liver lecture (`content/hb/L01.yaml`, `content/lessons/hb/L01.yaml`)
and of a finished gastro lecture (`content/git/`). The `endo.*` index entries are already on main: branch from `origin/main`.

## The source
- Book chapter 5. Text by printed page: `text/LNN.txt`; page images: `img/pNNN.jpg`.
- Printed pages restart at 1 in chapter 5. `src.page` = printed chapter-5 page. File header: `book: { chapter: 5, pages: [first, last] }`.
- Use the OCR only for tables, figures and image-only pages, and always check it against `img/pNNN.jpg` (OCR garbles numbers, drug names and arrows).
- The book never goes in git. Items paraphrase; `src.quote` ≤ 25 words. The book's answer is the exam answer; guideline differences go in `update:` with name and year.

## What each thread writes (it owns only these files)
- `content/endo/LNN.yaml`: `batna.content/v1`, ids `endo.L01.001`…, every item `status: unverified`. Same density as the liver
  lectures (≈1 card/cloze per 45 words, 1 vignette MCQ per 150 words with a reason per distractor, 2–5 cases, every table as `table`
  or clozes, every end-of-lecture question as an `essay` with `tags: [book-question]`).
- `content/lessons/endo/LNN.yaml`: Egyptian-Arabic interactive lessons (3–7 per lecture, book order, every page covered): why it matters,
  mnemonics, light jokes, drawn SVG (algorithms, anatomy, scales), question steps, a `try` step and a `practice` list. Numbered lists
  (`1. `) render as lists. Ideas for this course: MEN 1 / 2A / 2B as three "P" cards, thyroid tests in pregnancy as trimester ranges, paraneoplastic hormones by tumour, NET markers and grading, the pitfalls in TFTs as "spot the trap" cases, endocrine emergencies (thyroid storm, myxoedema coma, adrenal crisis, pituitary apoplexy) as timed algorithms.
- Calculators: use an existing one (`docs/CALCULATORS.md`) as a `calc` item. If a score needs a new calculator, ask the thread
  "Plan Batna for the whole MSc" with the book page and the full scoring table; write it as a `table` meanwhile.
- Figures: a crop of the book's own figure may go in `content/lessons/img/endo/` with `book: true` and the printed page (docs/LESSONS.md § Images). No generated images.
- Page refs: `cargo run -p batna-tools -- refs --lecture endo.LNN --write` reads `courses/endo/text/` and writes `content/book-refs/endo/LNN.tsv`.
  Hand checks go in `courses/endo/verdicts/LNN.tsv`. Every chip carries a quote.
- `changes/endo-LNN.yaml` (one learner-visible line per lecture). Book problems: append to `medstudy/book-issues.md` under a "Endocrinology (chapter 5)" heading.
- Then `just check` (incl. `batna-tools validate`), play the lessons in the desktop app (lessons BRIEF), open a PR with base `main`, get CI
  green and squash-merge yourself. Finish and merge one lecture before starting the next, so Mohamed can study it right away. Rebuild the
  Anki file after each merge (`cargo run -q -p batna-tools -- anki --out /mnt/project-files/medstudy/anki/`).
- Reply to Mohamed in 3–4 short Egyptian-Arabic lines with a Markdown table of counts.

App changes (course switcher, per-course pages, Anki per course, new calculators) belong to the thread "Plan Batna for the whole MSc". Ask it rather than editing app code.

## Split
| Thread | Chunk | Lectures | Printed pages |
|---|---|---|---:|
| A | 61 | endo.L01 Multiple endocrine neoplasia · endo.L02 Endocrine disorders of pregnancy · endo.L03 Paraneoplastic syndromes · endo.L04 Endocrine and systemic disorders | 76 |
| B | 62 | endo.L05 Neuroendocrine tumours · endo.L06 Pitfalls in thyroid function tests · endo.L07 Endocrine emergencies | 62 |

Overlaps: the earlier lecture owns a shared fact and the later one links to it. Across courses, keep the facts consistent with the course that owns the topic (check `content/` on main before writing a shared fact).
