import { open, prepare, exec, close } from "wasm:sqlite-wasi/sqlite";

const printSqliteValue = (value, label) => {
  if (value === null) {
    console.log(`${label}=null`);
    return;
  }

  // ComponentizeJS variants are encoded as { tag, val }.
  const { tag, val } = value;

  switch (tag) {
    case "null":
      console.log(`${label}=null`);
      break;
    case "integer":
      console.log(`${label}=int=${val}`);
      break;
    case "real":
      console.log(`${label}=real=${val}`);
      break;
    case "text":
      console.log(`${label}=text=${val}`);
      break;
    case "blob":
      console.log(`${label}=blob=${val.length} bytes`);
      break;
    default:
      throw new Error(`unknown sqlite-value tag: ${tag}`);
  }
};

const valueByName = (row, name) => {
  const index = row.columns.indexOf(name);
  if (index === -1) {
    return undefined;
  }
  return row.values[index];
};

export const run = {
  run() {
    const db = open("file:/workspace/js-client.db?vfs=unix-dotfile");

    exec(db, "drop table if exists demo", undefined);

    exec(db, "create table demo (id integer, name text, note text, ratio real, big_id integer)", undefined);

    {
      const insert = prepare(
        db,
        "insert into demo (id, name, note, ratio, big_id) values (?, ?, ?, ?, ?)",
      );
      const info = insert.run([
        { tag: "integer", val: 1 },
        { tag: "text", val: "hello from rust" },
        { tag: "null" },
        { tag: "real", val: 3.25 },
        { tag: "integer", val: 9007199254740993n },
      ]);
      console.log(`changes=${info.changes} last_insert_rowid=${info.lastInsertRowid}`);
      if (!insert.release()) {
        throw new Error("expected insert.release() to return true on first call");
      }
    }

    {
      const select = prepare(db, "select id, name, note, ratio, big_id from demo");
      const rows = select.all(undefined);

      for (const row of rows) {
        for (const column of ["id", "name", "note", "ratio", "big_id"]) {
          const value = valueByName(row, column);
          if (value === undefined) {
            throw new Error(`expected ${column} to exist in row`);
          }
          printSqliteValue(value, column);
        }
      }
      if (!select.release()) {
        throw new Error("expected select.release() to return true on first call");
      }
    }

    {
      const selectOne = prepare(db, "select id, name, note, ratio, big_id from demo where id = ?");
      const singleRow = selectOne.one([
        { tag: "integer", val: 1 },
      ]);
      if (singleRow === undefined) {
        throw new Error("expected one() to return a row");
      }
      if (singleRow.columns.length !== 5) {
        throw new Error(`expected one() to return 5 columns, got ${singleRow.columns.length}`);
      }
      const name = valueByName(singleRow, "name");
      if (name?.tag !== "text" || name.val !== "hello from rust") {
        throw new Error("expected to look up one() value by column name");
      }
      console.log("one() got single row back");
      if (!selectOne.release()) {
        throw new Error("expected selectOne.release() to return true on first call");
      }
    }

    close(db);
  },
};
