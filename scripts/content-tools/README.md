# Content helper scripts

Python helpers the content threads used, copied from the Claude project's
shared folder on 2026-11-05 so they survive without it. They are snapshots:
not run by CI, and some read the book text, which lives outside git
(`docs/HANDOFF.md` § Sources outside git). Point them at that folder with
`BATNA_SOURCES=/path/to/medstudy` (a clone of the private repo `MohamedRef/Batna-sources`; default `/mnt/project-files/medstudy`).
Run with `python3 -I` from the repo root; they need `pyyaml`.

| Script | What it does |
|---|---|
| `tahshishat-gen.py <repo> <out-prefix>` | Collects every enumerate `hook` (تحشيشة) from `content/*/L*.yaml` into one `.md` + `.csv` for Mohamed. Rerun after any hook change. |
| `side_by_side.py <lecture.yaml> <book LNN.txt> [id-prefix]` | Prints each item next to the book page it cites, for accuracy checks. |
| `expand_quotes.py <repo> <course> [--lecture L01] --report out.tsv [--apply]` | Widens `src.quote` to the full paragraph/bullet/table on the cited page (no-cap quote rule). Uses `pagerefs_lib.py`-style page parsing. |
| `cuts.py <repo> <course>...` | Lists quotes that stop mid-sentence. |
| `fixcuts.py <repo> <course> <cuts.tsv>` | Widens the listed quotes to the whole sentence. |
| `abbrfix.py <repo> <course>...` | Spells out each abbreviation at its first mention per lecture (text/why/caption) from `content/glossary.yaml`. It can hit a quote line first; revert those (validate flags changed quotes). |
| `pagerefs_lib.py` | Book page parsing shared by the page-reference audit. |

After any of them: `cargo run -q -p batna-tools -- validate`, review the diff by
hand, then the usual `just check`.
