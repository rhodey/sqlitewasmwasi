# SQLiteWasmWasi
SQLite WASI P2 component.

Use SQLite from WASI P2. Run using [wasmtime](https://github.com/bytecodealliance/wasmtime#installation) or [other](https://github.com/yoshuawuyts/awesome-wasm-components#host-runtimes).

[Rusqlite](https://crates.io/crates/rusqlite) is used internally but the API exposed is closer to [better-sqlite3](https://www.npmjs.com/package/better-sqlite3).

## Build component (docker)
```
just component-docker
```

## Build component (no docker)
+ [Install WASI SDK](https://github.com/WebAssembly/wasi-sdk#install)
+ `cp example.env .env` && edit .env
+ `just component`

## Example js
JS to WASM by the power of [ComponentizeJS](https://github.com/bytecodealliance/ComponentizeJS)
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

## Example rust
Rust to WASM by the power of [wasm32-wasip2](https://doc.rust-lang.org/nightly/rustc/platform-support/wasm32-wasip2.html)
```rust
use sqlite_wasm_wasi::{open, Value};

fn main() {
    if let Err(err) = example() {
        println!("!! error {:?}", err);
    }
}

fn example() -> Result<(), sqlite_wasm_wasi::Error> {
    let db = open("/app/example.rust.db")?;
    db.exec("drop table if exists example", &[])?;
    db.exec(
        "create table example (id integer, name text, note text, ratio real, big_int integer)",
        &[],
    )?;

    let mut insert =
        db.prepare("insert into example (id, name, note, ratio, big_int) values (?, ?, ?, ?, ?)")?;
    let mut info = insert.run(&[
        Value::Integer(1),
        Value::Text("hello from js".to_string()),
        Value::Null,
        Value::Real(3.25),
        Value::Integer(9_007_199_254_740_993),
    ])?;
    println!("{} == 1", info.changes);
    println!("{} == 1", info.last_insert_rowid);

    info = insert.run(&[
        Value::Integer(2),
        Value::Text("hello from js".to_string()),
        Value::Null,
        Value::Real(3.25),
        Value::Integer(9_007_199_254_740_993),
    ])?;
    println!("{} == 1", info.changes);
    println!("{} == 2", info.last_insert_rowid);

    let mut select = db.prepare("select id, name, note, ratio, big_int from example where id = ?")?;
    let row = select.one(&[Value::Integer(1)])?;
    println!("{:?}", row);

    select = db.prepare("select * from example where 1 = ? order by id")?;
    let mut rows = select.all(&[Value::Integer(1)])?;
    println!("{:?}", rows);

    db.exec("drop table if exists txn", &[])?;
    db.exec("create table txn (id integer)", &[])?;
    insert = db.prepare("insert into txn (id) values (?)")?;
    let mut insert_many = db.transaction(|nums: Vec<i64>| {
        for num in nums {
            insert.run(&[Value::Integer(num)])?;
        }
        Ok(())
    });

    let nums = vec![1, 4, 5, 6];
    insert_many(nums)?;

    select = db.prepare("select * from txn order by id")?;
    rows = select.all(&[])?;
    println!("{:?}", rows);
    db.close()?;

    Ok(())
}
```

## Example other
The JS and Rust packages are convenient wrappers around `component.wasm`.

After build you will have `target/wasm32-wasip2/release/component.wasm` and [sqlite.wit](https://github.com/rhodey/sqlitewasmwasi/blob/main/wit/sqlite.wit).

These two files are all that is needed to use SQLiteWasmWasi [from another language](https://github.com/yoshuawuyts/awesome-wasm-components#programming-language-support).

## Notes js
`let stmt = db.prepare('sql')` returns a "prepared statement".

Prepared statements can be run 1 or 100 or more times.

If your app is eg an HTTP server you need to be calling `stmt.release()` when done with it.

Rust has the `Drop` trait and does not need release.

ComponentizeJS may improve in the future.

## Notes
JS and Rust both detault to ["unix-dotfile" VFS](https://sqlite.org/vfs.html).

Unix-dotfile VFS is exactly like default "unix" except it avoids POSIX `flock()` system calls.

[Lock.host](https://github.com/rhodey/lock.host) will be forking and trying to upstream wasmtime flock support.

If you like SQLite you may also like [SQLitesuperfs](https://github.com/rhodey/sqlitesuperfs).

## Test
```
curl https://wasmtime.dev/install.sh -sSf | bash
cargo install wac-cli
just test
```

## License
hello@lock.host

MIT
