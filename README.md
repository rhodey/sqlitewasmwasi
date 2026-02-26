# SQLiteWASI
WASM WASI P2 SQLite component.

## Build
### Docker
```
just sqlite-docker
```

### No docker
+ [Install WASI SDK](https://github.com/WebAssembly/wasi-sdk#install)
+ cp example.env .env
+ edit .env
+ just sqlite

## Run
```
just build-rust run-rust
just build-js run-js
```

## License
hello@lock.host

MIT
