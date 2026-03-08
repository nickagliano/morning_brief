# morning_brief — Customization Guide

morning_brief is a scheduled notification daemon. Every day at 7am Eastern it fetches
your incomplete todos from todo and texts you a summary via txtme. It ships with
a hardcoded 7am ET schedule and a plain-text message format — both are ports.

## Ports

### `NOTIFY_TIME` / schedule

**What it does:** Controls when the daily brief fires.
**Default:** 7:00am America/New_York, hardcoded in `src/main.rs`.
**How to customize:** Edit `secs_until_7am_eastern()` in `src/main.rs`. To change
the time, replace the `NaiveTime::from_hms_opt(7, 0, 0)` values. To change the
timezone, replace `chrono_tz::America::New_York` with any tz from the `chrono-tz` crate
(e.g. `chrono_tz::America::Los_Angeles`).

### `MESSAGE_FORMAT`

**What it does:** Controls the text of the SMS.
**Default:** "Good morning! N todos:\n1. task\n2. task..."
**How to customize:** Edit the `// PORT: MESSAGE_FORMAT` block in `send_brief()` in
`src/main.rs`. The `pending` variable is a `Vec<&Task>` where each Task has a `text: String`
field. Return any `String` you want.

### `SIMPLE_TODO_URL`

**What it does:** Where morning_brief fetches tasks from.
**Default:** `http://localhost:8765`
**How to customize:** Set in `.env`. Point at any service that serves `GET /tasks`
returning `[{"text": "...", "done": false}, ...]`.

### `TXTME_URL` / `TXTME_API_KEY`

**What it does:** Where morning_brief sends the notification.
**Default:** `http://localhost:5543` with no key.
**How to customize:** Set in `.env`. Any service that accepts `POST /notify` with
`{"message": "..."}` and an `X-Api-Key` header works here.

## Getting Started

1. Clone: `git clone <repository>`
2. `cp .env.example .env` and fill in the URLs and API key
3. `chmod +x serve.sh && ./serve.sh`

Or via EPC: `epc deploy morning_brief --local ./morning_brief`
