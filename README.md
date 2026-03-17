# Maximum League — MVP

Desktop app for League draft: live LCU sync, top 3 recommender, AI reasoning.

## Stack

- **Tauri v2** (Rust backend)
- **React + TypeScript** (Vite frontend)
- **Drizzle ORM** + SQLite
- **Axios**, **Lucide React**

## Setup

```bash
npm install
```

Generate placeholder icons (required for Windows build):

```bash
npm run icons:generate
```

To use your own icon later: `npm run tauri icon path/to/512.png`

## Commands

| Command | Description |
|---------|-------------|
| `npm run dev` | Vite dev server (frontend only) |
| `npm run tauri:dev` | Full Tauri app (Rust + frontend) |
| `npm run tauri:build` | Production build |
| `npm run lint` | ESLint (strict, no warnings) |
| `npm test` | Vitest |
| `npm run db:generate` | Drizzle: generate migrations |
| `npm run db:migrate` | Drizzle: run migrations |
| `npm run db:studio` | Drizzle Studio |
| `npm run icons:generate` | Generate placeholder app icons |

**Database:** Schema in `drizzle/schema.ts`; migrations in `drizzle/migrations/`. SQLite file: `lol_ranked.db` (created on first migrate). For seed scripts, use `scripts/db.cjs` in Node only.

## Structure

- `src/` — React app
- `src-tauri/src/` — Rust (lcu/, ai/, lib.rs)
- `drizzle/` — Schema and migrations

## LCU (Phase 1.3)

With the League client running, the app can discover port/auth (Windows: `wmic` process or lockfile) and call:
- **`get_lcu_status`** — `{ connected, message, port }`
- **`get_live_draft_state`** — `{ in_champ_select, phase, time_left_sec, my_team_bans, their_team_bans, my_team_picks, their_team_picks, local_player_cell_id }`

Use from the frontend via `@tauri-apps/api`: `invoke('get_lcu_status')`, `invoke('get_live_draft_state')`.

## Phase 1

See `MVP-WORKING-PLAN.md` in the template repo for the full plan.
