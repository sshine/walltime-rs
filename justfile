# Default recipe: list available commands
default:
    @just --list

eval:
    cargo build
    cp target/debug/walltime .
    cargo clean
    ./walltime -t -0 -d "Compiling (.*)" -m 1 cargo build

# Format all code (Rust + Nix + Markdown)
fmt:
    treefmt

# Check formatting (Rust + Nix + Markdown)
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

readme_args := "--project-root crates/walltime --input src/lib.rs --no-title --no-license --no-badges"

# Regenerate README.md from the walltime crate docs
readme:
    cargo readme {{readme_args}} -o README.md

# Check README.md is in sync with the crate docs
readme-check:
    cargo readme {{readme_args}} | diff - README.md

# Regenerate Cargo.nix from Cargo.toml/Cargo.lock
cargo-nix:
    cargo-nix

# Check Cargo.nix is in sync with Cargo.toml/Cargo.lock
cargo-nix-check:
    cargo-nix --check

release_targets := "x86_64-unknown-linux-musl aarch64-unknown-linux-musl"

# Assert the release tag names the version cargo would publish
check-version version:
    @pkgid="$(cargo pkgid -p walltime)"; crate="v${pkgid##*#}"; \
    if [ "$crate" != "{{version}}" ]; then \
        echo "tag {{version}} does not match crate version $crate" >&2; exit 1; \
    fi

# Build the static release artifacts and their checksums into dist/
dist version: (check-version version)
    rm -rf dist && mkdir -p dist
    set -e; for target in {{release_targets}}; do \
        cargo build --release --locked --target "$target" -p walltime; \
        name="walltime-{{version}}-$target"; \
        tar -czf "dist/$name.tar.gz" -C "target/$target/release" walltime; \
        ( cd dist && sha256sum "$name.tar.gz" > "$name.tar.gz.sha256" ); \
    done
    @ls -1 dist

# Print the SRI hashes nix/binary.nix pins, in nixpkgs system order
dist-hashes:
    @for target in {{release_targets}}; do \
        system="${target%%-*}-linux"; \
        hash="$(nix hash file --sri --type sha256 dist/walltime-*-"$target".tar.gz)"; \
        printf '"%s" = "%s";\n' "$system" "$hash"; \
    done

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
