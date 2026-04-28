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

# Generate one durable policy-v2 selfplay run with the production-equivalent uniform-prior baseline
selfplay GAMES="100" PLAYOUTS="20000" MAX_SAMPLES="100000": install_cargo install_training
    cd training && uv run generate_selfplay.py --games {{GAMES}} --playouts {{PLAYOUTS}} --max-samples {{MAX_SAMPLES}}

# Generate one durable policy-v2 experimental selfplay run using neural network root priors
selfplay_nn GAMES="200" PLAYOUTS="800" MAX_SAMPLES="100000" PRIOR_WEIGHT="0.01": install_cargo install_training
    cd training && uv run generate_selfplay.py --games {{GAMES}} --playouts {{PLAYOUTS}} --nn --prior-weight {{PRIOR_WEIGHT}} --max-samples {{MAX_SAMPLES}}

# Rebuild the active training_data.jsonl replay view from uniform teacher runs only
replay MAX_SAMPLES="100000": install_training
    cd training && uv run replay.py build --max-samples {{MAX_SAMPLES}}

# Rebuild replay with explicitly selected teachers for experiments
replay_experimental TEACHERS="uniform,nn_root" MAX_SAMPLES="100000": install_training
    cd training && uv run replay.py build --max-samples {{MAX_SAMPLES}} --include-teachers {{TEACHERS}}

# Train the model on existing data
train_only EPOCHS="20": install_training
    cd training && uv run train.py --epochs {{EPOCHS}}

# Evaluate trained policy priors against the uniform-prior production baseline
evaluate_model PAIRS="200" PLAYOUTS="400" MIN_SCORE="0.53": install_cargo
    cargo run --release --bin nn_vs_mcts -- {{PAIRS}} {{PLAYOUTS}} --summary-only --min-score {{MIN_SCORE}}

# Evaluate trained policy priors without enforcing a promotion gate
evaluate_model_report PAIRS="100" PLAYOUTS="400": install_cargo
    cargo run --release --bin nn_vs_mcts -- {{PAIRS}} {{PLAYOUTS}}

# Sweep root prior blend weights without enforcing a promotion gate
evaluate_prior_sweep PAIRS="200" PLAYOUTS="400" WEIGHTS="0.00,0.02,0.05,0.08,0.10,0.15,0.25": install_cargo install_training
    cd training && uv run prior_sweep.py --pairs {{PAIRS}} --playouts {{PLAYOUTS}} --weights {{WEIGHTS}}

# Sanity-check evaluator by comparing uniform priors against uniform priors
evaluate_uniform PAIRS="20" PLAYOUTS="200": install_cargo
    cargo run --release --bin nn_vs_mcts -- {{PAIRS}} {{PLAYOUTS}} --uniform-vs-uniform

# Promote the current ONNX model to the browser-served artifact path after evaluation passes
promote_model PAIRS="200" PLAYOUTS="400" MIN_SCORE="0.53": install_cargo
    cargo run --release --bin nn_vs_mcts -- {{PAIRS}} {{PLAYOUTS}} --summary-only --min-score {{MIN_SCORE}}
    mkdir -p www/public/models
    cp training/artifacts/model.onnx www/public/models/htmf-policy.onnx
    @echo "Promoted training/artifacts/model.onnx to www/public/models/htmf-policy.onnx"

# Run one iteration: selfplay + training + evaluation
train GAMES="100" PLAYOUTS="20000" EPOCHS="20": install_cargo install_training
    @echo "Running selfplay with {{GAMES}} games, {{PLAYOUTS}} playouts..."
    just selfplay {{GAMES}} {{PLAYOUTS}}
    @echo "Training for {{EPOCHS}} epochs..."
    just train_only {{EPOCHS}}
    @echo "Evaluating trained model for promotion..."
    just promote_model
    @echo "Training iteration complete and model passed the promotion gate."

# Run iterative training loop with promotion gate
iterate ITERATIONS="20" GAMES="200" PLAYOUTS="1000" EPOCHS="20" EVAL_PAIRS="100" EVAL_PLAYOUTS="400": install_cargo install_training
    cd training && uv run iterate.py --iterations {{ITERATIONS}} --games {{GAMES}} --playouts {{PLAYOUTS}} --epochs {{EPOCHS}} --eval-pairs {{EVAL_PAIRS}} --eval-playouts {{EVAL_PLAYOUTS}}

# Create blank models for debugging
blank_models: install_training
    cd training && uv run create_blank_models.py

# Tiny end-to-end ML smoke test
ml_smoke: install_cargo install_training
    cd training && uv run generate_selfplay.py --games 1 --playouts 20 --max-samples 1000
    cd training && uv run train.py --epochs 1 --num-filters 8 --num-blocks 1 --batch-size 16
    cargo run --release --bin nn_vs_mcts -- 1 20
