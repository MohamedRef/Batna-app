# Batna: shared rules for every thread

Batna (باطنة) is Mohamed's internal-medicine study app for his Egyptian MSc. It is built on the same ideas as Kabar: short retrieval sessions, FSRS spaced repetition, offline content, phone and tablet first. The first content is Chapter 1 (Hepato-Biliary Pancreatic Unit) of the Alexandria Faculty of Medicine "Internal Medicine Postgraduate Lectures" e-book, 2025, part 1. **His exam is in January 2027**, so content that he can study from soon matters more than polish.

The study and opinion behind this plan: https://claude.ai/artifact/4YysQ71AHvM77GsEyJ62Nv (a copy is in this folder).

"Batna" is a working name. If Mohamed renames the app, chunk 1 changes it in one PR.

## 1. Branch and repository rules

- Repo: **`MohamedRef/Batna-app`**, branch `main`. Batna moved there on 2026-10-28 from the old orphan branch `batna` of `MohamedRef/Kabar-app`, with its full history. Never push Batna work to Kabar-app.
- **Never touch Kabar-app** and never open a Batna PR there. Kabar stays fully independent, with its own code and assets. Batna does not depend on Kabar crates. Where a Kabar module is useful (FSRS wiring, SQLite store, theme, Android TTS, AI router), **copy it** into Batna, adapt it, and credit the source path in a comment (`// adapted from kabar-app main: crates/core/src/srs.rs`).
- Work on your thread's own designated branch, created from `origin/main` of MohamedRef/Batna-app. Open PRs with **base `main`** on `MohamedRef/Batna-app`.
- Before merging: merge the latest `origin/main` (MohamedRef/Batna-app) into your branch, run `just check`, and wait for CI to be green on that commit. Then squash-merge the PR yourself. Mohamed said to do routine repo steps without asking.
- Never force-push `main`, and never rewrite another thread's history.
- Commit messages, code, comments and docs are in English. Reply to Mohamed in Egyptian Arabic, and keep replies short.

## 2. Non-negotiables (same spirit as Kabar)

- $0 a month. Ask before adding anything that could cost money.
- No secrets in the repo, ever.
- **The book text is never committed.** Cards paraphrase the book and cite its page. `quote:` holds the page's own words with no length cap (Mohamed, 2026-10-29): the full passage or table that proves the fact and gives the complete reference, as text (one row per line), never images. The PDF and the extracted text live only in `/mnt/project-files/medstudy/book/`. Batna is private and personal and is never published.
- Never send real patient data to any AI engine. Only learning text goes out.
- Tests never call live APIs.
- UI text is Egyptian Arabic (RTL, logical CSS properties, Latin isolated with `<bdi>`). **Medical content is in English**, like the book and the exam. Lay out by window width (compact under 600dp, medium under 840dp, expanded 840dp and up), never by device name.
- No generated images and no image assets. Use text, tables, and simple CSS or SVG only where a diagram is really needed.
- Tone: warm and specific. No guilt, no fake urgency.

## 3. File ownership (no overlap)

Each chunk owns the paths listed in its brief and edits nothing else. Files that everyone needs (the workspace `Cargo.toml`, `crates/app/src/main.rs` routes, `crates/app/src/i18n/mod.rs`, `content/index.yaml`) belong to **chunk 1**. A feature chunk may add **one line** to register its route or module and its own strings file (`i18n/<feature>.rs`). On a conflict, merge `origin/main` (MohamedRef/Batna-app) and keep both sides.

Workspace layout, which chunk 1 creates:

```
Cargo.toml  justfile  CLAUDE.md  README.md  docs/RUN.md  ATTRIBUTION.md
crates/core      batna-core: pure logic (schema types, FSRS, session composer, calculators, grading). No UI, no IO.
crates/app       batna-app: Dioxus 0.7 app (desktop + Android). One module dir per feature.
crates/tools     batna-tools: validate, packs, anki export, stats.
content/hb/L01.yaml … L20.yaml   one file per lecture (Chapter 1 = hb)
content/index.yaml               lecture list, titles, book page ranges, owner chunk
schemas/content.schema.json
fixtures/
.github/workflows/ci.yml
```

