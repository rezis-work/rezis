# users-api

Copy environment template (optional; defaults match below):

```bash
cp .env.example .env
```

Run:

```bash
cargo run -p users-api
```

The server reads **`PORT`** from `.env` via **`RezisApp::listen_from_env`** in the `rezis` crate.
