# GitHub repository metadata (maintainers)

Apply these in the repository **About** settings on GitHub (description and topics are not stored in git).

## Description

```txt
A NestJS-inspired Rust backend framework built on Axum.
```

## Topics

Suggested labels:

```txt
rust
axum
tokio
backend
web-framework
nestjs
rest-api
cli
validation
microservices
```

## `rezis-macros`

Per project policy: **do not publish an empty procedural-macro crate.** Macros are **planned for a later release** once there is at least one useful derive or helper. Until then, no `rezis-macros` workspace crate is required; optional local experimentation is fine but should stay unpublishable or undocumented until ready.

## Crates.io metadata (before real publish)

`cargo package` / `cargo publish` emit warnings until **`repository`** (and optionally **`homepage`**, **`documentation`**) are set on the crates you publish. Add **`[package]`** keys in [`crates/rezis/Cargo.toml`](../crates/rezis/Cargo.toml) and [`crates/rezis-cli/Cargo.toml`](../crates/rezis-cli/Cargo.toml), or inherit from workspace **`[workspace.package]`** in the root [`Cargo.toml`](../Cargo.toml) once you have the Git URL.

Workspace crates ship as **`0.1.0-alpha.1`**. CLI **`rezis new`** writes **`rezis = "0.1.0-alpha.1"`** unless **`--rezis-path`** is passed.