## 4. Content format `batna.content/v1`

There is one YAML file per lecture. Every item has a stable id, a kind, a book page, and a status. Example:

```yaml
schema: batna.content/v1
lecture: hb.L01
title: Liver Cirrhosis
author: Dr. Mohamed Bakosh            # lecturer named in the book
book: { chapter: 1, pages: [1, 22] }  # printed page numbers, not PDF pages
topics:                               # short slugs used for interleaving and stats
  - { id: definition, title: Definition and causes }
  - { id: child-pugh, title: Child-Pugh and MELD }
items:
  - id: hb.L01.001                    # never reuse or renumber; append new ids
    kind: card                        # card | cloze | mcq | enumerate | case | table | order | calc | essay
    topic: definition
    front: Anatomical definition of cirrhosis?
    back: A diffuse process with fibrosis and regenerative nodules that disrupts normal liver architecture.
    src: { page: 1, quote: "diffuse process with fibrosis and regenerative nodule formation" }
    status: unverified                # unverified | approved | rejected. Only Mohamed sets approved.
    update:                           # optional: where current guidelines differ from the book
      text: …
      ref: "EASL CPG decompensated cirrhosis 2018"
    tags: [high-yield]                # optional: high-yield, numbers, book-question
```

Fields for each kind (shared fields: id, kind, topic, src, status, optional update and tags):

| kind | fields | notes |
|---|---|---|
| `card` | `front`, `back` | One fact per card, and the answer short enough to say aloud. |
| `cloze` | `text` with `{{c1::…}}` | Use at most 3 clozes per item. Numbers and thresholds go here. |
| `mcq` | `stem`, `options` (4–5), `answer` (0-based), `why` (one line per option) | Single best answer. A clinical vignette is preferred. Every distractor gets a reason it is wrong. |
| `enumerate` | `prompt`, `points` (list), `pass` (how many count as a pass) | The "Mention / Enumerate" exam style. |
| `essay` | `prompt`, `outline` (list of headings with 1–3 bullet points each) | The lecture's own end-of-lecture questions go here with `tags: [book-question]`. |
| `case` | `stem`, `steps`: list of `{ reveal, question, options, answer, why }` | Key-feature style. Each step reveals new data, then asks one decision. |
| `table` | `title`, `columns`, `rows`, `blanks` (list of `[row, col]`) | A comparison grid with cells to fill in. |
| `order` | `prompt`, `steps` (correct order) | Algorithms and management sequences. |
| `calc` | `calculator` (`child_pugh`, `meld`, `meld_na`, `saag`, `maddrey`, `r_ratio`, `bclc`, …), `inputs`, `expect`, `interpret` | A vignette with numbers. The app computes the score and asks what it means. |

Targets per lecture: about 1 card or cloze per 45 words of book text, 1 MCQ per 150 words, 2–5 cases, every end-of-lecture question as an `essay`, and every book table as a `table` or a set of clozes. Prefer quality to volume.

## 5. How content gets checked for accuracy

