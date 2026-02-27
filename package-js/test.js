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
    console.log('error', msg)
    console.log('actual >', actual)
    console.log('expected', expected)
  }
}

function equalsBlob(actual, expected, msg) {
  const ok = actual instanceof Uint8Array &&
    expected instanceof Uint8Array &&
    actual.length === expected.length &&
    actual.every((val, idx) => val === expected[idx])
  if (!ok) {
    console.log('pass', msg)
  } else {
    console.log('error', msg)
    console.log('actual >', actual)
    console.log('expected', expected)
  }
}

const basic = () => {
  console.log('basic')
  const db = open('/app/test.js.db')
  db.exec('drop table if exists basic')

  let num = db.exec('create table basic (id integer, name text, note text, ratio real, big_int integer)')
  equals(num, 0n, 'create table no rows')

  let statement = db.prepare('insert into basic (id, name, note, ratio, big_int) values (?, ?, ?, ?, ?)')
  let info = statement.run([1, 'hello from js', null, 3.25, 9007199254740993n])
  equals(info.changes, 1n, 'insert 1 row')
  equals(info.lastInsertRowid, 1n, 'row id 1')
  equals(true, statement.release(), 'release true')
  equals(false, statement.release(), 'release false')

  statement = db.prepare('insert into basic (id, name, note, ratio, big_int) values (?, ?, ?, ?, ?)')
  info = statement.run([2, 'hello from js', null, 3.25, 9007199254740993n])
  equals(info.changes, 1n, 'insert 1 row')
  equals(info.lastInsertRowid, 2n, 'row id 2')
  equals(true, statement.release(), 'release true')
  equals(false, statement.release(), 'release false')

  const obj1 = { id: 1n, name: 'hello from js', note: null, ratio: 3.25, big_int: 9007199254740993n }
  const obj2 = { ...obj1, id: 2n }

  statement = db.prepare('select id, name, note, ratio, big_int from basic where id = 1')
  let row = statement.one()
  equals(row, obj1, 'select 1 row A')

  statement = db.prepare('select id, name, note, ratio, big_int from basic where id = ?')
  row = statement.one([1])
  equals(row, obj1, 'select 1 row B')

  statement = db.prepare('select id, name, note, ratio, big_int from basic where id = ?')
  row = statement.one([2])
  equals(row, obj2, 'select 1 row C')

  statement = db.prepare('select id, name, note, ratio, big_int from basic where id = 3')
  row = statement.one()
  equals(row, null, 'select 1 row NULL')

  statement = db.prepare('select id, name, note, ratio, big_int from basic order by id')
  let rows = statement.all()
  equals(rows.length, 2, 'select 2 rows')
  equals(rows[0], obj1, 'select row id 1')
  equals(rows[1], obj2, 'select row id 2')

  statement = db.prepare('select id, name, note, ratio, big_int from basic where id = ?')
  rows = statement.all([1])
  equals(rows.length, 1, 'select 1 rows')
  equals(rows[0], obj1, 'select row id 1')

  statement = db.prepare('select id, name, note, ratio, big_int from basic where id = ?')
  rows = statement.all([3])
  equals(rows.length, 0, 'select 0 rows')

  num = db.exec('update basic set id = 3 where id = ?', [1])
  equals(num, 1n, 'update 1 rows')
  num = db.exec('update basic set id = 3 where id = ?', [1])
  equals(num, 0n, 'update 0 rows')
  num = db.exec('delete from basic where 1 = ?', [1])
  equals(num, 2n, 'delete 2 rows')

  statement = db.prepare('select 3 where 1 = 1')
  row = statement.one()
  equals(row, { '3': 3n }, 'select 3 col name')

  db.close()
  equals(1, 1, 'close')
}

