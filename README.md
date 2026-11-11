# Batna (باطنة)

Mohamed's internal-medicine study app for the Egyptian MSc in General Internal
Medicine (exam January 2027): short retrieval sessions with spaced repetition
(FSRS), an MCQ bank with explanations, key-feature cases, interactive
Egyptian-Arabic lessons, calculators and an oral-exam mode. It runs offline on
the Galaxy S24 Ultra, the Galaxy Tab S11, the Mac and in the browser. Personal
only, never published to a store.

| | |
|---|---|
| **Release** | **2026.11.2** (alpha, 2026-11-08): a signed, sideloaded APK plus a self-signed Mac app ([releases](https://github.com/MohamedRef/Batna-app/releases)). Versions follow the calendar (`YEAR.MONTH.N`); previews every 4 hours ([docs/RELEASE.md](docs/RELEASE.md)). |
| **Web** | https://batna-web.hamorefaat85.workers.dev ([docs/WEB.md](docs/WEB.md)) |
| **Sync** | Batna Cloud, a free Cloudflare Worker: Google or username + password sign-in ([docs/CLOUD.md](docs/CLOUD.md)) |
| **Cost to run** | $0/month |
| **Content status** | Every item is `unverified` until Mohamed approves it in the app. |

## Content

From the Alexandria Faculty of Medicine Internal Medicine Postgraduate
Lectures (2025) and the Basics of Research slide decks. The textbook itself
never goes in git; items paraphrase it and cite the page.

| Course | Lectures | Lesson files |
|---|--:|--:|
| Hepatology (`hb`) | 20 | 20 |
| Gastroenterology (`git`) | 11 | 11 |
| Rheumatology & Immunology (`rheum`) | 21 | 21 |
| Hematology (`hem`) | 12 | 12 |
| Endocrinology (`endo`) | 7 | 7 |
| Diabetes, Metabolism, Lipids (`dm`) | 23 | 23 |
| Geriatrics (`geri`) | 13 | 13 |
| Nephrology (`neph`) | 14 | 14 |
| Basics of Research (`res`) | 16 | 16 |
| **Total** | **137** | **137** |

Plus 98 calculators (each cites its original paper, and every result band
says what it means, the next step and the differential), an abbreviations
dictionary per course, normal-range chips on lab values and a تحشيشة
(Egyptian-Arabic mnemonic) for every list.

## What it does

| Feature | State |
|---|---|
| Today: daily plan, due/new counts, Continue a running session | ✓ |
| Study: FSRS reviews, Undo, keyboard shortcuts, session survives a restart | ✓ |
| Question bank, key-feature cases, oral-exam mode | ✓ |
| Lessons: step player, one reading column on wide screens, own highlights, تحشيشة reminders when a list comes back | ✓ |
| Calculators with cited sources and next steps | ✓ |
| Stats and planner | ✓ |
| Own cards, Share-to-Batna on Android, Anki export (`batna.apkg`, a subdeck per lecture) | ✓ |
| Themes: Paper, Night, Night shift, true black, Study, Study dim, three System themes | ✓ |
| Mac: full-window layout, resizable panes, Focus Mode, ⌘K palette | ✓ |
| Sync between devices, "not my device" sign-in | ✓ |
| Updates over the air: signed app feed, separate content pack, What's new before and after each update | ✓ |
| Courses with no material yet (cardiology, chest, neurology and 8 more) | waiting for sources |

## Docs

Continuing the work (agents and new chats): start with [AGENTS.md](AGENTS.md)
and [docs/HANDOFF.md](docs/HANDOFF.md), which hold the architecture, commands,
CI, releases, rules, current state and open work.

| Doc | What |
|---|---|
| [RUN.md](docs/RUN.md) | How to run and install on every platform |
| [CONTENT.md](docs/CONTENT.md), [LESSONS.md](docs/LESSONS.md), [AUTHORING.md](docs/AUTHORING.md) | How to write items and lessons, the workflow and its checks |
| [CALCULATORS.md](docs/CALCULATORS.md) | Every calculator and its sources |
| [RELEASE.md](docs/RELEASE.md), [COMPATIBILITY.md](docs/COMPATIBILITY.md) | Releases, version numbers, what old and new apps do |
| [CONTENT-DELIVERY.md](docs/CONTENT-DELIVERY.md) | Why the content pack is one download, with the numbers |
| [CLOUD.md](docs/CLOUD.md), [WEB.md](docs/WEB.md) | Batna Cloud and the web build |
| [TESTING.md](docs/TESTING.md) | Test harnesses and browser checks |
| [UI-MAINTENANCE.md](docs/UI-MAINTENANCE.md) | UI architecture and improvement priorities |
| [ANKI.md](docs/ANKI.md) | The Anki export |
