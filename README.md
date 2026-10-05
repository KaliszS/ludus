# Ludus

Self-hosted productivity app: habits, daily check-ins, plan levels that fill up
from them, and statistics over time. The terminal client has not been started yet.

| Directory | Service  | Stack                        |
|-----------|----------|------------------------------|
| `api/`    | daemon   | Rust, Axum, Diesel, Postgres |
| `gui/`    | web      | SvelteKit, Tailwind, PWA     |
| `tui/`    | terminal | Go, Bubble Tea               |

## Requirements

Docker, Rust, [Bun](https://bun.sh), [just](https://github.com/casey/just) and
the Diesel CLI:

```sh
cargo install diesel_cli --no-default-features --features postgres
```

## Running

```sh
cp .env.example .env
just setup      # database, migrations, GUI dependencies
just api-run    # daemon, first terminal
just gui-dev    # web client, second terminal
```

Then open <http://localhost:7531>.

| Service  | Port  |
|----------|-------|
| Postgres | 7532  |
| daemon   | 7530  |
| web      | 7531  |

Database contents survive `docker compose down`; only `just db-reset` discards
them.

Signing in needs a Google OAuth client of type "Web application" with this
redirect URI, its ID and secret in `.env` as `LUDUS_GOOGLE_CLIENT_*`:

```
http://localhost:7530/v1/auth/google/callback
```

`LUDUS_REGISTRATION` decides what happens to someone signing in for the first
time: `open` creates the account, `approval` creates it pending until an admin
sets `users.status` to `active`, `closed` turns them away.

Run `just` for the full list of recipes.
