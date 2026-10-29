# Writing Batna content

The full rules are in `/mnt/project-files/medstudy/SPLIT.md` §4–5. In short:

- There is one file per lecture: `content/hb/LNN.yaml`, listed in `content/index.yaml` with its printed page range.
- Every item has `id` (`hb.L01.001`, never reused), `kind`, `topic`, `src.page` (inside the lecture's range), and `status: unverified`.
- The kinds are `card`, `cloze`, `mcq`, `enumerate`, `essay`, `case`, `table`, `order` and `calc`. `fixtures/content/sample.yaml` shows one of each.
- Write from the book only. Put guideline differences in `update: { text, ref }`.
- `src.page` is the page that holds the fact the item tests (the answer, not the review-question page), and `src.quote` (required: every page chip shows it) is that page's own words containing it, at most 25 words. `/mnt/project-files/medstudy/pagerefs/pick_quotes.py` proposes one and `set_quote.py` writes it.
- Page references are locked per lecture in `content/book-refs/LNN.tsv`. After changing a page or quote, run `cargo run -p batna-tools -- refs --lecture hb.LNN --write` where the book text is (`/mnt/project-files/medstudy/book`); a reference it can't confirm from the text gets read by eye and listed in `pagerefs/verdicts/LNN.tsv` (`ref<TAB>page<TAB>ok<TAB>reason`). `validate` fails on any page or quote that changed since its check, and on a quote that shares too few terms with its item.
- Run `just validate` before every PR. It checks the schema, unknown keys (typos), ids, page ranges, quote length, MCQ shape, cloze syntax, table grids, calculators, and that nothing is `approved` unless Mohamed approved it.
