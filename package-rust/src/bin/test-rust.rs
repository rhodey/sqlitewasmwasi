use sqlite_wasm_wasi::{open, row_value, Error, Row, Value, NO_PARAMS};

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Integer(v) => format!("{v}n"),
        Value::Real(v) => {
            if v.fract() == 0.0 {
                format!("{v:.1}r")
            } else {
                let v = v.to_string();
                format!("{v}r")
            }
        }
        Value::Text(v) => format!("\"{v}\""),
        Value::Blob(v) => format!("{:?}", v),
    }
}

fn row_to_string(row: &Row) -> String {
    let mut parts = Vec::new();
    for (key, value) in row {
        parts.push(format!("\"{key}\":{}", value_to_string(value)));
    }
    format!("{{{}}}", parts.join(","))
}

fn equals(actual: String, expected: String, msg: &str) {
    if actual == expected {
        println!("pass {msg}");
    } else {
        println!("error {msg}");
        println!("actual > {actual}");
        println!("expected {expected}");
    }
}

fn equals_blob(actual: &[u8], expected: &[u8], msg: &str) {
    let ok = actual == expected;
    if ok {
        println!("pass {msg}");
    } else {
        println!("error {msg}");
        println!("actual > {:?}", actual);
        println!("expected {:?}", expected);
    }
}

fn basic() -> Result<(), Error> {
    println!("basic");
    let db = open("/app/test.rust.db")?;
    db.exec("drop table if exists basic", &NO_PARAMS)?;

    let mut num = db.exec(
        "create table basic (id integer, name text, note text, ratio real, big_int integer)",
        &NO_PARAMS,
    )?;
    equals(
        format!("{}n", num),
        "0n".to_string(),
        "create table no rows",
    );

    let mut statement =
        db.prepare("insert into basic (id, name, note, ratio, big_int) values (?, ?, ?, ?, ?)")?;
    let mut info = statement.run::<Value>(&[
        1_i64.into(),
        "hello from js".into(),
        Value::Null,
        3.25_f64.into(),
        9_007_199_254_740_993_i64.into(),
    ])?;
    equals(
        format!("{}n", info.changes),
        "1n".to_string(),
        "insert 1 row",
    );
    equals(
        format!("{}n", info.last_insert_rowid),
        "1n".to_string(),
        "row id 1",
    );
    equals(
        statement.release()?.to_string(),
        "true".to_string(),
        "release true",
    );
    equals(
        statement.release()?.to_string(),
        "false".to_string(),
        "release false",
    );

    statement =
        db.prepare("insert into basic (id, name, note, ratio, big_int) values (?, ?, ?, ?, ?)")?;
    info = statement.run::<Value>(&[
        2_i64.into(),
        "hello from js".into(),
        Value::Null,
        3.25.into(),
        9_007_199_254_740_993.into(),
    ])?;
    equals(
        format!("{}n", info.changes),
        "1n".to_string(),
        "insert 1 row",
    );
    equals(
        format!("{}n", info.last_insert_rowid),
        "2n".to_string(),
        "row id 2",
    );
    equals(
        statement.release()?.to_string(),
        "true".to_string(),
        "release true",
    );
    equals(
        statement.release()?.to_string(),
        "false".to_string(),
        "release false",
    );

    let obj1 = row_from_values(1, "hello from js", 3.25, 9_007_199_254_740_993);
    let obj2 = row_from_values(2, "hello from js", 3.25, 9_007_199_254_740_993);

    statement = db.prepare("select id, name, note, ratio, big_int from basic where id = 1")?;
    let mut row = statement.one(&NO_PARAMS)?.unwrap();
    equals(row_to_string(&row), row_to_string(&obj1), "select 1 row A");

    statement = db.prepare("select id, name, note, ratio, big_int from basic where id = ?")?;
    row = statement.one(&[1_i64])?.unwrap();
    equals(row_to_string(&row), row_to_string(&obj1), "select 1 row B");

    statement = db.prepare("select id, name, note, ratio, big_int from basic where id = ?")?;
    row = statement.one(&[2_i64])?.unwrap();
    equals(row_to_string(&row), row_to_string(&obj2), "select 1 row C");

    statement = db.prepare("select id, name, note, ratio, big_int from basic where id = 3")?;
    let row_or_null = statement.one(&NO_PARAMS)?;
    equals(
        row_or_null
            .map(|r| row_to_string(&r))
            .unwrap_or_else(|| "null".to_string()),
        "null".to_string(),
        "select 1 row NULL",
    );

    statement = db.prepare("select id, name, note, ratio, big_int from basic order by id")?;
    let mut rows = statement.all(&NO_PARAMS)?;
    equals(rows.len().to_string(), "2".to_string(), "select 2 rows");
    equals(
        row_to_string(&rows[0]),
        row_to_string(&obj1),
        "select row id 1",
    );
    equals(
        row_to_string(&rows[1]),
        row_to_string(&obj2),
        "select row id 2",
    );

    statement = db.prepare("select id, name, note, ratio, big_int from basic where id = ?")?;
    rows = statement.all(&[1_i64])?;
    equals(rows.len().to_string(), "1".to_string(), "select 1 rows");
    equals(
        row_to_string(&rows[0]),
        row_to_string(&obj1),
        "select row id 1",
    );

    statement = db.prepare("select id, name, note, ratio, big_int from basic where id = ?")?;
    rows = statement.all(&[3_i64])?;
    equals(rows.len().to_string(), "0".to_string(), "select 0 rows");

    num = db.exec("update basic set id = 3 where id = ?", &[1_i64])?;
    equals(format!("{}n", num), "1n".to_string(), "update 1 rows");
    num = db.exec("update basic set id = 3 where id = ?", &[1_i64])?;
    equals(format!("{}n", num), "0n".to_string(), "update 0 rows");
    num = db.exec("delete from basic where 1 = ?", &[1_i64])?;
    equals(format!("{}n", num), "2n".to_string(), "delete 2 rows");

    statement = db.prepare("select 3 where 1 = 1")?;
    row = statement.one(&NO_PARAMS)?.unwrap();
    let mut expected = Row::new();
    expected.insert("3".to_string(), 3_i64.into());
    equals(
        row_to_string(&row),
        row_to_string(&expected),
        "select 3 col name",
    );

    db.close()?;
    equals("1".to_string(), "1".to_string(), "close");
    Ok(())
}

