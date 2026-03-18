use sqlite_wasm_wasi::{open, Value, NO_PARAMS};

fn main() {
    if let Err(err) = example() {
        println!("!! error {:?}", err);
    }
}

fn example() -> Result<(), sqlite_wasm_wasi::Error> {
    let db = open("/app/example.rust.db")?;
    db.exec("drop table if exists example", &NO_PARAMS)?;
    db.exec(
        "create table example (id integer, name text, note text, ratio real, big_int integer)",
        &NO_PARAMS,
    )?;

    let mut insert =
        db.prepare("insert into example (id, name, note, ratio, big_int) values (?, ?, ?, ?, ?)")?;
    let mut info = insert.run(&[
        &1_i32,
        &"hello from js",
        &Value::Null,
        &3.25_f32,
        &9_007_199_254_740_993_i64,
    ])?;
    println!("{} == 1", info.changes);
    println!("{} == 1", info.last_insert_rowid);

    info = insert.run(&[
        &2_i32,
        &"hello from js",
        &Value::Null,
        &3.25_f32,
        &9_007_199_254_740_993_i64,
    ])?;
    println!("{} == 1", info.changes);
    println!("{} == 2", info.last_insert_rowid);

    let mut select =
        db.prepare("select id, name, note, ratio, big_int from example where id = ?")?;
    let row = select.one(&[&1_i32])?;
    println!("{:?}", row);

    select = db.prepare("select * from example where 1 = ? order by id")?;
    let mut rows = select.all(&[&1_i32])?;
    println!("{:?}", rows);

    db.exec("drop table if exists txn", &NO_PARAMS)?;
    db.exec("create table txn (id integer)", &NO_PARAMS)?;
    insert = db.prepare("insert into txn (id) values (?)")?;
    let mut insert_many = db.transaction(|nums: Vec<i64>| {
        for num in nums {
            insert.run(&[&num])?;
        }
        Ok(())
    });

    let nums = vec![1, 4, 5, 6];
    insert_many(nums)?;

    select = db.prepare("select * from txn order by id")?;
    rows = select.all(&NO_PARAMS)?;
    println!("{:?}", rows);
    db.close()?;

    Ok(())
}
