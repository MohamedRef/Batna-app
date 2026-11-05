# Basics of Research (course `res`), Fall 2026

Source: `schedule-fall-2026.pdf` (Alexandria Faculty of Medicine, Department of Community Medicine,
course code 05500801), sent by Mohamed 2026-10-29. 3 credit hours, semester 1.
Lectures come as PDF slide decks (Mohamed: first 7 coming next). Slides live in `slides/` here and
never go in git, same rule as the hepatobiliary book.

## Schedule

Lectures are on Saturdays and Tuesdays. The final is an electronic exam after the 12 Dec revision; its date isn't on the schedule yet.

| Id | Date | Lecturer | Topics |
|----|------|----------|--------|
| **I. Research methodology** ||||
| res.L01 | Sat 3 Oct | Prof. Eman Darwish | Introduction to health research; research process overview |
| res.L02 | Tue 6 Oct | Prof. Nadia Farghaly | Scope and definitions of epidemiology; measuring disease frequency |
| res.L03 | Sat 10 Oct | Prof. Hany Ziady | Research question(s); formulating objectives and hypotheses |
| res.L04 | Tue 13 Oct | Prof. Aida Mohey | Study designs overview (1ry and 2ry research); primary research; descriptive studies |
| res.L05 | Sat 17 Oct | Prof. Sahar Khashab | Analytic epidemiologic studies |
| res.L06 | Tue 20 Oct | Prof. Afaf Gaber | Experimental studies 1 |
| res.L07 | Sat 24 Oct | Prof. Afaf Gaber | Experimental studies 2 |
| res.L08 | Tue 27 Oct | Prof. Sahar Khashab | Revision: practice exercises on study designs |
| res.L09 | Sat 31 Oct | Ass. Prof. Eman Anwar, Ass. Prof. Nahla Gamal | Research budget; ethical principles; plagiarism |
| res.L10 | Tue 3 Nov | Ass. Prof. Eman Anwar | Data collection methods; questionnaire design; searching the literature |
| res.L11 | Sat 7 Nov | Prof. Heba El-Weshahi | Sampling methods and sample size |
| res.L12 | Tue 10 Nov | Prof. Heba El-Weshahi | Bias and confounders |
| res.L13 | Sat 14 Nov | Ass. Prof. Noha Sleem, Prof. Salwa Tayel | Writing a scientific paper; Vancouver referencing |
| res.L14 | Tue 17 Nov | Prof. Maha Moamen | Measures of diagnostic accuracy |
| **II. Biostatistics** ||||
| res.L15 | Sat 21 Nov | Prof. Sahar Khashab | Uses of biostatistics; variables; data presentation |
| res.L16 | Tue 24 Nov | Ass. Prof. Noha Sleem | Normal distribution; Z-score |
| res.L17 | Sat 28 Nov | Ass. Prof. Nahla Gamal | Null/alternative hypothesis; hypothesis testing; t-test, chi-square |
| res.L18 | Tue 1 Dec | Prof. Heba El-Weshahi | Type I/II errors; power; correlation; nonparametric tests; choosing a test |
| res.L19 | Sat 5 Dec | Prof. Aida Mohey | Introduction to EBM |
| res.L20 | Tue 8 Dec | Prof. Aida Mohey | EBM applications; critical appraisal |
| res.L21 | Sat 12 Dec | Prof. Heba El-Weshahi | Revision |

The plan numbers the lectures by schedule order. Revision sessions (L08, L21) become practice sets, not new content.

## Content pipeline (ready for the slides)

Same as the hepatobiliary course, adapted to slides:

1. **Intake (this thread):** slides go to `slides/res-LNN.pdf`. Text goes in `text/LNN.txt` per slide (`pdftotext -layout`, one `=== slide N ===` block each), and slide images in `img/LNN-sNN.webp` for figures. None of this goes in git.
2. **Index:** `content/index.yaml` gets `res.LNN` entries. `pages` = slide range `[1, N]`, and `src.page` in items means the slide number. The app labels slides "slide N" instead of "ص N" for `res` (a small app change).
3. **Content per lecture:** `content/res/LNN.yaml` (cards, MCQs, cloze, calc items) and `content/lessons/res/LNN.yaml` (interactive lessons in Egyptian Arabic), with the same validator and quote rules. Every item is `status: unverified`.
4. **Research-only tools** (from the approved brainstorm): a 2×2 diagnostic-accuracy calculator (L14), incidence/prevalence/RR/OR/NNT calculators (L02, L05–L07), a "pick the study design" drill (L04–L08), a "pick the statistical test" decision tree (L17–L18), Z-score and sample-size playgrounds (L11, L16), and critical-appraisal mini-abstracts (L20).
5. **Accuracy pass:** an independent re-check against the slides, as chunk 19 did for the hepatobiliary book.
6. **Anki:** `Batna::Basics of Research::L01 …` in the one batna.apkg.

Threads: one per 2–3 lectures once slides arrive (L01–L03, L04–L05, L06–L08 for the first batch), plus one tools thread for the calculators and drills.

## Calculator backlog
Research calculators are on main (#405). res.L02 was converted in #410, and res.L04/L05 in #408. Still mcq because no calculator fits them yet: res.L02 019 (plain proportion), 036 and 057 (duration D = P / I), 044 (crude death rate) and 045 (age-specific comparison). Candidates: a `duration` mode on prevalence_id and a `crude_rate` calculator.
