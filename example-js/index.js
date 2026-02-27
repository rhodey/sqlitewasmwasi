import { open } from 'sqlite-wasm-wasi'

const example = () => {
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
}

export const run = {
  run() {
    try {
      example()
    } catch (err) {
      console.log('!! error', err)
    }
  }
}
