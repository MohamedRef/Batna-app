# Brief: Rheumatology and Clinical Immunology content threads (course `rheum`)

Read first: `/mnt/project-files/medstudy/SPLIT.md` (§1–6 rules), `README.md` beside this file,
`/mnt/project-files/medstudy/lessons/BRIEF.md` (lesson style), and in the Batna-app repo `docs/CONTENT.md`,
`docs/LESSONS.md`. Copy the shape of a finished liver lecture (`content/hb/L01.yaml`, `content/lessons/hb/L01.yaml`)
and of a finished gastro lecture (`content/git/`). The `rheum.*` index entries are already on main: branch from `origin/main`.

## The source
- Book chapter 3. Text by printed page: `text/LNN.txt`; page images: `img/pNNN.jpg`.
- Printed pages restart at 1 in chapter 3. `src.page` = printed chapter-3 page. File header: `book: { chapter: 3, pages: [first, last] }`.
- Use the OCR only for tables, figures and image-only pages, and always check it against `img/pNNN.jpg` (OCR garbles numbers, drug names and arrows).
- The book never goes in git. Items paraphrase; `src.quote` ≤ 25 words. The book's answer is the exam answer; guideline differences go in `update:` with name and year.

## What each thread writes (it owns only these files)
- `content/rheum/LNN.yaml`: `batna.content/v1`, ids `rheum.L01.001`…, every item `status: unverified`. Same density as the liver
  lectures (≈1 card/cloze per 45 words, 1 vignette MCQ per 150 words with a reason per distractor, 2–5 cases, every table as `table`
  or clozes, every end-of-lecture question as an `essay` with `tags: [book-question]`).
- `content/lessons/rheum/LNN.yaml`: Egyptian-Arabic interactive lessons (3–7 per lecture, book order, every page covered): why it matters,
  mnemonics, light jokes, drawn SVG (algorithms, anatomy, scales), question steps, a `try` step and a `practice` list. Numbered lists
  (`1. `) render as lists. Ideas for this course: cytokines as a who-makes-what-acts-on-what map, Gell–Coombs types as four cards, the complement pathways as three roads into C3, vasculitis by vessel size (Chapel Hill) drawn as vessels, ACR/EULAR criteria as playable point counters (calc only if a calculator exists; else tables), DMARD/biologic targets as a menu, SLE autoantibodies as a matching game.
- Calculators: use an existing one (`docs/CALCULATORS.md`) as a `calc` item. If a score needs a new calculator, ask the thread
  "Plan Batna for the whole MSc" with the book page and the full scoring table; write it as a `table` meanwhile.
- Figures: a crop of the book's own figure may go in `content/lessons/img/rheum/` with `book: true` and the printed page (docs/LESSONS.md § Images). No generated images.
- Page refs: `cargo run -p batna-tools -- refs --lecture rheum.LNN --write` reads `courses/rheum/text/` and writes `content/book-refs/rheum/LNN.tsv`.
  Hand checks go in `courses/rheum/verdicts/LNN.tsv`. Every chip carries a quote.
- `changes/rheum-LNN.yaml` (one learner-visible line per lecture). Book problems: append to `medstudy/book-issues.md` under a "Rheumatology and Clinical Immunology (chapter 3)" heading.
- Then `just check` (incl. `batna-tools validate`), play the lessons in the desktop app (lessons BRIEF), open a PR with base `main`, get CI
  green and squash-merge yourself. Finish and merge one lecture before starting the next, so Mohamed can study it right away. Rebuild the
  Anki file after each merge (`cargo run -q -p batna-tools -- anki --out /mnt/project-files/medstudy/anki/`).
- Reply to Mohamed in 3–4 short Egyptian-Arabic lines with a Markdown table of counts.

App changes (course switcher, per-course pages, Anki per course, new calculators) belong to the thread "Plan Batna for the whole MSc". Ask it rather than editing app code.

## Split
| Thread | Chunk | Lectures | Printed pages |
|---|---|---|---:|
| A | 55 | rheum.L01 Cytokines · rheum.L02 Gout and pseudogout · rheum.L03 Hypersensitivity reactions | 67 |
| B | 56 | rheum.L04 Antiphospholipid syndrome · rheum.L05 Autoimmunity · rheum.L06 Basic immunology · rheum.L07 Complement system | 55 |
| C | 57 | rheum.L08 Vasculitis: an approach · rheum.L09 Idiopathic inflammatory myopathies · rheum.L10 Immune-based therapy · rheum.L11 Immune deficiency disorders | 74 |
| D | 58 | rheum.L12 Juvenile idiopathic arthritis · rheum.L13 Musculoskeletal ultrasound and nailfold capillaroscopy · rheum.L14 Osteoporosis · rheum.L15 Osteoarthritis update | 44 |
| E | 59 | rheum.L16 Rheumatoid arthritis · rheum.L17 Seronegative spondyloarthritis I | 60 |
| F | 60 | rheum.L18 Sjögren syndrome · rheum.L19 Systemic lupus erythematosus · rheum.L20 Seronegative spondyloarthritis II: psoriatic arthritis · rheum.L21 Systemic sclerosis | 53 |

Overlaps: the earlier lecture owns a shared fact and the later one links to it. Across courses, keep the facts consistent with the course that owns the topic (check `content/` on main before writing a shared fact).
