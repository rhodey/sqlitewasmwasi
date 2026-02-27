# SQLiteWasmWasi
SQLite WASI P2 component.

Use SQLite from WASI. Run using [wasmtime](https://github.com/bytecodealliance/wasmtime#installation) or [other](https://github.com/yoshuawuyts/awesome-wasm-components#host-runtimes).

[Rusqlite](https://crates.io/crates/rusqlite) is used internally but the API exposed is closer to [better-sqlite3](https://www.npmjs.com/package/better-sqlite3).

## Build component (docker)
```
just component-docker
```

## Build component (no docker)
+ [Install WASI SDK](https://github.com/WebAssembly/wasi-sdk#install)
+ `cp example.env .env` && edit .env
+ `just component`

## Example JS
JS to WASM by the power of [ComponentizeJS](https://github.com/bytecodealliance/ComponentizeJS)
```js
import { open } from 'sqlite-wasm-wasi'

const db = open('/app/test.js.db')
db.exec('drop table if exists demo')
db.exec('create table demo (id integer, name text, note text, ratio real, big_int integer)')

let statement = db.prepare('insert into demo (id, name, note, ratio, big_int) values (?, ?, ?, ?, ?)')
let info = statement.run([1, 'hello from js', null, 3.25, 9007199254740993n])
console.log(info.changes, '== 1')
console.log(info.lastInsertRowid, '== 1')

info = statement.run([2, 'hello from js', null, 3.25, 9007199254740993n])
console.log(info.changes, '== 1')
console.log(info.lastInsertRowid, '== 2')

statement = db.prepare('select id, name, note, ratio, big_int from demo where id = ?')
let row = statement.one([1])
console.log(row) // >> { id: 1n, name: 'hello from js', note: null, ratio: 3.25, big_int: 9007199254740993n }

statement = db.prepare('select * from demo where 1 = ? order by id')
let rows = statement.all([1])
console.log(rows) // >> [ ..., ... ]
```

## Example Rust
Rust to WASM by the power of [wasm32-wasip2](https://doc.rust-lang.org/nightly/rustc/platform-support/wasm32-wasip2.html)
```rust
// todo
```

## Run tests
Install [wasmtime](https://github.com/bytecodealliance/wasmtime#installation) then:
```
just test
```

## License
hello@lock.host

MIT
