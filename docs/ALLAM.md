# Dr. Allam's course (sealed)

Dr. Mahmoud Allam's 24-day internal medicine revision course is on YouTube: 101 videos, taught in Egyptian Arabic. Mohamed has his own transcripts of it: 86 lessons with timestamps, one WebVTT subtitle file per video, and an index of the lessons with links to the exact second. On 2026-11-21 he asked that they live in this repo **encrypted**, the way Steps keeps its VQ bank (steps-app: docs/VQ.md).

The repo is public, so only the sealed bytes go in. They open only with Mohamed's key.

Batna uses the course in two ways, both written by hand from the opened transcripts:

- **Video links.** A lesson's `videos:` list points to the second in a video where its subject is explained. The lesson header shows them (docs/LESSONS.md § Videos).
- **Lesson asides.** An aside starting «من شرح د. علام:» adds his explanation where it helps: a simpler way to see it, a useful fact, or an exam point. Only the syllabus's subjects go in, and the book stays the exam answer.

## What came of it (2026-11-21)

- An agent read each of his 86 lessons (the transcript and the subtitle timings), the app's lessons and the book. It proposed links to the second where he explains an app lesson, and notes where his explanation adds something. Places where he contradicts the book were left out and recorded in that agent's notes.
- A separate agent then read every proposed note against the book. It kept the note, edited it, or dropped it, and kept at most 3 notes per lesson.
- The result: **771 links on 432 lessons** and **412 lecturer notes on 263 lessons**, in hb, git, hem, neph, endo, dm, rheum and geri. His infectious-disease lessons and most of his cardiology, chest and neurology fall outside the syllabus and add little.
- The working files (per-lesson notes of where he differs from the book, and the verdicts) stayed outside the repo.

## The key

- 32 random bytes, written as 52 letters and digits in groups of four (`ABCD-EFGH-…`). Spaces, dashes and case don't matter when you type it.
- `batna-tools allam keygen` prints a new key and its **key id** (8 hex digits). Nothing is written to disk.
- The key is kept by Mohamed, in his password manager. It is **never in git**, an issue or a pull request. The current key id is `148444bd`.
- The key id names the key in the sealed files. It tells nothing about the key.
- To change the key:
  1. `allam open` with the old key.
  2. Delete `content/allam/`.
  3. `allam seal` with the new key.

## The files

There is one sealed file per file of the opened folder, at `content/allam/<path>.yaml`:

```yaml
# Dr. Mahmoud Allam's course, one file, sealed (docs/ALLAM.md).
schema: batna.sealed/v1
path: lessons/HEP-L01.md
key: "148444bd"  # key id: which key sealed it (never the key)
sealed: <base64url>
```

The opened folder holds 189 files:

| Path | What |
|---|---|
| `lessons/<ID>.md` | His 86 lessons (`HEP-L01` … `CNS-L14`). Each has YAML front matter (title, branch, source videos, start), a table of sub-topic links, then the verbatim transcript under 5-minute headings. |
| `subtitles/BL-NNN.vtt` | One WebVTT file per video (`BL-001` … `BL-101`), with a cue every 10–30 seconds. Exact seconds for links come from here. |
| `index/lessons.json` | Every lesson's metadata and deep links. |
| `index/timestamps.md` | The master index of lessons and timestamps. |

Crypto, adapted from steps-app main (crates/core/src/sealed.rs), in `crates/tools/src/allam.rs`:

- XChaCha20-Poly1305, with the file's `path` as associated data. A blob moved to another path doesn't open.
- The file's bytes are compressed with DEFLATE before sealing.
- The subkey is keyed BLAKE2b-256 of `batna sealed enc`.
- The nonce is keyed BLAKE2b, under the subkey of `batna sealed nonce`, of `path ‖ 0x00 ‖ bytes`. The same file under the same key always gives the same bytes, so an untouched file never shows in a diff.
- A blob is `0x01 ‖ nonce ‖ ciphertext+tag`, in base64url without padding.

## Reading or changing them

With the key in the shell (`export BATNA_ALLAM_KEY=…`):

1. `cargo run -p batna-tools -- allam open ../allam-open` writes every file to `../allam-open/<path>`. The folder must be outside the repo and new or empty. `allam-open/` is in `.gitignore` in case a copy lands in the repo anyway.
2. To change the set, edit that folder. Then run `cargo run -p batna-tools -- allam seal ../allam-open`, which writes `content/allam/` and removes the sealed files the folder no longer has. Opening and then sealing again changes nothing.
3. Delete `../allam-open` when done. The opened text never goes in git, an issue or a PR. Asides paraphrase him and never quote long stretches.

## What validate checks

- **Always, without the key:**
  - each file's schema;
  - `path` equal to the file's own path without `.yaml`;
  - a blob with the right shape;
  - one key id across all files.
- **With `BATNA_ALLAM_KEY`:** every file is opened.
- **Without the key**, the summary line says `Allam course: N sealed files (BATNA_ALLAM_KEY not set, not opened)`. CI doesn't hold the key, so it checks the shape only.
