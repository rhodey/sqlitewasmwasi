// Build this file as a WASI P2 component with ComponentizeJS.
// The generated component imports `wasm:wasi/sqlite:component/sqlite`.

import { open, prepare, query, closeStatement, closeDb } from "wasm:wasi/sqlite:component/sqlite";

export function run() {
  const db = open(":memory:");

  const init = prepare(db, "create table demo (id integer, name text)");
  query(init);
  closeStatement(init);

  const insert = prepare(db, "insert into demo values (1, 'hello from js')");
  query(insert);
  closeStatement(insert);

  const select = prepare(db, "select id, name from demo");
  const rows = query(select);

  for (const row of rows) {
    console.log(JSON.stringify(row.values));
  }

  closeStatement(select);
  closeDb(db);
}
