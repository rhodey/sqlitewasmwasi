# wasm-wasi-sqlite
WASM WASI P2 SQlite component

## Goals
The goal is to create a WASM WASI P2 interface and component for SQLite

It needs to be possible to use the SQLite component with a Rust WASM app which uses target = wasm32-wasip2

It needs also to be possible to use the SQLite component with a JS WASM app which uses ComponentizeJS and wasip2

Both WASM apps need to run using wasmtime

Try first to make the component using the rust crate [rusqlite](https://github.com/rusqlite/rusqlite)

We are going to try to make rusqlite work for awhile and only if we really need to we will use Rust bindings to SQLite C code and do it ourselves

We need to start by defining a simple WASI WIT Interface it should be just enough to:

+ open db
+ create a prepared statement
+ run a prepared statement and return results
+ close db

I think it is important that we somehow compile using a WASM WASI compatible libc

Ask for clarification as needed

## License
mike@rhodey.org

MIT
