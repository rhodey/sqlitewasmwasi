mod bindings {
    wit_bindgen::generate!({
        path: "../../wit",
        world: "sqlite-client",
    });
}

use bindings::wasm::wasi_sqlite::sqlite::{close, exec, open, prepare, SqliteValue};

fn main() {
    let db = open("file:/workspace/rust-client.db?vfs=unix-dotfile").expect("open db");

    exec(db, "drop table if exists demo", None).expect("drop old table");

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
    {
        let insert = prepare(
            db,
            "insert into demo (id, name, note, ratio, big_id) values (?, ?, ?, ?, ?)",
        )
        .expect("prepare insert");
        let info = insert.run(Some(&insert_params)).expect("run insert");
        println!(
            "changes={} last_insert_rowid={}",
            info.changes, info.last_insert_rowid
        );
        assert!(
            insert.release(),
            "release insert should be first release call"
        );
    }

    {
        let select =
            prepare(db, "select id, name, note, ratio, big_id from demo").expect("prepare select");
        let rows = select.all(None).expect("query rows");

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
        assert!(
            select.release(),
            "release select should be first release call"
        );
    }

    {
        let select_one = prepare(
            db,
            "select id, name, note, ratio, big_id from demo where id = ?",
        )
        .expect("prepare select one");
        let row = select_one
            .one(Some(&[SqliteValue::Integer(1)]))
            .expect("query one row")
            .expect("expected one() to return a row");
        assert_eq!(row.values.len(), 5, "expected one() to return 5 columns");
        println!("one() got single row back");
        assert!(
            select_one.release(),
            "release select one should be first release call"
        );
    }

    close(db).expect("close db");
}
