set dotenv-load := true

default:
    @just --list --unsorted

# --- all services ---

# Bring up the database and install GUI dependencies
setup: db-up gui-install
    @echo "Next: just api-migrate, then api-dev and gui-dev in separate terminals"

# Everything that must pass before a commit
check: api-check tui-check gui-check

fmt: api-fmt tui-fmt gui-fmt

test: api-test tui-test

build: api-build tui-release gui-build

# --- database ---

db-up:
    docker compose up -d postgres

db-down:
    docker compose down

# Drop the database along with its data and start over
db-reset:
    docker compose down -v
    docker compose up -d postgres

db-logs:
    docker compose logs -f postgres

db-shell:
    docker compose exec postgres psql -U ludus -d ludus

# --- api (Rust) ---

# Run the daemon, restarting on file changes (needs cargo-watch)
api-dev:
    cd api && cargo watch -x run

api-run *args:
    cd api && cargo run -- {{ args }}

api-build:
    cd api && cargo build --release

api-fmt:
    cd api && cargo fmt

api-lint:
    cd api && cargo clippy --all-targets --all-features -- -D warnings

api-test:
    cd api && cargo test

api-check: api-fmt api-lint api-test

# just api-migration create_users
api-migration name:
    cd api && diesel migration generate {{ name }}

api-migrate:
    cd api && diesel migration run

api-rollback n="1":
    cd api && diesel migration revert -n {{ n }}

# Roll migrations back and re-apply them, proving down.sql actually works
api-redo n="1":
    cd api && diesel migration redo -n {{ n }}

# Which migrations are applied and which are pending
api-migrations:
    cd api && diesel migration list

# Regenerate src/repo/schema.rs from the live database
api-schema:
    cd api && diesel print-schema > src/repo/schema.rs

api-audit:
    cd api && cargo audit

# --- tui (Go) ---

tui-dev *args:
    cd tui && go run ./cmd/ludus {{ args }}

tui-build:
    cd tui && go build -o bin/ludus ./cmd/ludus

# Stripped static binary for distribution
tui-release:
    cd tui && CGO_ENABLED=0 go build -trimpath -ldflags="-s -w" -o bin/ludus ./cmd/ludus

tui-fmt:
    cd tui && gofmt -w .

tui-lint:
    cd tui && go vet ./...
    cd tui && go tool golangci-lint run

tui-test:
    cd tui && go test -race ./...

tui-check: tui-fmt tui-lint tui-test

tui-tidy:
    cd tui && go mod tidy

tui-vuln:
    cd tui && go tool govulncheck ./...

# --- gui (SvelteKit) ---

gui-install:
    cd gui && bun install

gui-dev:
    cd gui && bun run dev

# Static build into gui/build
gui-build:
    cd gui && bun run build

gui-preview:
    cd gui && bun run preview

gui-fmt:
    cd gui && bun run format

gui-lint:
    cd gui && bun run lint

gui-check:
    cd gui && bun run check

# Generate TypeScript types from the daemon's OpenAPI document
gui-types:
    cd gui && bunx openapi-typescript "$PUBLIC_DEFAULT_API_URL/v1/openapi.json" -o src/lib/api/schema.d.ts
