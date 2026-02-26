# SQLiteWASI
SQLite WASI P2 component.

Use SQLite from WASI. Run using [wasmtime](https://github.com/bytecodealliance/wasmtime#installation) or [one of these](https://github.com/yoshuawuyts/awesome-wasm-components#host-runtimes).

## Build component (docker)
```
just component-docker
```

## Build component (no docker)
+ [Install WASI SDK](https://github.com/WebAssembly/wasi-sdk#install)
+ `cp example.env .env` && edit .env
+ `just component`

## Example JS
```js
import { open } from 'sqlite-wasi'

const db = open('/app/test.js.db')
db.exec('drop table if exists demo')
db.exec('create table demo (id integer, name text, note text, ratio real, big_id integer)')

let statement = db.prepare('insert into demo (id, name, note, ratio, big_id) values (?, ?, ?, ?, ?)')
let info = statement.run([1, 'hello from js', null, 3.25, 9007199254740993n])
console.log(info.changes, '== 1')
console.log(info.lastInsertRowid, '== 1')

info = statement.run([2, 'hello from js', null, 3.25, 9007199254740993n])
console.log(info.changes, '== 1')
console.log(info.lastInsertRowid, '== 2')

statement = db.prepare('select id, name, note, ratio, big_id from demo where id = ?')
let row = statement.one([1])
console.log(row) // >> { id: 1n, name: 'hello from js', note: null, ratio: 3.25, big_id: 9007199254740993n }

statement = db.prepare('select * from demo where 1 = ? order by id')
let rows = statement.all([1])
console.log(rows) // >> [ ... ]
```

## Run tests
Install [wasmtime](https://github.com/bytecodealliance/wasmtime#installation) then:
```
just test
```

## License
hello@lock.host

MIT
