# Batna lessons: brief for content threads

Mohamed (2026-10-28): Batna must TEACH, not only quiz. Each lecture gets
short interactive lessons in Egyptian Arabic (medical terms in English) that
explain the why, with mnemonics, light Egyptian jokes (tahshish), drawn SVG
diagrams, and checks between screens. Treat this as a seed: be creative.

- Repo MohamedRef/Batna-app, base `main`. Never touch Kabar-app.
- Format, player and validator: PR #141 (thread "Add interactive lessons to
  Batna"). Until it merges, branch from `claude/project-thread-1xzc2m`;
  after, from `origin/main`.
- Rules and step kinds: `docs/LESSONS.md` in the repo. Model to copy:
  `content/lessons/hb/L01.yaml`.
- You own ONLY `content/lessons/hb/LNN.yaml` for your lectures. Need a new
  step kind or a player change? Message the "Add interactive lessons to
  Batna" thread instead of editing app code.
- Source = `/mnt/project-files/medstudy/book/LNN.txt` (never commit it).
  Book answer wins; guideline differences only in `update` steps with
  name + year. status: unverified. Quotes <= 25 words.
- Cover every page of the lecture, in book order, in 3-7 lessons of 6-10
  minutes. Every lesson: >= 1 question step (aim for one every 2-3 screens),
  something that sticks (mnemonic / analogy / joke / exam trap), a `try`
  step with playable items (card, cloze, enumerate, essay, table, order) and
  a `practice` list of the lecture items it teaches.
- Draw figures where they really help (pathways, vicious cycles, algorithms,
  anatomy sketches, score scales). Stack panels vertically (phone first,
  viewBox about 330 wide) so labels stay readable.
- Before the PR: `just check` green, then play each lesson in the app
  (`BATNA_OPEN_SCREEN=learn BATNA_OPEN_LESSON=hb.LNN.lesson1
  BATNA_LESSON_STEP=n`, Xvfb screenshots: see memory kabar-desktop-qa) and
  fix anything that looks wrong. Squash-merge yourself after green CI.
- Reply to Mohamed in short Egyptian Arabic.
