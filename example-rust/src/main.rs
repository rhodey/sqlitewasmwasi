use sqlite_wasm_wasi::{open, Value};

fn main() {
    if let Err(err) = example() {
        println!("!! error {:?}", err);
    }
}

fn example() -> Result<(), sqlite_wasm_wasi::Error> {
    let db = open("/app/example.rust.db")?;
    db.exec("drop table if exists example", &[])?;
    db.exec(
        "create table example (id integer, name text, note text, ratio real, big_int integer)",
        &[],
    )?;

    let mut insert =
        db.prepare("insert into example (id, name, note, ratio, big_int) values (?, ?, ?, ?, ?)")?;
    let mut info = insert.run(&[
        Value::Integer(1),
        Value::Text("hello from js".to_string()),
        Value::Null,
        Value::Real(3.25),
        Value::Integer(9_007_199_254_740_993),
    ])?;
    println!("{} == 1", info.changes);
    println!("{} == 1", info.last_insert_rowid);

    info = insert.run(&[
        Value::Integer(2),
        Value::Text("hello from js".to_string()),
        Value::Null,
        Value::Real(3.25),
        Value::Integer(9_007_199_254_740_993),
    ])?;
    println!("{} == 1", info.changes);
    println!("{} == 2", info.last_insert_rowid);

    let mut select =
        db.prepare("select id, name, note, ratio, big_int from example where id = ?")?;
    let row = select.one(&[Value::Integer(1)])?;
    println!("{:?}", row);

    select = db.prepare("select * from example where 1 = ? order by id")?;
    let mut rows = select.all(&[Value::Integer(1)])?;
    println!("{:?}", rows);

    db.exec("drop table if exists txn", &[])?;
    db.exec("create table txn (id integer)", &[])?;
    insert = db.prepare("insert into txn (id) values (?)")?;
    let mut insert_many = db.transaction(|nums: Vec<i64>| {
        for num in nums {
            insert.run(&[Value::Integer(num)])?;
        }
        Ok(())
    });

    let nums = vec![1, 4, 5, 6];
    insert_many(nums)?;

    select = db.prepare("select * from txn order by id")?;
    rows = select.all(&[])?;
    println!("{:?}", rows);
    db.close()?;

    Ok(())
}
