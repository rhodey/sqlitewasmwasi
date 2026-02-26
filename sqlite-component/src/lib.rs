use std::cell::RefCell;
use std::collections::HashMap;

use rusqlite::types::ValueRef;
use rusqlite::Connection;

wit_bindgen::generate!({
    path: "../wit",
    world: "sqlite-component",
});

use exports::wasm::wasi_sqlite::sqlite::{
    Guest, SqliteError, SqliteRow, SqliteRunInfo, SqliteValue,
};

#[derive(Clone, Debug)]
struct PreparedStatement {
    db: u32,
    sql: String,
}

#[derive(Default)]
struct Manager {
    next_id: u32,
    dbs: HashMap<u32, Connection>,
    statements: HashMap<u32, PreparedStatement>,
}

impl Manager {
    fn alloc_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    fn open(&mut self, path: &str) -> Result<u32, SqliteError> {
        let conn = Connection::open(path).map_err(map_error)?;
        let id = self.alloc_id();
        self.dbs.insert(id, conn);
        Ok(id)
    }

    fn prepare(&mut self, db: u32, sql: &str) -> Result<u32, SqliteError> {
        {
            let conn = self.dbs.get(&db).ok_or_else(|| invalid_handle("db", db))?;
            conn.prepare(sql).map_err(map_error)?;
        }

        let id = self.alloc_id();
        self.statements.insert(
            id,
            PreparedStatement {
                db,
                sql: sql.to_string(),
            },
        );
        Ok(id)
    }

    fn exec(&self, db: u32, sql: &str) -> Result<(), SqliteError> {
        let conn = self.dbs.get(&db).ok_or_else(|| invalid_handle("db", db))?;
        conn.execute_batch(sql).map_err(map_error)
    }

    fn run(&mut self, statement: u32) -> Result<SqliteRunInfo, SqliteError> {
        let prepared = self
            .statements
            .get(&statement)
            .cloned()
            .ok_or_else(|| invalid_handle("statement", statement))?;

        let conn = self
            .dbs
            .get_mut(&prepared.db)
            .ok_or_else(|| invalid_handle("db", prepared.db))?;
        let changes = conn.execute(&prepared.sql, []).map_err(map_error)?;
        Ok(SqliteRunInfo {
            changes: changes as u64,
            last_insert_rowid: conn.last_insert_rowid(),
        })
    }

    fn one(&self, statement: u32) -> Result<Option<SqliteRow>, SqliteError> {
        Ok(self.all(statement)?.into_iter().next())
    }

    fn all(&self, statement: u32) -> Result<Vec<SqliteRow>, SqliteError> {
        let prepared = self
            .statements
            .get(&statement)
            .cloned()
            .ok_or_else(|| invalid_handle("statement", statement))?;

        let conn = self
            .dbs
            .get(&prepared.db)
            .ok_or_else(|| invalid_handle("db", prepared.db))?;
        let mut stmt = conn.prepare(&prepared.sql).map_err(map_error)?;
        let col_count = stmt.column_count();
        let mut rows = stmt.query([]).map_err(map_error)?;

        let mut out = Vec::new();
        while let Some(row) = rows.next().map_err(map_error)? {
            let mut values = Vec::with_capacity(col_count);
            for index in 0..col_count {
                let value = row.get_ref(index).map_err(map_error)?;
                values.push(value_from_ref(value));
            }
            out.push(SqliteRow { values });
        }
        Ok(out)
    }

    fn close_db(&mut self, db: u32) -> Result<(), SqliteError> {
        self.dbs
            .remove(&db)
            .ok_or_else(|| invalid_handle("db", db))?;
        self.statements.retain(|_, statement| statement.db != db);
        Ok(())
    }

    fn release_statement(&mut self, statement: u32) -> Result<(), SqliteError> {
        self.statements
            .remove(&statement)
            .ok_or_else(|| invalid_handle("statement", statement))?;
        Ok(())
    }
}

thread_local! {
    static MANAGER: RefCell<Manager> = RefCell::new(Manager::default());
}

fn with_manager<T>(f: impl FnOnce(&mut Manager) -> T) -> T {
    MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        f(&mut manager)
    })
}

fn map_error(err: rusqlite::Error) -> SqliteError {
    SqliteError {
        code: -1,
        message: err.to_string(),
    }
}

fn invalid_handle(kind: &str, id: u32) -> SqliteError {
    SqliteError {
        code: -2,
        message: format!("unknown {kind} handle: {id}"),
    }
}

fn value_from_ref(value: ValueRef<'_>) -> SqliteValue {
    match value {
        ValueRef::Null => SqliteValue::Null,
        ValueRef::Integer(v) => SqliteValue::Integer(v),
        ValueRef::Real(v) => SqliteValue::Real(v),
        ValueRef::Text(v) => SqliteValue::Text(String::from_utf8_lossy(v).to_string()),
        ValueRef::Blob(v) => SqliteValue::Blob(v.to_vec()),
    }
}

struct Component;

impl Guest for Component {
    fn open(path: String) -> Result<u32, SqliteError> {
        with_manager(|manager| manager.open(&path))
    }

    fn exec(db: u32, sql: String) -> Result<(), SqliteError> {
        with_manager(|manager| manager.exec(db, &sql))
    }

    fn prepare(db: u32, sql: String) -> Result<u32, SqliteError> {
        with_manager(|manager| manager.prepare(db, &sql))
    }

    fn run(statement: u32) -> Result<SqliteRunInfo, SqliteError> {
        with_manager(|manager| manager.run(statement))
    }

    fn one(statement: u32) -> Result<Option<SqliteRow>, SqliteError> {
        with_manager(|manager| manager.one(statement))
    }

    fn all(statement: u32) -> Result<Vec<SqliteRow>, SqliteError> {
        with_manager(|manager| manager.all(statement))
    }

    fn close(db: u32) -> Result<(), SqliteError> {
        with_manager(|manager| manager.close_db(db))
    }

    fn release(statement: u32) -> Result<(), SqliteError> {
        with_manager(|manager| manager.release_statement(statement))
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::Manager;

    #[test]
    fn end_to_end_query_works() {
        let mut manager = Manager::default();
        let db = manager.open(":memory:").expect("open should work");

        manager
            .exec(db, "create table items (id integer, name text)")
            .expect("create should run");

        let insert = manager
            .prepare(db, "insert into items values (1, 'apple'), (2, 'pear')")
            .expect("prepare insert should work");
        let info = manager.run(insert).expect("insert should run");
        assert_eq!(info.changes, 2);
        assert!(info.last_insert_rowid >= 1);

        let select = manager
            .prepare(db, "select id, name from items order by id")
            .expect("prepare select should work");
        let rows = manager.all(select).expect("query should run");

        assert_eq!(rows.len(), 2);
        manager.close_db(db).expect("close db should work");
    }
}
