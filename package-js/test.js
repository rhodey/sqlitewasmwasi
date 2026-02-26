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

// todo: test select no col name
// todo: test insert bigint in real col
const test = () => {
  console.log('test')
  const db = open('file:/app/test.js.db?vfs=unix-dotfile')
  db.exec('drop table if exists demo')

  let num = db.exec('create table demo (id integer, name text, note text, ratio real, big_id integer)')
  equals(num, 0n, 'create table no rows')

  let statement = db.prepare('insert into demo (id, name, note, ratio, big_id) values (?, ?, ?, ?, ?)')
  let info = statement.run([1, 'hello from rust', null, 3.25, 9007199254740993n])
  equals(info.changes, 1n, 'insert 1 row')
  equals(info.lastInsertRowid, 1n, 'row id 1')
  equals(true, statement.release(), 'release true')
  equals(false, statement.release(), 'release false')

  statement = db.prepare('insert into demo (id, name, note, ratio, big_id) values (?, ?, ?, ?, ?)')
  info = statement.run([2, 'hello from rust', null, 3.25, 9007199254740993n])
  equals(info.changes, 1n, 'insert 1 row')
  equals(info.lastInsertRowid, 2n, 'row id 2')
  equals(true, statement.release(), 'release true')
  equals(false, statement.release(), 'release false')

  const obj1 = { id: 1n, name: 'hello from rust', note: null, ratio: 3.25, big_id: 9007199254740993n }
  const obj2 = { ...obj1, id: 2n }

  statement = db.prepare('select id, name, note, ratio, big_id from demo where id = 1')
  let row = statement.one()
  equals(row, obj1, 'select 1 row A')

  statement = db.prepare('select id, name, note, ratio, big_id from demo where id = ?')
  row = statement.one([1])
  equals(row, obj1, 'select 1 row B')

  statement = db.prepare('select id, name, note, ratio, big_id from demo where id = ?')
  row = statement.one([2])
  equals(row, obj2, 'select 1 row C')

  statement = db.prepare('select id, name, note, ratio, big_id from demo where id = 3')
  row = statement.one()
  equals(row, null, 'select 1 row NULL')

  statement = db.prepare('select id, name, note, ratio, big_id from demo order by id')
  let rows = statement.all()
  equals(rows.length, 2, 'select 2 rows')
  equals(rows[0], obj1, 'select row id 1')
  equals(rows[1], obj2, 'select row id 2')

  statement = db.prepare('select id, name, note, ratio, big_id from demo where id = ?')
  rows = statement.all([1])
  equals(rows.length, 1, 'select 1 rows')
  equals(rows[0], obj1, 'select row id 1')

  statement = db.prepare('select id, name, note, ratio, big_id from demo where id = ?')
  rows = statement.all([3])
  equals(rows.length, 0, 'select 0 rows')

  num = db.exec('update demo set id = 3 where id = ?', [1])
  equals(num, 1n, 'update 1 rows')

  num = db.exec('update demo set id = 3 where id = ?', [1])
  equals(num, 0n, 'update 0 rows')

  num = db.exec('delete from demo where 1 = ?', [1])
  equals(num, 2n, 'delete 2 rows')

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
