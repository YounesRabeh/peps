# Testing Peps

This directory contains the Rust integration tests for the Peps language
implementation. Each file is compiled as a separate test target and exercises
the public API exposed by the `peps` crate.

## Run the tests

From the repository root, run the complete Rust suite:

```sh
cargo test --all-targets --all-features
```

Run one test file while working on a particular layer:

```sh
cargo test --test lexer_tests
cargo test --test parser_tests
```

Run one named test:

```sh
cargo test --test lexer_tests lexes_emoji_number
```

The release workflow also requires formatting and Clippy to pass:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Test suites

| File | Covers |
| --- | --- |
| `lexer_tests.rs` | Source text becoming tokens, including invalid syntax diagnostics. |
| `parser_tests.rs` | Tokens becoming the Peps abstract syntax tree. |
| `semantic_tests.rs` | Type checking, scopes, and semantic error diagnostics. |
| `compiler_tests.rs` | Compilation from source to bytecode instructions. |
| `vm_tests.rs` | Bytecode execution and observable program output. |
| `example_programs_tests.rs` | Every `.peps` file under `examples/` executes successfully. |

## Adding a test

Add behavior-specific tests to the file for the affected layer. Prefer testing
through the smallest appropriate public API:

- Lexing: `lexer::lex`
- Parsing: `parser::parse(lexer::lex(source)?)`
- Semantic checks: `semantic::check`
- Compilation: `compile_source`
- End-to-end execution: `run_source`

When changing the language syntax or behavior, include both successful and
invalid-input cases where applicable. Add or update a runnable `.peps` example
when the change affects documented language features; `example_programs_tests`
will execute it automatically.

## Browser IDE tests

The React/Vite IDE tests live separately in `ide/`. Run them from that
directory:

```sh
pnpm test -- --run
```

For the full local release gate, including the IDE build, see
[Development, Testing, and Releases](../docs/README.md#test-before-building-artifacts).
