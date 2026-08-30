# Agents Guide

This file documents the conventions every agent (human or AI) must follow
when editing this repository.

## Comment style

Comments placed on top of variables, functions, types, or any other code
logic must be **at most 2 lines**.

If a comment needs more than 2 lines to explain something, summarise it in
2 lines and move the full reasoning into a regular commit message, an ADR,
or another design document — never into a comment block. Long comments
hide between vertical folds and rot out of sync with the code they sit
above; short comments stay close to the point and force the writer to keep
them sharp.

This rule applies to every file in the repository: Rust source under
`src/`, TypeScript and Svelte source under `web/src/`, CSS files,
TOML/JSON/JS configuration files, and anywhere else comments are written.

The rule includes all comment flavours:

- Rust `///` and `//!` doc comments.
- Rust `//` line comments and `/* ... */` block comments.
- TypeScript/JavaScript `//`, `/* ... */`, and JSDoc `/** ... */`.
- Svelte `<!-- ... -->` template comments.
- CSS `/* ... */` comments.