fn strict() -> Result<(), Error> {
    println!("strict");
    let db = open("/app/test.rust.db")?;
    db.exec("drop table if exists nums", &NO_PARAMS)?;
    db.exec(
        "create table nums (id integer, ratio real) strict",
        &NO_PARAMS,
    )?;
    let mut statement = db.prepare("insert into nums (id, ratio) values (?, ?)")?;
    let mut info = statement.run::<Value>(&[1_i64.into(), 3.25_f64.into()])?;
    equals(
        format!("{}n", info.changes),
        "1n".to_string(),
        "insert 1 real",
    );
    info = statement.run::<Value>(&[2_i64.into(), 2_i64.into()])?;
    equals(
        format!("{}n", info.changes),
        "1n".to_string(),
        "insert 1 int as real",
    );

    match statement.run::<Value>(&[4_i64.into(), "abc".into()]) {
        Ok(_) => println!("error insert text as real throws"),
        Err(_) => println!("pass insert text as real throws"),
    }

    statement = db.prepare("select * from nums order by id")?;
    let mut rows = statement.all(&NO_PARAMS)?;
    equals(rows.len().to_string(), "2".to_string(), "select 3 rows");
    equals(
        row_to_string(&rows[0]),
        row_to_string(&row_num(1, 3.25)),
        "select row id 1",
    );
    equals(
        row_to_string(&rows[1]),
        row_to_string(&row_num(2, 2.0)),
        "select row id 2",
    );

    db.exec("drop table if exists nums", &NO_PARAMS)?;
    db.exec("create table nums (id integer, ratio real)", &NO_PARAMS)?;
    statement = db.prepare("insert into nums (id, ratio) values (?, ?)")?;
    info = statement.run::<Value>(&[1_i64.into(), "abc".into()])?;
    equals(
        format!("{}n", info.changes),
        "1n".to_string(),
        "insert 1 text",
    );

    statement = db.prepare("select * from nums order by id")?;
    rows = statement.all(&NO_PARAMS)?;
    equals(rows.len().to_string(), "1".to_string(), "select 1 rows");
    equals(
        row_to_string(&rows[0]),
        row_to_string(&row_num(1, "abc")),
        "select row id 1",
    );

    db.close()?;
    equals("1".to_string(), "1".to_string(), "close");
    Ok(())
}

