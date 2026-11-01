# Batna (باطنة)

Mohamed's internal-medicine study app: short retrieval sessions, spaced repetition (FSRS), MCQs with explanations, key-feature cases, calculators, and an oral-exam mode. It runs offline on the S24 Ultra, the Tab S11 and the Mac. It is private and never published.

The first content is Chapter 1 (Hepato-Biliary Pancreatic Unit) of the Alexandria Faculty of Medicine Internal Medicine Postgraduate Lectures (2025): 20 lectures, written into `content/hb/`.

| Part | State |
|---|---|
| Content schema `batna.content/v1` and validator | ✓ |
| App shell: navigation, RTL layout for phone and tablet, searchable grouped Settings sheet, themes (Paper, Night, Night shift), text size, bundled fonts, drawn icons, Android Back, diagnostics | ✓ |
| Review: FSRS scheduling, Today screen, ✓ / ✎ / ✗ content checks, export and restore | ✓ |
| Sync between Mac, phone and tablet: Batna's own free Cloudflare Worker, Google sign-in, devices list ([docs/CLOUD.md](docs/CLOUD.md)) | ✓ (deploy pending) |
| Your own cards: add-a-card form, Share-to-Batna on Android, their own study deck and Anki subdeck | ✓ |
| Updates over the air: signed feed, "new version" notification, and what's new before and after each update with a changelog in Settings (Morphe-style sheet) | ✓ |
| Study sessions, question bank, cases, oral mode, stats | in progress |
| Anki export | `batna-tools anki`: one `batna.apkg`, a subdeck per lecture ([docs/ANKI.md](docs/ANKI.md)) |

How to run it: [docs/RUN.md](docs/RUN.md). How to write content: [docs/CONTENT.md](docs/CONTENT.md); the workflow and its checks: [docs/AUTHORING.md](docs/AUTHORING.md). What old and new apps do with each version number: [docs/COMPATIBILITY.md](docs/COMPATIBILITY.md). Why the content pack is one download, with the numbers: [docs/CONTENT-DELIVERY.md](docs/CONTENT-DELIVERY.md).

UI architecture and improvement priorities: [docs/UI-MAINTENANCE.md](docs/UI-MAINTENANCE.md).
