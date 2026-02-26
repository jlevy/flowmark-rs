# Makefile for local development workflows.

.DEFAULT_GOAL := default

.PHONY: default format format-rust format-docs

# Run both Rust code formatting and markdown formatting.
default: format

format: format-rust format-docs

format-rust:
	cargo fmt --all

# Use the Rust CLI in this repo to format all markdown recursively.
format-docs:
	cargo run --quiet --bin flowmark -- --auto .