const strict = () => {
  console.log('strict')
  const db = open('/app/test.js.db')

  // strict
  db.exec('drop table if exists nums')
  db.exec('create table nums (id integer, ratio real) strict')
  let statement = db.prepare('insert into nums (id, ratio) values (?, ?)')
  let info = statement.run([1, 3.25])
  equals(info.changes, 1n, 'insert 1 real')
  info = statement.run([2, 2])
  equals(info.changes, 1n, 'insert 1 int as real')
  info = statement.run([3, 3n])
  equals(info.changes, 1n, 'insert 1 bigint as real')

  try {
    statement.run([4, 'abc'])
    console.log('error', 'insert text as real throws')
  } catch (err) {
    console.log('pass', 'insert text as real throws')
  }

  statement = db.prepare('select * from nums order by id')
  let rows = statement.all()
  equals(rows.length, 3, 'select 3 rows')
  equals(rows[0], { id: 1n, ratio: 3.25 }, 'select row id 1')
  equals(rows[1], { id: 2n, ratio: 2.0 }, 'select row id 2')
  equals(rows[2], { id: 3n, ratio: 3.0 }, 'select row id 3')

  // not strict
  db.exec('drop table if exists nums')
  db.exec('create table nums (id integer, ratio real)')
  statement = db.prepare('insert into nums (id, ratio) values (?, ?)')
  info = statement.run([1, 'abc'])
  equals(info.changes, 1n, 'insert 1 text')

  statement = db.prepare('select * from nums order by id')
  rows = statement.all()
  equals(rows.length, 1, 'select 1 rows')
  equals(rows[0], { id: 1n, ratio: 'abc' }, 'select row id 1')

  db.close()
  equals(1, 1, 'close')
}

const txn = () => {
  console.log('txn')
  const db = open('/app/test.js.db')
  db.exec('drop table if exists txn')

  db.exec('create table txn (id integer)')
  let insert = db.prepare('insert into txn (id) values (?)')
  let info = insert.run([1])
  equals(info.changes, 1n, 'insert 1 row')
  equals(info.lastInsertRowid, 1n, 'row id 1')

  const obj = [1n, 4n, 5n, 6n].map((n) => ({ id: n }))

  let select = db.prepare('select * from txn order by id')
  let rows = select.all()
  equals(rows.length, 1, 'select 1 rows')
  equals(rows[0], obj[0], 'select obj0')

  // commit
  let txn = db.transaction((nums) => {
    for (const num of nums) {
      info = insert.run([num])
      equals(info.changes, 1n, 'insert 1 row')
    }
  })

  const nums = obj.slice(1).map((obj) => obj.id)
  txn(nums)

  rows = select.all()
  equals(rows.length, obj.length, `select ${obj.length} rows`)
  for (let i = 0; i < obj.length; i++) {
    equals(rows[i], obj[i], `select obj${i}`)
  }

  txn = db.transaction((nums) => {
    for (const num of nums) {
      insert.run([num])
    }
    // rollback
    throw new Error('test')
  })

  try {
    txn(nums)
    console.log('error', 'txn throws')
  } catch (err) {
    console.log('pass', 'txn throws')
    equals(err.message, 'test', 'txn throws msg')
  }

  rows = select.all()
  equals(rows.length, obj.length, `select ${obj.length} rows`)
  for (let i = 0; i < obj.length; i++) {
    equals(rows[i], obj[i], `select obj${i}`)
  }

  db.close()
  equals(1, 1, 'close')
}

const misc = () => {
  console.log('misc')
  const db = open('/app/test.js.db')

  db.exec('drop table if exists misc')
  db.exec('create table misc (id integer, buf blob)')

  const blob = new Uint8Array([1, 2, 3])
  let statement = db.prepare('insert into misc (id, buf) values (?, ?)')
  let info = statement.run([1, blob])
  equals(info.changes, 1n, 'insert 1 row')

  statement = db.prepare('select * from misc')
  let row = statement.one()
  equals(row.id, 1n, 'row id 1')
  equalsBlob(row.buf, blob, 'row buf ok')

  equals(true, statement.release(), 'release true')

  try {
    statement.run()
    console.log('error', 'released statement run throws')
  } catch (err) {
    console.log('pass', 'released statement run throws')
  }

  try {
    statement.one()
    console.log('error', 'released statement one throws')
  } catch (err) {
    console.log('pass', 'released statement one throws')
  }

  try {
    statement.all()
    console.log('error', 'released statement all throws')
  } catch (err) {
    console.log('pass', 'released statement all throws')
  }

  db.close()
  equals(1, 1, 'close')

  try {
    db.exec('drop table if exists misc')
    console.log('error', 'closed db throws')
  } catch (err) {
    console.log('pass', 'closed db throws')
  }

  const db2 = open('file:/app/test.js.db?vfs=unix-dotfile')
  equals(1, 1, 'vfs open')
  db2.close()
  equals(1, 1, 'vfs close')

  try {
    open('file:/app/test.js.db?vfs=notfound')
    console.log('error', 'vfs open throws')
  } catch (err) {
    console.log('pass', 'vfs open throws')
  }
}

export const run = {
  run() {
    try {
      basic()
      strict()
      txn()
      misc()
    } catch (err) {
      console.log('!! error', err)
    }
  }
}
