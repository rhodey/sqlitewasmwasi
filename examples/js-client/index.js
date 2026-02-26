// Build this file as a WASI P2 component with ComponentizeJS.
// The generated component imports `wasm:wasi-sqlite/sqlite` and
// exports `wasi:cli/run@0.2.3#run`.

import {
  open,
  prepare,
  exec,
  all,
  one,
  run as runStatement,
  release,
  close,
} from "wasm:wasi-sqlite/sqlite";

const printSqliteValue = (value) => {
  if (value === null) {
    console.log("null");
    return;
  }

  // ComponentizeJS variants are encoded as { tag, val }.
  const { tag, val } = value;

  switch (tag) {
    case "null":
      console.log("null");
      break;
    case "integer":
      console.log(`int=${val}`);
      break;
    case "real":
      console.log(`real=${val}`);
      break;
    case "text":
      console.log(`text=${val}`);
      break;
    case "blob":
      console.log(`blob=${val.length} bytes`);
      break;
    default:
      throw new Error(`unknown sqlite-value tag: ${tag}`);
  }
};

export const run = {
  run() {
    const db = open("file:/workspace/js-client.db?vfs=unix-dotfile");

    exec(db, "drop table if exists demo", undefined);

    exec(db, "create table demo (id integer, name text, note text, ratio real, big_id integer)", undefined);

    const insert = prepare(
      db,
      "insert into demo (id, name, note, ratio, big_id) values (?, ?, ?, ?, ?)",
    );
    const info = runStatement(insert, [
        { tag: "integer", val: 1 },
        { tag: "text", val: "hello from rust" },
        { tag: "null" },
        { tag: "real", val: 3.25 },
        { tag: "integer", val: 9007199254740993n },
      ]);
    console.log(`changes=${info.changes} last_insert_rowid=${info.lastInsertRowid}`);
    release(insert);

    const select = prepare(db, "select id, name, note, ratio, big_id from demo");
    const rows = all(select, undefined);

    for (const row of rows) {
      for (const value of row.values) {
        printSqliteValue(value);
      }
    }

    release(select);

    const selectOne = prepare(db, "select id, name, note, ratio, big_id from demo where id = ?");
    const singleRow = one(selectOne, [
      { tag: "integer", val: 1 },
    ]);
    if (singleRow === undefined) {
      throw new Error("expected one() to return a row");
    }
    if (singleRow.values.length !== 5) {
      throw new Error(`expected one() to return 5 columns, got ${singleRow.values.length}`);
    }
    console.log("one() got single row back");
    release(selectOne);

    close(db);
  },
};
