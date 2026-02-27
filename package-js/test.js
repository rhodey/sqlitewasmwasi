import { open } from './index.js'

function toString(obj) {
  const o = { ...obj }
  Object.keys(o).forEach((key) => {
    if (typeof o[key] !== 'bigint') { return }
    o[key] += 'n'
  })
  return JSON.stringify(o)
}

function equals(actual, expected, msg) {
  actual = typeof actual === 'object' ? toString(actual) : actual
  expected = typeof expected === 'object' ? toString(expected) : expected
  if (actual === expected) {
    console.log('pass', msg)
  } else {
    console.log('fail', msg)
    console.log('actual >', actual)
    console.log('expected', expected)
  }
}

// todo: transactions
// todo: test blob type
// todo: test use after close
// todo: test use after release
const test = () => {
  console.log('test')
  const db = open('/app/test.js.db')
  db.exec('drop table if exists demo')

  let num = db.exec('create table demo (id integer, name text, note text, ratio real, big_int integer)')
  equals(num, 0n, 'create table no rows')

  let statement = db.prepare('insert into demo (id, name, note, ratio, big_int) values (?, ?, ?, ?, ?)')
  let info = statement.run([1, 'hello from js', null, 3.25, 9007199254740993n])
  equals(info.changes, 1n, 'insert 1 row')
  equals(info.lastInsertRowid, 1n, 'row id 1')
  equals(true, statement.release(), 'release true')
  equals(false, statement.release(), 'release false')

  statement = db.prepare('insert into demo (id, name, note, ratio, big_int) values (?, ?, ?, ?, ?)')
  info = statement.run([2, 'hello from js', null, 3.25, 9007199254740993n])
  equals(info.changes, 1n, 'insert 1 row')
  equals(info.lastInsertRowid, 2n, 'row id 2')
  equals(true, statement.release(), 'release true')
  equals(false, statement.release(), 'release false')

  const obj1 = { id: 1n, name: 'hello from js', note: null, ratio: 3.25, big_int: 9007199254740993n }
  const obj2 = { ...obj1, id: 2n }

  statement = db.prepare('select id, name, note, ratio, big_int from demo where id = 1')
  let row = statement.one()
  equals(row, obj1, 'select 1 row A')

  statement = db.prepare('select id, name, note, ratio, big_int from demo where id = ?')
  row = statement.one([1])
  equals(row, obj1, 'select 1 row B')

  statement = db.prepare('select id, name, note, ratio, big_int from demo where id = ?')
  row = statement.one([2])
  equals(row, obj2, 'select 1 row C')

  statement = db.prepare('select id, name, note, ratio, big_int from demo where id = 3')
  row = statement.one()
  equals(row, null, 'select 1 row NULL')

  statement = db.prepare('select id, name, note, ratio, big_int from demo order by id')
  let rows = statement.all()
  equals(rows.length, 2, 'select 2 rows')
  equals(rows[0], obj1, 'select row id 1')
  equals(rows[1], obj2, 'select row id 2')

  statement = db.prepare('select id, name, note, ratio, big_int from demo where id = ?')
  rows = statement.all([1])
  equals(rows.length, 1, 'select 1 rows')
  equals(rows[0], obj1, 'select row id 1')

  statement = db.prepare('select id, name, note, ratio, big_int from demo where id = ?')
  rows = statement.all([3])
  equals(rows.length, 0, 'select 0 rows')

  num = db.exec('update demo set id = 3 where id = ?', [1])
  equals(num, 1n, 'update 1 rows')
  num = db.exec('update demo set id = 3 where id = ?', [1])
  equals(num, 0n, 'update 0 rows')
  num = db.exec('delete from demo where 1 = ?', [1])
  equals(num, 2n, 'delete 2 rows')

  // without names
  statement = db.prepare('select 3 where 1 = 1')
  row = statement.one()
  equals(row, { '3': 3n }, 'select 3 col name')

  // strict types
  db.exec('drop table if exists nums')
  db.exec('create table nums (id integer, ratio real) strict')
  statement = db.prepare('insert into nums (id, ratio) values (?, ?)')
  info = statement.run([1, 3.25])
  equals(info.changes, 1n, 'insert 1 real')
  info = statement.run([2, 2])
  equals(info.changes, 1n, 'insert 1 int as real')
  info = statement.run([3, 3n])
  equals(info.changes, 1n, 'insert 1 bigint as real')

  try {
    statement.run([4, 'abc'])
    console.log('fail', 'insert text as real throws')
  } catch (err) {
    console.log('pass', 'insert text as real throws')
  }

  statement = db.prepare('select * from nums order by id')
  rows = statement.all()
  equals(rows.length, 3, 'select 3 rows')
  equals(rows[0], { id: 1n, ratio: 3.25 }, 'select row id 1')
  equals(rows[1], { id: 2n, ratio: 2.0 }, 'select row id 2')
  equals(rows[2], { id: 3n, ratio: 3.0 }, 'select row id 3')

  // not strict types
  db.exec('drop table if exists nums')
  db.exec('create table nums (id integer, ratio real)')
  statement = db.prepare('insert into nums (id, ratio) values (?, ?)')
  info = statement.run([1, 'abc'])
  equals(info.changes, 1n, 'insert 1 real')

  statement = db.prepare('select * from nums order by id')
  rows = statement.all()
  equals(rows.length, 1, 'select 1 rows')
  equals(rows[0], { id: 1n, ratio: 'abc' }, 'select row id 1')

  // done
  db.close()
  equals(1, 1, 'close')
}

export const run = {
  run() {
    try {
      test()
    } catch (err) {
      console.log('!! error', err)
    }
  }
}
