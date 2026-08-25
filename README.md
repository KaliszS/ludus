# Ludus

Self-hosted productivity app.

| Directory | Service  | Stack                        |
|-----------|----------|------------------------------|
| `api/`    | daemon   | Rust, Axum, Diesel, Postgres |
| `tui/`    | terminal | Go, Bubble Tea               |
| `gui/`    | web      | SvelteKit, Tailwind          |

```sh
cp .env.example .env
just setup
just api-migrate
just api-dev
just gui-dev
```

Run `just` for the full list of recipes.
