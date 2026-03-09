# walltime

A library and CLI for measuring time spent in a process

## Architecture

This is a Cargo workspace with crates under `crates/`:

- **walltime-core**: Core library with domain logic and error types
- **walltime-cli**: CLI binary application

## Development

### Nix (recommended)

This project uses a Nix flake for reproducible development:

```sh
direnv allow    # auto-activates the dev shell
# or manually:
nix develop     # enter the dev shell
```

### Common commands

```sh
just            # list all available commands
just fmt        # format code
just lint       # run clippy
just test       # run all tests
just ci         # run all CI checks locally
```

### Snapshot testing

Uses [insta](https://insta.rs/) for snapshot tests:

```sh
just snap       # run tests and review snapshot changes
```

## Conventions

- Use `thiserror` for error types; avoid `unwrap()` (denied by clippy lint)
- All public items should have doc comments
- Run `just ci` before pushing
- Use insta snapshots for parser output and complex data structures
