# AGENTS.md — Autotext

## What this is
A Rust desktop application that provides automatic text insertion/expansion text templates, with an encrypted SQLite database and a GTK GUI.

## Stack
- Rust (edition 2021)
- GTK 3 / glib / gio / gdk-pixbuf GUI
- SQLite (rusqlite)
- openssl, blake2, cryptostream, base64 (encryption)
- rand, regex

## Build
```bash
cargo build --release
```

## Run
```bash
./run.sh  # or cargo run
```

## Structure
- `src/main.rs` — application entry point
- `src/lib.rs` — library logic (mylib)
- `src/database.rs` — SQLite storage layer
- `settings.db` — default database
- `databases/`, `resources/` — data and assets
- `Cargo.toml` — dependencies

## Conventions
- No comments in code unless asked.
- Verify: `cargo check && cargo build`