import { open as openDb, exec, prepare, close } from 'wasm:sqlite-wasi/sqlite'

// todo: blob
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

const unwrapErr = (fn) => {
  try {
    return fn()
  } catch (err) {
    if (err.payload) {
      const errr = new Error(err.payload.message)
      errr.code = err.payload.code
      throw errr
    }
    throw err
  }
}

class Statement {
  constructor(stmt) {
    this.stmt = stmt
  }

  run(params=[]) {
    params = valuesToParams(params)
    const fn = () => this.stmt.run(params)
    return unwrapErr(fn)
  }

  one(params=[]) {
    params = valuesToParams(params)
    const fn = () => {
      const row = this.stmt.one(params)
      if (row === null || row === undefined) { return null }
      return rowsToObjects([row])[0]
    }
    return unwrapErr(fn)
  }

  all(params=[]) {
    params = valuesToParams(params)
    const fn = () => {
      const rows = this.stmt.all(params)
      return rowsToObjects(rows)
    }
    return unwrapErr(fn)
  }

  release() {
    const fn = () => this.stmt.release()
    return unwrapErr(fn)
  }
}

class Database {
  constructor(handle) {
    this.handle = handle
  }

  exec(sql, params=[]) {
    params = valuesToParams(params)
    const fn = () => exec(this.handle, sql, params)
    return unwrapErr(fn)
  }

  prepare(sql) {
    const stmt = prepare(this.handle, sql)
    return new Statement(stmt)
  }

  close() {
    const fn = () => close(this.handle)
    return unwrapErr(fn)
  }
}

export function open(uri) {
  uri = uri.startsWith('file:') ? uri : `file:${uri}`
  return new Database(openDb(uri))
}
