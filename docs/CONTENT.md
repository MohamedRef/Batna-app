# Writing Batna content

The full rules are in `/mnt/project-files/medstudy/SPLIT.md` §4–5. In short:

- There is one file per lecture: `content/hb/LNN.yaml`, listed in `content/index.yaml` with its printed page range.
- Every item has `id` (`hb.L01.001`, never reused), `kind`, `topic`, `src.page` (inside the lecture's range), and `status: unverified`.
- The kinds are `card`, `cloze`, `mcq`, `enumerate`, `essay`, `case`, `table`, `order` and `calc`. `fixtures/content/sample.yaml` shows one of each.
- Write from the book only. Put guideline differences in `update: { text, ref }`.
- Run `just validate` before every PR. It checks the schema, unknown keys (typos), ids, page ranges, quote length, MCQ shape, cloze syntax, table grids, calculators, and that nothing is `approved` unless Mohamed approved it.
