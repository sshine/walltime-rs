# Default recipe: list available commands
default:
    @just --list

eval:
    cargo build
    cp target/debug/wtime .
    cargo clean
    ./wtime -t -0 -d "Compiling (.*)" -m 1 cargo build

# Format all code (Rust + Nix)
fmt:
    treefmt

# Check formatting (Rust + Nix)
fmt-check:
    treefmt --fail-on-change --no-cache

# Run clippy lints
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
test:
    cargo test --all-features

# Run tests with verbose output
test-verbose:
    cargo test --all-features -- --nocapture

# Build release
build:
    cargo build --release --all-features

# Generate documentation
doc *args='':
    cargo doc --no-deps --all-features {{args}}

readme:
    cargo-readme-workspace

readme-check:
    cargo-readme-workspace --check

# Regenerate Cargo.nix from Cargo.toml/Cargo.lock
cargo-nix:
    cargo-nix

# Check Cargo.nix is in sync with Cargo.toml/Cargo.lock
cargo-nix-check:
    cargo-nix --check

# Run CI checks locally
ci: fmt-check lint test doc readme-check cargo-nix-check build
    @echo "All CI checks passed!"

# Watch for changes and run tests
watch:
    cargo watch -x test

# Clean build artifacts
clean:
    cargo clean

# Review snapshot test changes
snap:
    cargo insta test --review
