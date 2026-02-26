import { open } from './index.js'

function toString(obj) {
  const o = { ...obj }
  Object.keys(o).forEach((key) => {
    if (typeof o[key] === 'bigint') {
      o[key] = `${o[key]}n`
    } else {
    }
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

const test = () => {
  console.log('hi hi')
  const db = open('file:/app/test.js.db?vfs=unix-dotfile')
  db.exec('drop table if exists demo')

  let num = db.exec('create table demo (id integer, name text, note text, ratio real, big_id integer)')
  equals(num, 0n, 'create table no rows')

  let statement = db.prepare('insert into demo (id, name, note, ratio, big_id) values (?, ?, ?, ?, ?)')
  let info = statement.run([1, 'hello from rust', null, 3.25, 9007199254740993n])
  equals(info.changes, 1n, 'insert 1 row')
  equals(true, statement.release(), 'release true')
  equals(false, statement.release(), 'release false')

  statement = db.prepare('insert into demo (id, name, note, ratio, big_id) values (?, ?, ?, ?, ?)')
  info = statement.run([2, 'hello from rust', null, 3.25, 9007199254740993n])
  equals(info.changes, 1n, 'insert 1 row')
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

  statement = db.prepare('select id, name, note, ratio, big_id from demo')
  let rows = statement.all()
  for (const row of rows) {
    console.log(123, row)
  }

  statement = db.prepare('select id, name, note, ratio, big_id from demo where 1 = ?')
  rows = statement.all([1])
  for (const row of rows) {
    console.log(456, row)
  }

  statement = db.prepare('select id, name, note, ratio, big_id from demo where 1 = ?')
  rows = statement.all([3])
  for (const row of rows) {
    console.log(789, row)
  }

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
