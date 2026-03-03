# sqlite-wasm-wasi
Use SQLite from WASM WASI. See [parent](https://github.com/rhodey/sqlitewasmwasi)

## Example
See [full example](https://github.com/rhodey/sqlitewasmwasi/tree/main/example-js)
```js
import { open } from 'sqlite-wasm-wasi'

const db = open('/app/example.js.db')
db.exec('drop table if exists example')
db.exec('create table example (id integer, name text, note text, ratio real, big_int integer)')

let insert = db.prepare('insert into example (id, name, note, ratio, big_int) values (?, ?, ?, ?, ?)')
let info = insert.run([1, 'hello from js', null, 3.25, 9007199254740993n])
console.log(info.changes, '== 1')
console.log(info.lastInsertRowid, '== 1')

info = insert.run([2, 'hello from js', null, 3.25, 9007199254740993n])
console.log(info.changes, '== 1')
console.log(info.lastInsertRowid, '== 2')

let select = db.prepare('select id, name, note, ratio, big_int from example where id = ?')
let row = select.one([1])
console.log(row) // >> { id: 1n, name: 'hello from js', note: null, ratio: 3.25, big_int: 9007199254740993n }

select = db.prepare('select * from example where 1 = ? order by id')
let rows = select.all([1])
console.log(rows) // >> [ ..., ... ]

// transactions
db.exec('drop table if exists txn')
db.exec('create table txn (id integer)')
insert = db.prepare('insert into txn (id) values (?)')
let insertMany = db.transaction((nums) => {
  for (const num of nums) { insert.run([num]) }
})

const nums = [1, 4, 5, 6]
insertMany(nums)

select = db.prepare('select * from txn order by id')
rows = select.all()
console.log(rows) // { id: 1n }, { id: 4n }, { id: 5n }, { id: 6n }
db.close()
```

## Setup
If you want to build `component.wasm` yourself, see: [parent](https://github.com/rhodey/sqlitewasmwasi)

Otherwise get `component.wasm` from `node_modules/sqlite-wasm-wasi/dist/` after npm install sqlite-wasm-wasi

## Notes
`let stmt = db.prepare('sql')` returns a "prepared statement".

Prepared statements can be run 1 or 100 or more times.

If your app is eg an HTTP server you need to be calling `stmt.release()` when done with it.

Rust has the `Drop` trait and does not need release.

ComponentizeJS may improve in the future.

## Notes
`open()` detaults to ["unix-dotfile" VFS](https://sqlite.org/vfs.html).

Unix-dotfile VFS is exactly like default "unix" except it avoids POSIX `flock()` system calls.

[Lock.host](https://github.com/rhodey/lock.host) will be forking and trying to upstream wasmtime flock support.

## License
hello@lock.host

MIT
