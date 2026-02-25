# wasm-wasi-sqlite

WASM WASI P2 SQLite component.

## Goals

The goal is to create a WASM WASI P2 interface and component for SQLite.

It needs to be possible to use the SQLite component with a Rust WASM app which uses `target = wasm32-wasip2`.

It needs also to be possible to use the SQLite component with a JS WASM app which uses ComponentizeJS and WASI P2.

Both WASM apps need to run using wasmtime.

Try first to make the component using the rust crate [rusqlite](https://github.com/rusqlite/rusqlite).

We are going to try to make rusqlite work for awhile and only if we really need to we will use Rust bindings to SQLite C code and do it ourselves.

We need to start by defining a simple WASI WIT Interface it should be just enough to:

+ open db
+ create a prepared statement
+ run a prepared statement and return results
+ close db

I think it is important that we somehow compile using a WASM WASI compatible libc.

## What is in this repo now

- A first-pass WIT interface in `wit/sqlite.wit` with just enough surface area for:
  - open DB
  - create prepared statement
  - run prepared statement and return rows
  - close DB
- A Rust implementation in `sqlite-component` using `rusqlite` (`libsqlite3-sys` with `bundled` SQLite).
- A Rust WASI P2 client example (`examples/rust-client`).
- A JS WASI P2 client example intended for ComponentizeJS (`examples/js-client`).

## WIT interface

The interface is defined in `wit/sqlite.wit`.

```wit
open: func(path: string) -> result<db-handle, sqlite-error>
prepare: func(db: db-handle, sql: string) -> result<statement-handle, sqlite-error>
query: func(statement: statement-handle) -> result<list<sqlite-row>, sqlite-error>
close-db: func(db: db-handle) -> result<_, sqlite-error>
close-statement: func(statement: statement-handle) -> result<_, sqlite-error>
```

Rows are returned as `list<sqlite-value>` where `sqlite-value` supports null/int/real/text/blob.

## libc / WASI compatibility note

This implementation intentionally starts from `rusqlite` and enables the `bundled` SQLite build.
That keeps us on the `rusqlite` path while allowing SQLite C code to be compiled for WASI targets
through the Rust target toolchain (including WASI-compatible libc support in the target environment).

## Build and validate locally

This repository is focused on the WASM WASI P2 component workflow.

### End-to-end validation with wasmtime

Run the scripted test that:
- builds the SQLite component for `wasm32-wasip2`,
- builds the Rust client for `wasm32-wasip2`,
- composes them into one component,
- runs with `wasmtime`, and
- verifies stdout includes the expected log lines.

```bash
./scripts/test-wasmtime-rust-client.sh
```

Expected output includes:

```text
int=1
text=hello from rust
```

Tooling expected:
- `wac` (`cargo install wac-cli`)
- `wasmtime`
- Rust target `wasm32-wasip2` (`rustup target add wasm32-wasip2`)
- `wasi-sdk`/WASI sysroot available for building bundled SQLite C code (the script auto-detects `/opt/wasi-sdk-25`)

## Next steps

1. Add parameter binding to prepared statements.
2. Add explicit execute-vs-query split for non-SELECT statements.
3. Add transaction support.
4. Add a reproducible compose script (once final toolchain choice is fixed).

## License

mike@rhodey.org

MIT
