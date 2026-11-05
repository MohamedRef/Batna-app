# Brief: Basics of Research content threads (course `res`)

Read first: `/mnt/project-files/medstudy/SPLIT.md` (§1–5 rules), `docs/CONTENT.md` and `docs/LESSONS.md` in the
Batna-app repo, `/mnt/project-files/medstudy/lessons/BRIEF.md` (lesson style), and `README.md` beside this file.
Batna-app PR #399 adds `content/program.yaml`, the `res.*` index entries and slide-aware `refs`.
Branch from `origin/main` after it merges.

## The source: slide decks, not a book
- Slides: `slides/res-LNN.pdf`. Text: `text/LNN.txt`, one `=== slide N ===` block per slide. Image-only slides
  were OCR'd and say `[OCR from slide image; check against the image]`. Always check those against
  `img/LNN-s-NN.jpg`, because OCR garbles formulas.
- None of it goes in git: no slide text, no PDF. Items paraphrase, and `src.quote` holds at most 25 words.
- **`src.page` is the slide number.** The index `pages` for res.LNN is `[1, <slide count>]`.
- The exam answer is the lecturer's slide. A textbook difference goes in `update:` with its source.

## What each thread writes (it owns only these files)
- `content/res/LNN.yaml`: the same `batna.content/v1` schema as hepatology, with ids `res.L01.001`… The
  research-friendly kinds are `mcq` (study-design and measure "which is it?" stems), `card`, `cloze`, `order`
  (research process steps), `table` (design comparisons), `case` (a mini study scenario → design, bias,
  measure) and `calc` (incidence, prevalence, attack rate, RR, OR, AR, NNT; compute from the slide formulas).
- `content/lessons/res/LNN.yaml`: interactive Egyptian-Arabic lessons, the same style as the liver ones:
  why it matters, mnemonics, jokes, drawn SVG, check steps. Research is abstract, so lean on concrete
  Egyptian clinic or ward examples ("عايز تعرف الشيشة بتعمل cancer ولا لأ...").
- Figures: a crop of the slide's own diagram may go in `content/lessons/img/res/` with `book: true` and the
  slide number, the same rule as book figures. Text never goes in.
- Then run `cargo run -p batna-tools -- refs --lecture res.LNN --write`. It reads `courses/res/text/` and
  writes `content/book-refs/res/LNN.tsv`. Hand checks go in `courses/res/verdicts/LNN.tsv`.
- Every item is `status: unverified`. Add `changes/res-LNN.yaml`. Run `just check` and green CI, then
  squash-merge yourself.

## Split
| Thread | Lectures | Chunk | Slides |
|---|---|---|---|
| A | res.L01 Introduction to health research · res.L02 Measures of disease frequency | 30 | 53 + 71 |
| B | res.L03 Research question, objectives, hypotheses | 31 | 85 |
| C | res.L04 Descriptive studies · res.L05 Analytic studies | 32 | 53 + 74 |
| D | res.L06 Experimental studies 1 (+ L07 and the L08 revision when they arrive) | 33 | 56 |

App changes (the "slide N" label instead of "ص N", research calculators and drills) belong to the thread
"Build Batna for the whole MSc". Ask it rather than editing app code.
