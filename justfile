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

# run ai
run-ai:
  @cargo run -p chaos-symphony-ai

# run replication
run-replication:
  @cargo run -p chaos-symphony-replication

# run simulation
run-simulation:
  @cargo run -p chaos-symphony-simulation

# test
test:
  @cargo test
