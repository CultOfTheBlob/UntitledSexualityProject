default: run

run:
  @trunk serve

fmt:
  @cargo fmt

lint: fmt
  @cargo clippy -- -D warnings

test: lint
  @cargo test

build: test
  @nix build
