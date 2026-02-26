import { open as openDb, exec, prepare, close } from 'wasm:sqlite-wasi/sqlite'

const valuesToParams = (arr) => {
  return arr.map((val, idx) => {
    if (typeof val === 'bigint') {
      return { tag: 'integer', val }
    } else if (typeof val === 'number' && Number.isInteger(val)) {
      return { tag: 'integer', val }
    } else if (typeof val === 'number') {
      return { tag: 'real', val }
    } else if (typeof val === 'string') {
      return { tag: 'text', val }
    } else if (val === null) {
      return { tag: 'null' }
    } else {
      throw new Error(`sqlite param ${idx} unsupported type`)
    }
  })
}

const rowsToObjects = (rows) => {
  const result = []
  for (const row of rows) {
    const obj = {}
    for (let c = 0; c < row.values.length; c++) {
      const name = row.columns[c]
      obj[name] = row.values[c].val ?? null
    }
    result.push(obj)
  }
  return result
}

class Statement {
  constructor(stmt) {
    this.stmt = stmt
  }

  run(params=[]) {
    params = valuesToParams(params)
    const info = this.stmt.run(params)
    return info
  }

  one(params=[]) {
    params = valuesToParams(params)
    const row = this.stmt.one(params)
    if (row === null || row === undefined) { return null }
    return rowsToObjects([row])[0]
  }

  all(params=[]) {
    params = valuesToParams(params)
    const rows = this.stmt.all(params)
    return rowsToObjects(rows)
  }

  release() {
    return this.stmt.release()
  }
}

class Database {
  constructor(handle) {
    this.handle = handle
  }

  exec(sql, params=[]) {
    params = valuesToParams(params)
    return exec(this.handle, sql, params)
  }

  prepare(sql) {
    const stmt = prepare(this.handle, sql)
    return new Statement(stmt)
  }

  close() {
    return close(this.handle)
  }
}

export function open(uri) {
  uri = uri.startsWith('file:') ? uri : `file:${uri}`
  return new Database(openDb(uri))
}
