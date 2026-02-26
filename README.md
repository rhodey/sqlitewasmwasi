# SQLiteWASI
WASI P2 SQLite component.

Use SQLite from WASM apps. Run using [wasmtime](https://github.com/bytecodealliance/wasmtime) or [one of these](https://github.com/yoshuawuyts/awesome-wasm-components#host-runtimes).

## Build component (docker)
```
just component-docker
```

## Build component (no docker)
+ [Install WASI SDK](https://github.com/WebAssembly/wasi-sdk#install)
+ `cp example.env .env` && edit .env
+ `just component`

## Run examples
```
just build-rust run-rust
just build-js run-js
```

## License
hello@lock.host

MIT
