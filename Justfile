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

# Training dependencies
install_training:
    @command -v uv > /dev/null || (echo "uv not found. Please install from https://docs.astral.sh/uv/" && exit 1)
    cd training && uv sync

# Generate selfplay training data (traditional MCTS, high quality for bootstrapping)
selfplay GAMES="100" PLAYOUTS="20000": install_cargo
    mkdir -p training/artifacts
    cargo run --release -p selfplay -- {{GAMES}} {{PLAYOUTS}} > training/artifacts/training_data.jsonl
    @echo "Generated training data with auxiliary targets (ownership, score_diff)"

# Generate selfplay training data using neural network guidance (faster, more games)
selfplay_nn GAMES="200" PLAYOUTS="800": install_cargo
    mkdir -p training/artifacts
    cargo run --release -p selfplay -- {{GAMES}} {{PLAYOUTS}} --nn >> training/artifacts/training_data.jsonl
    @echo "Generated NN-guided training data with auxiliary targets"

# Train the model on existing data
train_only EPOCHS="20": install_training
    cd training && uv run train.py --epochs {{EPOCHS}}

# Run one iteration: selfplay + training (traditional MCTS for bootstrapping)
train GAMES="100" PLAYOUTS="20000" EPOCHS="20": install_cargo install_training
    @echo "Running selfplay with {{GAMES}} games, {{PLAYOUTS}} playouts..."
    just selfplay {{GAMES}} {{PLAYOUTS}}
    @echo "Training for {{EPOCHS}} epochs..."
    just train_only {{EPOCHS}}
    @echo "Training iteration complete!"

# Run iterative training loop (AlphaZero-style with NN-guided selfplay)
iterate ITERATIONS="20" GAMES="200" PLAYOUTS="1000" EPOCHS="20": install_cargo install_training
    cd training && uv run iterate.py --iterations {{ITERATIONS}} --games {{GAMES}} --playouts {{PLAYOUTS}} --epochs {{EPOCHS}}

# Create blank models for debugging
blank_models: install_training
    cd training && uv run create_blank_models.py
