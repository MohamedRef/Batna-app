# Writing Batna lessons (`batna.lesson/v1`)

Lessons are the teaching layer: Mohamed opens a lesson, understands the
*why*, and only then reviews the lecture's cards. Cards and questions stay in
`content/hb/LNN.yaml`; lessons live beside them in
**`content/lessons/hb/LNN.yaml`**, one file per lecture, and point at the
cards by id.

The model to copy is `content/lessons/hb/L01.yaml`.

## The rules (the validator enforces most of them)

1. **The book is the answer.** Teach only from `/mnt/project-files/medstudy/book/LNN.txt`.
   Every step that teaches a fact gets `page:`: the printed page where that
   fact is (`batna-tools refs` checks it, see docs/CONTENT.md). A `quote:`
   is required (every page chip shows it) and must be that page's own words
   holding the fact. No length cap (Mohamed, 2026-10-29): quote the full
   passage or table, as text with one row per line, never an image. Only a figure step showing the book's own
   figure may go without one. The book text never goes in git.
2. **Guideline differences** go in an `update` step with `ref:` naming the
   guideline and year. Never change the book's answer. Unsure → leave it out.
3. **`status: unverified`** on every lesson. Only Mohamed approves.
4. **Language.** Explanations are Egyptian Arabic (عامية مصرية), warm and
   direct, like a senior resident explaining at 2 am. Medical terms, drug
   names, numbers with units, and anything the exam writes in English stay
   in English: write them inline and the app isolates them left-to-right.
   Case stems, check questions and options may be in English (the exam is).
5. **Teach the why.** For every list or number, give the mechanism or the
   logic that makes it obvious. If the book gives no reason, an accepted
   physiology reason is fine, said plainly; never invent book facts.
