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
just setup      # database, migrations, stand-in user, GUI dependencies
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

Authentication does not exist yet - every request runs as one seeded user, which
is why the daemon binds to loopback only.

Run `just` for the full list of recipes.
