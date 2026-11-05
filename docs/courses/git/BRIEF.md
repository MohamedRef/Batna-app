# Brief: Gastroenterology content threads (course `git`)

Read first: `/mnt/project-files/medstudy/SPLIT.md` (§1–6 rules), `README.md` beside this file,
`/mnt/project-files/medstudy/lessons/BRIEF.md` (lesson style), and in the Batna-app repo `docs/CONTENT.md`,
`docs/LESSONS.md`. Copy the shape of a finished liver lecture: `content/hb/L01.yaml` and
`content/lessons/hb/L01.yaml`. Batna-app PR #406 adds the `git.*` index entries; branch from `origin/main`
after it merges (or branch from it).

## The source
- The same book as the liver, chapter 2. Text by printed page: `text/LNN.txt`; page images: `img/pNNN.jpg`.
- Printed pages restart at 1 in chapter 2 (printed p = PDF p+423). `src.page` = printed chapter-2 page.
  File header: `book: { chapter: 2, pages: [first, last] }`.
- Each page has the text layer, then the OCR of its image. Use the OCR only for tables/figures/image-only
  pages and always check it against `img/pNNN.jpg` (OCR garbles numbers, drug names and arrows).
- The book never goes in git. Items paraphrase; `src.quote` ≤ 25 words. The book's answer is the exam
  answer; guideline differences go in `update:` with name and year.

## What each thread writes (it owns only these files)
- `content/git/LNN.yaml`: `batna.content/v1`, ids `git.L01.001`…, every item `status: unverified`.
  Same density as the liver lectures (≈1 card/cloze per 45 words, 1 vignette MCQ per 150 words with a
  reason per distractor, 2–5 cases, every table as `table` or clozes, every end-of-lecture question as an
  `essay` with `tags: [book-question]`).
- `content/lessons/git/LNN.yaml`: Egyptian-Arabic interactive lessons (3–7 per lecture, book order, every
  page covered): why it matters, mnemonics, light jokes, drawn SVG (algorithms, anatomy, scales), question
  steps, a `try` step and a `practice` list. GI ideas: the GERD → PPI-refractory algorithm, H. pylori
  regimens as a menu, Forrest classification as a ladder, Rome IV as a map, polyp types and polyposis syndromes as a sorting game,
  Montreal classification for IBD, the Chicago classification of achalasia types as manometry sketches,
  the Glasgow-Blatchford score as a playable calculator (`calc`).
- Figures: a crop of the book's own figure may go in `content/lessons/img/git/` with `book: true` and the
  printed page (docs/LESSONS.md § Images). No generated images.
- Page refs: `cargo run -p batna-tools -- refs --lecture git.LNN --write` reads `courses/git/text/` and
  writes `content/book-refs/git/LNN.tsv`. Hand checks (quotes read off a page image, ok verdicts) go in
  `courses/git/verdicts/LNN.tsv`. Every chip carries a quote.
- `changes/git-LNN.yaml` (one learner-visible line per lecture). Book problems: append to
  `medstudy/book-issues.md` under a "Gastroenterology (chapter 2)" heading.
- Then `just check` (incl. `batna-tools validate`), play the lessons in the desktop app (lessons BRIEF),
  open a PR with base `main`, get CI green and squash-merge yourself. Finish and merge one lecture before
  starting the next, so Mohamed can study it right away. Rebuild the Anki file after each merge
  (`cargo run -q -p batna-tools -- anki --out /mnt/project-files/medstudy/anki/`).
- Reply to Mohamed in 3–4 short Egyptian-Arabic lines with a Markdown table of counts.

App changes (course switcher, per-course pages in the book sheet, Anki per course) belong to the thread
"Plan Batna for the whole MSc". Ask it rather than editing app code.

## Split
| Thread | Chunk | Lectures | Printed pages |
|---|---|---|---:|
| A | 40 | git.L01 Acid-related disorders · git.L02 Advanced endoscopic procedures | 75 |
| B | 41 | git.L03 Functional GI disorders (24 image-only pages) | 67 |
| C | 42 | git.L04 GI tumors · git.L05 GI polyps | 63 |
| D | 43 | git.L06 IBD · git.L07 Lower GI bleeding · git.L08 Celiac and co. · git.L09 Malabsorption (only 246–256 new) | 52 new |
| E | 44 | git.L10 Esophageal motility disorders · git.L11 GI bleeding | 44 |

Overlaps to keep consistent: GI bleeding (L07 lower, L11 upper, and hb.L03/L04 variceal bleeding), dysphagia
(L04 esophageal cancer vs L10 motility), GERD (L01 vs L03 functional heartburn vs L10). The earlier lecture
owns a shared fact; the later one links to it.
