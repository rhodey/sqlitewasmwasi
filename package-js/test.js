import { open } from './index.js'

export const run = {
  run() {
    const db = open('file:/app/test.js.db?vfs=unix-dotfile')
    db.exec('drop table if exists demo')
    db.exec('create table demo (id integer, name text, note text, ratio real, big_id integer)')

    let statement = db.prepare('insert into demo (id, name, note, ratio, big_id) values (?, ?, ?, ?, ?)')
    let info = statement.run([1, 'hello from rust', null, 3.25, 9007199254740993n])
    console.log(123, info)
    console.log(123, statement.release())

    statement = db.prepare('insert into demo (id, name, note, ratio, big_id) values (?, ?, ?, ?, ?)')
    info = statement.run([2, 'hello from rust', null, 3.25, 9007199254740993n])
    console.log(123, info)
    console.log(123, statement.release())

    statement = db.prepare('select id, name, note, ratio, big_id from demo where id = 1')
    let row = statement.one()
    console.log(456, row)

    statement = db.prepare('select id, name, note, ratio, big_id from demo where id = ?')
    row = statement.one([1])
    console.log(456, row)

    statement = db.prepare('select id, name, note, ratio, big_id from demo where id = 3')
    row = statement.one()
    console.log(456, row)

    statement = db.prepare('select id, name, note, ratio, big_id from demo')
    let rows = statement.all()
    for (const row of rows) {
      console.log(789, row)
    }

    statement = db.prepare('select id, name, note, ratio, big_id from demo where 1 = ?')
    rows = statement.all([1])
    for (const row of rows) {
      console.log(789, row)
    }

    statement = db.prepare('select id, name, note, ratio, big_id from demo where 1 = ?')
    rows = statement.all([3])
    for (const row of rows) {
      console.log(789, row)
    }

    db.close()
  },
}