fn txn() -> Result<(), Error> {
    println!("txn");
    let db = open("/app/test.rust.db")?;
    db.exec("drop table if exists txn", &NO_PARAMS)?;

    db.exec("create table txn (id integer)", &NO_PARAMS)?;
    let insert = db.prepare("insert into txn (id) values (?)")?;
    let mut info = insert.run(&[1_i64])?;
    equals(
        format!("{}n", info.changes),
        "1n".to_string(),
        "insert 1 row",
    );
    equals(
        format!("{}n", info.last_insert_rowid),
        "1n".to_string(),
        "row id 1",
    );

    let objs = vec![1, 4, 5, 6]
        .into_iter()
        .map(|n| row_id(n))
        .collect::<Vec<_>>();

    let select = db.prepare("select * from txn order by id")?;
    let mut rows = select.all(&NO_PARAMS)?;
    equals(rows.len().to_string(), "1".to_string(), "select 1 rows");
    equals(
        row_to_string(&rows[0]),
        row_to_string(&objs[0]),
        "select obj0",
    );

    let mut txn = db.transaction(|nums: Vec<i64>| {
        for num in nums {
            info = insert.run(&[num])?;
            equals(
                format!("{}n", info.changes),
                "1n".to_string(),
                "insert 1 row",
            );
        }
        Ok(())
    });

    let nums = objs
        .iter()
        .skip(1)
        .map(|obj| match row_value(obj, "id") {
            Some(Value::Integer(v)) => *v,
            _ => 0,
        })
        .collect::<Vec<_>>();
    txn(nums.clone())?;

    rows = select.all(&NO_PARAMS)?;
    equals(
        rows.len().to_string(),
        objs.len().to_string(),
        &format!("select {} rows", objs.len()),
    );
    for i in 0..objs.len() {
        equals(
            row_to_string(&rows[i]),
            row_to_string(&objs[i]),
            &format!("select obj{i}"),
        );
    }

    let mut txn = db.transaction(|nums: Vec<i64>| -> Result<(), Error> {
        for num in nums {
            insert.run(&[num])?;
        }
        Err(Error {
            code: -1,
            message: "test".to_string(),
        })
    });

    match txn(nums) {
        Ok(_) => println!("error txn throws"),
        Err(err) => {
            println!("pass txn throws");
            equals(err.message, "test".to_string(), "txn throws msg");
        }
    }

    rows = select.all(&NO_PARAMS)?;
    equals(
        rows.len().to_string(),
        objs.len().to_string(),
        &format!("select {} rows", objs.len()),
    );
    for i in 0..objs.len() {
        equals(
            row_to_string(&rows[i]),
            row_to_string(&objs[i]),
            &format!("select obj{i}"),
        );
    }

    db.close()?;
    equals("1".to_string(), "1".to_string(), "close");
    Ok(())
}

fn misc() -> Result<(), Error> {
    println!("misc");
    let db = open("/app/test.rust.db")?;

    db.exec("drop table if exists misc", &NO_PARAMS)?;
    db.exec("create table misc (id integer, buf blob)", &NO_PARAMS)?;

    let blob = vec![1, 2, 3];
    let mut statement = db.prepare("insert into misc (id, buf) values (?, ?)")?;
    let info = statement.run::<Value>(&[1_i64.into(), blob.clone().into()])?;
    equals(
        format!("{}n", info.changes),
        "1n".to_string(),
        "insert 1 row",
    );

    statement = db.prepare("select * from misc")?;
    let row = statement.one(&NO_PARAMS)?.unwrap();
    equals(
        match row_value(&row, "id") {
            Some(Value::Integer(v)) => format!("{v}n"),
            _ => "null".to_string(),
        },
        "1n".to_string(),
        "row id 1",
    );
    if let Some(Value::Blob(v)) = row_value(&row, "buf") {
        equals_blob(v, &blob, "row buf ok");
    } else {
        println!("error row buf ok");
    }

    equals(
        statement.release()?.to_string(),
        "true".to_string(),
        "release true",
    );

    match statement.run(&NO_PARAMS) {
        Ok(_) => println!("error released statement run throws"),
        Err(_) => println!("pass released statement run throws"),
    }

    match statement.one(&NO_PARAMS) {
        Ok(_) => println!("error released statement one throws"),
        Err(_) => println!("pass released statement one throws"),
    }

    match statement.all(&NO_PARAMS) {
        Ok(_) => println!("error released statement all throws"),
        Err(_) => println!("pass released statement all throws"),
    }

    db.close()?;
    equals("1".to_string(), "1".to_string(), "close");

    match db.exec("drop table if exists misc", &NO_PARAMS) {
        Ok(_) => println!("error closed db throws"),
        Err(_) => println!("pass closed db throws"),
    }

    let db2 = open("file:/app/test.rust.db?vfs=unix-dotfile")?;
    equals("1".to_string(), "1".to_string(), "vfs open");
    db2.close()?;
    equals("1".to_string(), "1".to_string(), "vfs close");

    match open("file:/app/test.rust.db?vfs=notfound") {
        Ok(_) => println!("error vfs open throws"),
        Err(_) => println!("pass vfs open throws"),
    }

    Ok(())
}

fn row_from_values(id: i64, name: &str, ratio: f64, big_int: i64) -> Row {
    let mut row = Row::new();
    row.insert("id".to_string(), id.into());
    row.insert("name".to_string(), name.into());
    row.insert("note".to_string(), Value::Null);
    row.insert("ratio".to_string(), ratio.into());
    row.insert("big_int".to_string(), big_int.into());
    row
}

fn row_num(id: i64, ratio: impl Into<Value>) -> Row {
    let mut row = Row::new();
    row.insert("id".to_string(), id.into());
    row.insert("ratio".to_string(), ratio.into());
    row
}

fn row_id(id: i64) -> Row {
    let mut row = Row::new();
    row.insert("id".to_string(), id.into());
    row
}

fn main() {
    if let Err(err) = basic()
        .and_then(|_| strict())
        .and_then(|_| txn())
        .and_then(|_| misc())
    {
        println!("!! error {:?}", err);
    }
}
