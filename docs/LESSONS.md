# Writing Batna lessons (`batna.lesson/v1`)

Lessons are the teaching layer: Mohamed opens a lesson, understands the
*why*, and only then reviews the lecture's cards. Cards and questions stay in
`content/hb/LNN.yaml`; lessons live beside them in
**`content/lessons/hb/LNN.yaml`**, one file per lecture, and point at the
cards by id.

The model to copy is `content/lessons/hb/L01.yaml`.

## The rules (the validator enforces most of them)

1. **The book is the answer.** Teach only from `/mnt/project-files/medstudy/book/LNN.txt`.
   Every step that teaches a fact gets `page:` (printed page). A `quote:`
   is optional and at most 25 words. The book text never goes in git.
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
8. **Short.** 6–10 minutes, 8–16 steps. A lecture usually needs 3–7 lessons
   that follow the book's order and together cover every page.
9. **Link to the cards.** End with a `try` step (1–5 playable items: card,
   cloze, enumerate, essay, table, order) and list the lecture items the
   lesson teaches in `practice:`.
10. **Figures are drawn, not pasted.** Inline SVG only, with theme classes
    (below) or `var(--token)` colours; no scripts, links, images, `<style>`
    or fixed colours. Labels in English. Under 24 KB. Draw what helps:
    pathways, vicious cycles, algorithms, anatomy sketches, scales.

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
| `explain` | `title?`, `text` | Prose. Blank line = new paragraph, `- ` = bullet, `**bold**` = highlighter. |
| `figure` | `title?`, `svg`, `caption`, `alt` | A drawing; tap to zoom. |
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
isolated for you, so write it naturally there.

## Check before the PR

`cargo run -p batna-tools -- validate` checks schema, ids, pages, quotes,
item links, SVG safety, mnemonic spelling and option counts. Then run the
app (`just run`, or `BATNA_OPEN_SCREEN=learn`) and play every lesson once.
