# Rezis

Rezis is a **NestJS-inspired** Rust backend framework built on **Axum**. It helps you structure APIs with **modules**, **controllers**, and **services**, **DTO validation**, consistent JSON **success/error envelopes**, **logging**, **CORS**, **`.env`-based config**, and a **`rezis`** CLI for scaffolding.

## Status

Rezis is currently in **alpha**: APIs and defaults may change between releases.

## Install CLI

```bash
cargo install rezis-cli
```

For development from this repo:

```bash
cargo install --path crates/rezis-cli
```

## Create an app

```bash
rezis new my-api
cd my-api
cargo run
```

The scaffold includes a **health** route (see below). Set `PORT` in `.env` if you need something other than the default (template uses `3000`).

## Generate a resource

```bash
rezis g resource users
```

Regenerate or restart the server after adding routes:

```bash
cargo run
```

## Try endpoints

With the server running (`cargo run`), from another terminal:

```bash
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/users
```

`/health` is included in a new project. `/users` is available after `rezis g resource users` (or your own module implementing that path).

## What Rezis v1 includes

- Axum underneath
- Modules, controllers, and services (NestJS-like layout)
- DTO validation (`validator`)
- JSON success/error envelopes
- Logging and CORS helpers
- `.env` config (e.g. `PORT`)
- CLI: `rezis new`, `rezis g …`

## Out of scope for v1

- Database abstraction
- Auth module
- OpenAPI
- WebSockets
- Queues
- Proc macros
- Full dependency-injection container

More detail for contributors: **[IMPLEMENTATION_GUIDE.md](IMPLEMENTATION_GUIDE.md)**.
