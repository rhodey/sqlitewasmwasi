mod bindings {
    wit_bindgen::generate!({
        path: "../../wit",
        world: "sqlite-client",
    });
}

use bindings::wasm::wasi_sqlite::sqlite::{
    close_db, close_statement, open, prepare, query, SqliteValue,
};

fn main() {
    let db = open(":memory:").expect("open db");

    let init = prepare(db, "create table demo (id integer, name text)").expect("prepare init");
    query(init).expect("run init");
    close_statement(init).expect("close init");

    let insert =
        prepare(db, "insert into demo values (1, 'hello from rust')").expect("prepare insert");
    query(insert).expect("run insert");
    close_statement(insert).expect("close insert");

    let select = prepare(db, "select id, name from demo").expect("prepare select");
    let rows = query(select).expect("query rows");

    for row in rows {
        for value in row.values {
            match value {
                SqliteValue::Integer(v) => println!("int={v}"),
                SqliteValue::Text(v) => println!("text={v}"),
                other => println!("other={other:?}"),
            }
        }
    }

    close_statement(select).expect("close select");
    close_db(db).expect("close db");
}