6. **Make it stick.** Each lesson should have at least one of: a mnemonic,
   an everyday analogy, a light Egyptian joke (`tone: joke`, kind, never
   about patients' suffering, no politics or religion), an exam trap.
7. **Interactive every 2–3 screens.** A lesson needs at least one question
   step; aim for one every two or three steps. Feedback explains *why*
   each wrong option is wrong.
8. **Short, and as many as the lecture needs** (Mohamed, 2026-11-13). Each
   lesson is 6–10 minutes and 8–16 steps, teaches at most about **25 cards**
   and **two book topics** (headings), and its `pages` are exactly the first
   and last page its steps cite. The number of lessons follows the lecture:
   about **one lesson per four book pages that carry cards**, so a long
   lecture gets more lessons, never thinner ones. They follow the book's
   order and together cover every page. A page with nothing to teach
   (blank, references, a figure already shown, review questions) is listed
   with its reason in `content/lesson-skips.yaml`, never given a filler
   step. When a lesson is split, the first part keeps its id (his done mark
   and place), the new part takes the lecture's next free `.lessonN`, and
   `practice` ids move to the part that teaches them.
9. **Link to the cards.** End with a `try` step (1–5 playable items: card,
   cloze, enumerate, essay, table, order) and list the lecture items the
   lesson teaches in `practice:`.
10. **Every figure of the book, in its place, explained** (Mohamed,
    2026-11-14, #983). Every picture and diagram the book prints (photo,
    histology, imaging, endoscopy, clinical sign, ECG, blood film, diagram,
    algorithm, chart, table printed as a picture) goes into the lesson that
    teaches its page, as a `figure` step with that `page`, **right after the
    step whose text or quote refers to it**. If a split moved that text to
    another lesson, the figure moves with it. Photos and the like are always
    the book's crop (`image: { book: true }`); a diagram or algorithm is the
    crop when it reads at phone width, otherwise a redrawn SVG that keeps the
    book's boxes, order and wording and says which figure it is in its title
    («شكل 3 في الكتاب، ص 9»). The caption explains it **in the lesson's
    context**: what the picture is, what to look at in each panel, arrow and
    label the book prints, and what that proves for the point the lesson is
    making; a figure the cards ask about gets a question step after it. Until
    marks on pictures land (#984), say where to look in words («أعلى
    اليمين»). A figure with nothing to teach (author photo, logo, decoration,
    a repeat of one already shown) is skipped with its reason. Each lecture's
    figures are listed in `content/book-figures/<course>/LNN.yaml` (see
    [Book figures](#book-figures)); `validate` fails when a listed figure is
    neither shown nor skipped. Lessons may still add drawings of their own
    (diagrams, pathways, vicious cycles, algorithms, scales, anatomy
    sketches) as inline SVG, with theme classes (below) or `var(--token)`
    colours; no scripts, links, embedded images, `<style>` or fixed colours.
    Labels in English. Under 24 KB. Every drawn label stays inside its
    picture at phone width (`lesson-figures.yml`, #980).
11. **Rich and easy to read.** Never cut information to make a step
    shorter; add the explanation that makes it click. Readability comes from
    shape: short paragraphs (about 3 lines), bullets for lists,
    `**term**: meaning` rows for definitions. Use `**bold**` for real terms
    only, and mark the 1–3 terms that matter most in the step with
    `==term==` (the marker highlight; the validator allows at most 3).
12. **Every abbreviation is explained.** A step's «اختصارات» chip (beside
    its «ص» chip) lists each abbreviation the step shows, text and figure
    labels alike, with its expansion from `content/glossary.yaml`. So the
    validator fails on any word with two or more capitals (`HRS-AKI`, `NITs`,
    `TIF1γ`) that the glossary doesn't explain. Add an entry (`kind: abbr`,
    the expansion in English, likeliest meaning first, then "also …"), or,
    when the word isn't an abbreviation (emphasis like `NOT`, a Roman
    numeral, a mnemonic's letters, a trial name), add it to `plain:` at the
    end of the glossary. The same entries fill the per-course «قاموس
    الاختصارات» (Lessons screen, More), with the lessons that use each one.
    An abbreviation that means something else in one course gets a second
    entry with `course:` (`{ term: ICP, …, text: "Intracranial pressure",
    course: neph }`): inside that course its chip, dictionary and hover
    show that sense; everywhere else the entry without a course does. Its
    approval id is `glossary.ICP@neph`.

13. **Bring the تحشيشة back.** A step that teaches a list which already has
    a تحشيشة (an enumerate item's `hook`, in this lecture or any other, or a
    `mnemonic` step of another lesson) lists it in `remind:`, so it shows
    under the step and, from another lesson, links back to it:

    ```yaml
    - kind: explain
      remind: [hb.L01.033, hb.L01.034, hb.L01.lesson2:SHIBS]
    ```

    Only on explain, figure, aside, mnemonic or open-table steps (a hook
    under a question gives the answer away). When the same list repeats,
    remind of the first تحشيشة instead of inventing another; partial
    overlap still earns a reminder. `validate` fails on an entry that is not
    an enumerate with a hook or a lesson's mnemonic word.

## File shape

```yaml
schema: batna.lesson/v1
lecture: hb.L01
lessons:
  - id: hb.L01.lesson1          # .lessonN, never reused
    title: "الكبد بيتليف إزاي؟"
    hook: "سطر واحد يخليه عايز يفتح الدرس"
    minutes: 8
    pages: [1, 4]               # inside the lecture's pages
    topics: [definition, causes] # the lecture's topic ids
    goals: ["...", "..."]        # optional, Arabic
    status: unverified
    steps: [...]
    recap: ["...", "..."]        # take-aways on the end screen
    practice: [hb.L01.001, ...]  # items of this lecture
```

## Step kinds

Every step may have `page:` and `quote:`.

| kind | fields | what he sees |
|---|---|---|
| `explain` | `title?`, `text` | Prose. Blank line = new paragraph, `- ` = bullet, `**bold**` = a term (weight only), `==key==` = one of the step's 1–3 key terms (marker). |
| `figure` | `title?`, `svg` or `image`, `caption`, `alt` | A drawing or a picture; tap to zoom. |
| `reveal` | `prompt`, `answer` | "فكّر الأول": think, then tap to see the answer. Use for every "why?". |
| `mnemonic` | `word`, `parts: [{key, text}]`, `hook?` | Letter tiles that open one by one. The keys must spell the word. |
| `aside` | `tone` (tip, exam, trap, analogy, joke), `text` | A coloured box. |
| `update` | `text`, `ref` | Guideline note; says the book wins in the exam. |
| `check` | `question`, `options` (3–5), `answer` (0-based), `why` (one per option) | Single best answer; options are shuffled. |
| `order` | `prompt`, `steps` (3–8, correct order), `why?` | Tap the steps in order. |
| `match` | `prompt`, `pairs` (3–6 `[left, right]`), `why?` | Tap left then right. Right sides must differ. |
| `sort` | `prompt`, `bins` (2–4), `cards` (3–10 `{text, bin}`), `why?` | Drop cards into bins. |
| `case` | `stem`, `steps: [{reveal?, question, options?, answer?, why}]` | A mini case, one decision at a time. Options are shuffled in the app, like `check`. |
| `table` | `title`, `columns`, `rows`, `hide?` (`[row, col]`) | A table with cells to recall. |
| `try` | `items` (1–5 ids) | The lecture's own cards, graded into spaced review. |

## SVG classes

Shapes: `fg-box` (plain box), `fg-ink` (dark outline), `fg-thin` (dashed
grey), `fg-acc` / `fg-good` / `fg-bad` (tinted box with coloured border),
`fg-dot-acc` / `fg-dot-good` / `fg-dot-bad` (solid fill), `fg-arrow`,
`fg-arrow-acc`, `fg-arrow-bad` (lines; add `marker-end="url(#yourId)"`),
`fg-head`, `fg-head-acc`, `fg-head-bad` (arrowhead fills inside `<marker>`).
Text: default is body colour; `t-sm`, `t-lg`, `t-b`, `t-mut`, `t-acc`,
`t-good`, `t-bad`, `t-ar` (Arabic font, right to left). Give marker ids a
lecture prefix (`L03a`). A `stroke-width="N"` attribute overrides the class
width.

SVG `<text>` has no `<bdi>`, so keep Arabic and English in separate `<text>`
elements: brackets or numbers around an English term inside an Arabic
`t-ar` label come out flipped. Lesson text (explain, captions, options) is
isolated for you, so write it naturally there. Arrows don't flip: between
Arabic words write `←` (it points along the reading direction), between
English words `→`.

## Images

A figure takes `image:` instead of `svg:`. The file goes in
**`content/lessons/img/hb/`** (name it `L05-ascites-us.webp`: lecture,
then what it shows). WebP, about 800 px wide, under 150 KB.

From the book (Mohamed allowed book figures in the repo, 2026-10-28; the
book's *text* still never goes in git). The step's `page:` cites it:

```yaml
- kind: figure
  page: 23
  title: "شكل الـ varices في المنظار"
  image: { file: hb/L03-varices-endoscopy.webp, book: true }
  caption: "..."
  alt: "Endoscopy view of three large oesophageal varices with red wale signs."
```

Crop it from `/mnt/project-files/medstudy/book/internal-medicine-pg-part1.pdf`:
`pdftoppm -r 200 -f P -l P -png book.pdf /tmp/p` renders PDF page P (the
printed page differs; check the page number on the render), then crop and
save as WebP with Pillow (`img.crop(box).save(out, "WEBP", quality=80)`).

Open-licence images (public domain, CC0, CC BY, CC BY-SA, CC BY-NC; never
ND) need `credit`, `license` and `source` (the https page it came from), and
a line in `ATTRIBUTION.md`:

```yaml
  image:
    file: hb/L05-ascites-us.webp
    credit: "James Heilman, MD"
    license: CC BY-SA 4.0
    source: https://commons.wikimedia.org/wiki/File:...
```

The app shows the credit under the caption. The pack build embeds the file,
so nothing is fetched at runtime.

## Book figures

The inventory of a lecture's figures (#983), made from the page images in
the private sources (`courses/<course>/img/pNNN.jpg`; Hepatology's are
rendered from `book/internal-medicine-pg-part1.pdf`, PDF page = printed
page + 4). Tools only, never in the content pack; the printed label or
caption is the only book text in it.

```yaml
# content/book-figures/hb/L01.yaml
lecture: hb.L01
figures:
  - page: 2
    label: "Figure 2: macronodular and micronodular cirrhosis"
    kind: histology      # photo histology imaging endoscopy clinical ecg film
                         # diagram algorithm chart table other
    file: hb/L01-macro-micronodular.webp     # a step on p2 shows this crop
  - page: 3
    label: "Figure 3: activated hepatic stellate cells"
    kind: diagram
    drawn: true          # an SVG step on p3 whose title says «في الكتاب»
  - page: 1
    label: "author photo"
    kind: other
    skip: "the lecturer's portrait; nothing to teach"
```

How to do one lecture:

1. `python3 -I scripts/content-tools/figure-candidates.py hb.L01 --sheets /tmp/sheets`
   lists, per page, the embedded pictures and the "Figure N" words, and
   writes contact sheets of the page images. Look at **every** page: drawn
   diagrams and vector tables have no embedded picture.
2. List every figure in the inventory. Tables set as text are not figures;
   a table printed as a picture is (`kind: table`; a `table` step that
   rebuilds it also counts as shown, so skip it with "rebuilt as table step
   N" only when that step is on the same page).
3. Crop each one from a fresh 200 dpi render (`pdftoppm -r 200 -f P -l P`),
   with its panel letters and arrows but without the printed caption, and
   save WebP about 800 px wide, under 150 KB (see [Images](#images)).
4. Place it after the step that refers to it and write the caption (rule 10).
   Replace an SVG that only approximates a book figure with the crop; keep
   a drawing that teaches something the book's picture does not.
5. `batna-tools validate`, then `batna-tools figures` for the progress
   table on #983. Say in the PR how many KB the images add.

## Reading layout

How the lesson player is laid out, and why (Mohamed, 2026-11-08: "study where the eye rests"). Change it only with evidence.

| Choice | Evidence |
|---|---|
| ≈ 60–75 characters a line (40rem column, 44rem with larger type from 1200px) | 55 characters a line gave the best on-screen comprehension (Dyson & Haselgrove, IJHCS 2001); 45–75 with 66 ideal (Bringhurst); moderate lengths are the easiest to read on screen (Dyson & Kipping 1998). |
| One centred column whose top stays put on every step; step heading at the top start (right) | Readers fixate the top and line starts; Arabic readers follow a mirrored F (NN/g eyetracking, 2006/2017). A fixed spot saves the eye hunting after each step. |
| Back/Next right under the card (sticky while a long step scrolls); phones keep them at the bottom | Fitts's law: nearer, larger targets are faster; on phones the bottom is the thumb's reach. |
| Lesson name muted above the card, not a second card | One focal point; the step is what to read. |
| Text colour by importance: `==key==` pill in the theme's opposite hue (`--mark`), then heading and `**terms**` in full text colour, body softened, meta muted | Signaling principle: cues on the few relevant words improve learning (Mayer; Cambridge Handbook of Multimedia Learning ch. 11). Hence max 3 marks a step. |
| Reading settings («Aa»): size, spacing and width per device; Book width is the column above, Narrow/Wide sit either side of it | Readers differ (age, eyes, screen distance); a reader-set size and spacing helps without changing the default that the research above picked. |
| Study theme: warm off-white page, charcoal text; Study dim: warm graphite page, parchment text | Dark text on a light page reads best in a lit room (Piepenbrock et al., Ergonomics 2013, 2014); a dark page tires less in a dark room and is most comfortable at moderate contrast (Xie et al., IEEE Access 2021; Vision Research 2025); moderate warmth, since 2800 K screens tired readers most (IJHCI 2024). |

## Own highlights

Mohamed can highlight any words in a step himself (`learn/highlight.js`, `batna_core::highlight`). They are his notes, not content: they are stored as synced settings (`hl.<lesson>.<id>`), never in `content/`. A highlight is found again by its words and a little text around them, so rewording a sentence in a lesson can drop a highlight on it; small edits elsewhere in the step keep it.

## Check before the PR

`cargo run -p batna-tools -- validate` checks schema, ids, pages, quotes,
item links, SVG safety, image files, sizes and licences, key-term counts,
mnemonic spelling, option counts and abbreviations (rule 12). It also
checks coverage against the book (`crates/tools/src/coverage.rs`, #833):
`pages` wider than the steps fail; pages no step cites warn, and fail once
the lecture is listed in `content/lesson-skips.yaml` (its coverage was
reviewed); `practice` cards from pages the lesson never cites and lessons
over 30 cards, 8 pages or 2 topics warn. A lesson's pages are the
different pages its steps cite (a review question a few pages on does not
widen it); in the slide-deck course (`res`) they are slides, and the limit is
16. A lesson that is wide on purpose (an end-of-deck quiz, three short
headings on two pages that would fall under eight steps if split) is listed
with its reason in `content/lesson-wide.yaml` and stops warning; a listed
lesson that is no longer wide warns again. `batna-tools lessons --warnings
<course>` lists the warnings, `lessons --report <course>` prints one TSV row
per lesson (pages, cited pages, cards, steps, words, explain steps with no
reason word, the lecture's words per page and uncited pages), and
`lessons --summary` the course table CI puts in the job summary.

**Depth bar** (#827): Hepatology. On the `lessons --report` count (all text
a learner reads in the steps, without quotes and SVG) Hepatology has about
225 words per book page and about 3 explain steps in 10 without a reason
word; bring other courses to that, never by padding. Then run the
app (`just run`, or `BATNA_OPEN_SCREEN=learn`) and play every lesson once.

## Saved responses

An unfinished lesson is the device-local setting `lessonpos.<id>`
(`batna_core::response::Attempt`, version 2). Besides the step he is on and
the first result of every step, it keeps each step's response (`Draft`): the
option picked, the order built, the pairs matched, the bins chosen, the case
parts answered, reveals, mnemonic letters and table cells opened. Responses
use authored indexes, never screen positions, and carry a fingerprint of the
step; editing a step drops its saved response. A step's state (open, draft,
checked, answered before, skipped) is derived from the restored attempt before
Check or Continue is decided. Records from before version 2 have results but
no responses: those steps show the answer key. Any answered question has a
↻ button that starts it over as practice; the first result stays the one
that counts. No explanatory notes on the page (Mohamed, 2026-10-29).
