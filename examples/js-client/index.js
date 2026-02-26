// Build this file as a WASI P2 component with ComponentizeJS.
// The generated component imports `wasm:wasi-sqlite/sqlite` and
// exports `wasi:cli/run@0.2.3#run`.

import {
  open,
  prepare,
  query,
  closeStatement,
  closeDb,
} from "wasm:wasi-sqlite/sqlite";

const printSqliteValue = (value) => {
  if (value === null || value === undefined) {
    console.log("other=null");
    return;
  }

  if (typeof value === "bigint" || typeof value === "number") {
    console.log(`int=${value}`);
    return;
  }

  if (typeof value === "string") {
    console.log(`text=${value}`);
    return;
  }

  if (typeof value === "object") {
    if ("integer" in value) {
      console.log(`int=${value.integer}`);
      return;
    }
    if ("text" in value) {
      console.log(`text=${value.text}`);
      return;
    }
    if ("tag" in value) {
      if (value.tag === "integer") {
        console.log(`int=${value.val}`);
        return;
      }
      if (value.tag === "text") {
        console.log(`text=${value.val}`);
        return;
      }
      console.log(`other=${JSON.stringify(value)}`);
      return;
    }
  }

  console.log(`other=${JSON.stringify(value)}`);
};

export const run = {
  run() {
    const db = open(":memory:");

    const init = prepare(db, "create table demo (id integer, name text)");
    query(init);
    closeStatement(init);

    const insert = prepare(db, "insert into demo values (1, 'hello from rust')");
    query(insert);
    closeStatement(insert);

    const select = prepare(db, "select id, name from demo");
    const rows = query(select);

    for (const row of rows) {
      for (const value of row.values) {
        printSqliteValue(value);
      }
    }

    closeStatement(select);
    closeDb(db);
  },
};
