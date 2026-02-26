mod bindings {
    wit_bindgen::generate!({
        path: "../../wit",
        world: "sqlite-client",
    });
}

use bindings::wasm::wasi_sqlite::sqlite::{close, open, prepare, query, release, SqliteValue};

fn main() {
    let db = open(":memory:").expect("open db");

    let init = prepare(
        db,
        "create table demo (id integer, name text, note text, ratio real, big_id integer)",
    )
    .expect("prepare init");
    query(init).expect("run init");
    release(init).expect("release init");

    let insert = prepare(
        db,
        "insert into demo (id, name, note, ratio, big_id) values (1, 'hello from rust', NULL, 3.25, 9007199254740993)",
    )
    .expect("prepare insert");
    query(insert).expect("run insert");
    release(insert).expect("release insert");

    let select =
        prepare(db, "select id, name, note, ratio, big_id from demo").expect("prepare select");
    let rows = query(select).expect("query rows");

    for row in rows {
        for value in row.values {
            match value {
                SqliteValue::Null => println!("null"),
                SqliteValue::Integer(v) => println!("int={v}"),
                SqliteValue::Text(v) => println!("text={v}"),
                SqliteValue::Real(v) => println!("real={v}"),
                other => println!("other={other:?}"),
            }
        }
    }

    release(select).expect("release select");
    close(db).expect("close db");
}
