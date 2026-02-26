mod bindings {
    wit_bindgen::generate!({
        path: "../../wit",
        world: "sqlite-client",
    });
}

use bindings::wasm::wasi_sqlite::sqlite::{
    all, close, exec, one, open, prepare, release, run, SqliteValue,
};

fn main() {
    let db = open(":memory:").expect("open db");

    exec(
        db,
        "create table demo (id integer, name text, note text, ratio real, big_id integer)",
        None,
    )
    .expect("exec init");

    let insert_params = vec![
        SqliteValue::Integer(1),
        SqliteValue::Text("hello from rust".to_string()),
        SqliteValue::Null,
        SqliteValue::Real(3.25),
        SqliteValue::Integer(9_007_199_254_740_993),
    ];
    let insert = prepare(
        db,
        "insert into demo (id, name, note, ratio, big_id) values (?, ?, ?, ?, ?)",
    )
    .expect("prepare insert");
    let info = run(insert, Some(&insert_params)).expect("run insert");
    println!(
        "changes={} last_insert_rowid={}",
        info.changes, info.last_insert_rowid
    );
    release(insert).expect("release insert");

    let select =
        prepare(db, "select id, name, note, ratio, big_id from demo").expect("prepare select");
    let rows = all(select, None).expect("query rows");

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

    let select_one = prepare(
        db,
        "select id, name, note, ratio, big_id from demo where id = ?",
    )
    .expect("prepare select one");
    let row = one(select_one, Some(&[SqliteValue::Integer(1)]))
        .expect("query one row")
        .expect("expected one() to return a row");
    assert_eq!(row.values.len(), 5, "expected one() to return 5 columns");
    println!("one() got single row back");
    release(select_one).expect("release select one");

    close(db).expect("close db");
}
