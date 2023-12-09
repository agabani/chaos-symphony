set shell := ["bash", "-uc"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# help
help:
  @just --list

# build
build:
  @cargo build

# format
format:
  @cargo fmt

# lint
lint: lint-code lint-format

# lint code
lint-code:
  @cargo clippy --workspace

# lint format
lint-format:
  @cargo fmt -- --check

# test
test:
  @cargo test
