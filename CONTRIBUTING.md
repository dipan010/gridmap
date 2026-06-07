# Contributing to gridmap

## Prerequisites

- Rust (stable, edition 2021)
- Python >= 3.9
- [maturin](https://www.maturin.rs/) (`pip install maturin`)

## Building from source

```bash
# Clone the repository
git clone https://github.com/dipanghosh/gridmap.git
cd gridmap

# Install Python dependencies
pip install maturin openpyxl pytest

# Build and install the extension in development mode
maturin develop --release
```

## Running tests

```bash
# Rust unit tests
cargo test --workspace

# Python tests (requires maturin develop first)
python -m pytest python/tests/ -v

# Benchmark harness
python bench/harness/run_harness.py
```

## Code quality

```bash
# Rust
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Python
python -m pytest python/tests/ -v
```

## PR guidelines

1. One logical change per PR
2. All existing tests must pass (`cargo test` + `pytest`)
3. New public functions require tests and doc comments
4. Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `test:`, `refactor:`, `docs:`, `chore:`
5. No `TODO` comments — track deferred work in issues
6. Rust code: `cargo clippy -- -D warnings` and `cargo fmt` must pass
7. Python code: type hints on public functions, frozen dataclasses for return types

## Architecture

See [CLAUDE.md](CLAUDE.md) for detailed architecture rules, module ownership, and dependency constraints. The key principle: Rust core owns all logic, Python handles only file I/O and API wrapping.

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache-2.0.
