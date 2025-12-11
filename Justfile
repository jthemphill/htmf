# Default recipe to list available commands
default:
    @just --list

# Remove build artifacts
clean:
    rm -rf wasm/pkg www/dist

# Check that cargo is installed
install_cargo:
    @command -v cargo > /dev/null || (echo "Cargo not found. Please install Rust from https://rustup.rs/" && exit 1)

# Run Rust tests
test_rust: install_cargo
    cargo test

# Install wasm-pack
install_wasm_pack: install_cargo
    cargo install wasm-pack

# Build WASM package
build_wasm: install_wasm_pack
    wasm-pack build wasm --target web --profiling

# Install bun dependencies
install: build_wasm
    bun install

# Install Playwright browsers
playwright_install:
    cd www && bunx playwright install --with-deps

# Start development server
dev: install
    cd www && bun run dev

# Build www
build_www: install
    cd www && bun run build

# Lint www
lint_www: install
    cd www && bun run lint

# Typecheck www
typecheck_www: install
    cd www && bun run typecheck

# Build everything (www, lint, typecheck)
build: build_www

# Preview production build
preview: build
    cd www && bun run preview

# Run www tests
test_www: install playwright_install
    cd www && bun run test:headless

# Run all tests
[parallel]
test: test_rust lint_www typecheck_www test_www

# Deploy to Cloudflare Pages
deploy: build install test
    cd www && bun run deploy:pages
