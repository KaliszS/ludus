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

Accounts sign in with an email and a password out of the box. Google is
optional: create an OAuth client of type "Web application" with this redirect
URI and put its ID and secret in `.env` as `LUDUS_GOOGLE_CLIENT_*`:

```
http://localhost:7530/v1/auth/google/callback
```

`LUDUS_REGISTRATION` decides what happens to someone new: `open` creates the
account, `approval` creates it pending, `closed` turns them away. There is no
email, so accounts are administered from the server:

```sh
just api-user list
just api-user add <email> [name]   # prints a generated password
just api-user password <user>      # issues a new one, ends every session
just api-user activate <user>      # approves a pending account
```

Run `just` for the full list of recipes.
