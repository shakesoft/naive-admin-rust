# justfile - Project unified command entry
set shell := ["pwsh", "-c"]
# check
check:
    cargo check &
    cd vue-naive-admin && pnpm lint

# Development mode: start backend + web together
dev:
    just dev-backend &
    just dev-web

# Start Rust backend (with hot reload)
dev-backend:
    cargo watch -x run -w src

dev-run:
    cargo run --bin main

# Start web (Vite dev mode)
dev-web:
    cd vue-naive-admin && pnpm dev

# Build all (production)
build:
    just build-backend
    just build-web

# Build Rust backend release
build-backend:
    cargo build --release && copy ./.env ./target/release/.env

# Build web production bundle
build-web:
    cd vue-naive-admin && pnpm build

# Clean build outputs
clean:
    rm -rf /target vue-naive-admin/dist