1. **Ground truth is the book.** Write items only from `/mnt/project-files/medstudy/book/LNN.txt`, which has the text by printed page. Every item has `src.page`. The exam is marked by the same department, so the book's answer is the answer.
2. **Updates are separate.** If you are confident that a current major guideline (EASL, AASLD, Baveno VII, BCLC 2022, KDIGO, ACG) differs from the book, add `update:` with the guideline name and year. Never change the book answer to match it. If you are not sure, leave it out.
3. **Self-check before the PR.** Re-read each item against its page. Check the numbers, the thresholds, the drug names, and that each MCQ has exactly one best answer.
4. **Independent check.** Chunk 19 (accuracy checker) re-verifies every merged lecture against the book page and files fixes as PRs or issues labelled `batna-accuracy`.
5. **Mohamed approves while studying.** Every item ships `status: unverified`. In the app, the first time he sees an item he marks it ✓ correct, ✎ edit, or ✗ wrong. Only his ✓ sets `approved`, and ✗ removes the item from review until it is fixed. Until the app does this, the Anki decks carry an `unverified` tag.
6. **Book problems.** Typos or contradictions in the book (for example "Bergin" for "Benign" in lecture 16's title) go in `/mnt/project-files/medstudy/book-issues.md`, never silently "fixed".

Run `cargo run -p batna-tools -- validate` before every content PR. Until chunk 1 lands the validator, keep to the schema above by hand.

## 6. Deliverable for January, before the app is ready

Mohamed wants **one** Anki file, not one per lecture: `/mnt/project-files/medstudy/anki/batna.apkg`. It holds a parent deck `Batna` with one subdeck per lecture. Note GUIDs come from item ids, so importing a newer `batna.apkg` updates his cards and keeps his progress. Chunk 9 owns the exporter (`batna-tools anki`, and maybe an in-app export). Build it from an up-to-date checkout of origin/main (MohamedRef/Batna-app) with `cargo run -q -p batna-tools -- anki --out /mnt/project-files/medstudy/anki/` (PR #339). For each of the 20 lectures it takes the merged file, else `content-wip/hb/LNN.yaml`, else the draft, so **every build has all 20 lectures and the file never shrinks**. `make_apkg.py` is retired. After merging a lecture, a content chunk regenerates `batna.apkg` this way. It tells Mohamed only that the deck now includes that lecture; it never sends a separate per-lecture file.

## 7. Chunk index

| # | Chunk | Kind | Depends on |
|---|---|---|---|
| 1 | Scaffold: orphan branch, workspace, schema, validator, CI, app shell (this thread) | infra | none |
| 2 | Study session player: card, cloze, enumerate, essay | feature | 1 |
| 3 | Review: FSRS scheduling, store, Today screen, verify-while-study | feature | 1 |
| 4 | Question bank and mock exams | feature | 1, 2 |
| 5 | Case player and clinical calculators | feature | 1 |
| 6 | Tables and algorithm ordering | feature | 1, 2 |
| 7 | Oral mode (TTS) and hands-free review | feature | 1, 3 |
| 8 | Stats, mistakes log, and January exam planner | feature | 1, 3 |
| 9 | Anki exporter (Rust) and offline content packs | tools | 1 |
| 10 | Content: L01 Cirrhosis + L02 Non-invasive assessment | content | schema |
| 11 | Content: L03 Portal hypertension + L04 Vascular liver diseases | content | schema |
| 12 | Content: L09 Ascites + L10 Acute liver failure | content | schema |
| 13 | Content: L05 HCV + L06 MASLD + L07 HBV + L08 DILI | content | schema |
| 14 | Content: L13 Immune liver disease | content | schema |
| 15 | Content: L11 Transplant + L12 Metabolic + L14 Systemic and pregnancy | content | schema |
| 16 | Content: L15 Pancreas | content | schema |
| 17 | Content: L16 Benign tumors + L17 Cholangiocarcinoma + L18 HCC | content | schema |
| 18 | Content: L19 Gallbladder + L20 Nutrition | content | schema |
| 19 | Accuracy checker (independent re-verification of all content) | review | 10–18 |
| 20 | App parity: sounds, vibration, reminders, onboarding, recap (PARITY.md §2–3) | feature | 1, 8 |
| 21 | App parity: export, restore, Batna Cloud sync (PARITY.md §4) | feature | 1, 3 |
| 22 | App parity: over-the-air updates, preview APK, release builds, icons (PARITY.md §5) | infra | 1, 20 |
| 23 | App parity: tablet panes, S Pen hover, keyboard, Mac menu (PARITY.md §6) | feature | 1, 2 |

Chunks 20–23 come from Mohamed's ask (2026-10-27) that Batna has every core app feature Kabar has; the full comparison is `PARITY.md`.

Content chunks can start right away: they only need the schema above and the book text. Each brief is in `chunks/`.
