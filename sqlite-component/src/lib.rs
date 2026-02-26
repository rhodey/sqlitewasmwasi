use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};

use rusqlite::types::ValueRef;
use rusqlite::Connection;

wit_bindgen::generate!({
    path: "../wit",
    world: "sqlite-component",
});

use exports::wasm::wasi_sqlite::sqlite::{Guest, SqliteError, SqliteRow, SqliteValue};

#[derive(Debug)]
struct PreparedStatement {
    db: u32,
    sql: String,
}

#[derive(Default)]
struct Manager {
    next_id: AtomicU32,
    dbs: Mutex<HashMap<u32, Connection>>,
    statements: Mutex<HashMap<u32, PreparedStatement>>,
}

impl Manager {
    fn alloc_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn open(&self, path: &str) -> Result<u32, SqliteError> {
        let conn = Connection::open(path).map_err(map_error)?;
        let id = self.alloc_id();
        self.dbs.lock().expect("db lock poisoned").insert(id, conn);
        Ok(id)
    }

    fn prepare(&self, db: u32, sql: &str) -> Result<u32, SqliteError> {
        {
            let dbs = self.dbs.lock().expect("db lock poisoned");
            let conn = dbs.get(&db).ok_or_else(|| invalid_handle("db", db))?;
            conn.prepare(sql).map_err(map_error)?;
        }

        let id = self.alloc_id();
        self.statements
            .lock()
            .expect("statement lock poisoned")
            .insert(
                id,
                PreparedStatement {
                    db,
                    sql: sql.to_string(),
                },
            );
        Ok(id)
    }

    fn query(&self, statement: u32) -> Result<Vec<SqliteRow>, SqliteError> {
        let prepared = {
            let statements = self.statements.lock().expect("statement lock poisoned");
            statements
                .get(&statement)
                .ok_or_else(|| invalid_handle("statement", statement))?
                .to_owned()
        };

        let dbs = self.dbs.lock().expect("db lock poisoned");
        let conn = dbs
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

    fn close_db(&self, db: u32) -> Result<(), SqliteError> {
        let mut dbs = self.dbs.lock().expect("db lock poisoned");
        dbs.remove(&db).ok_or_else(|| invalid_handle("db", db))?;

        let mut statements = self.statements.lock().expect("statement lock poisoned");
        statements.retain(|_, statement| statement.db != db);
        Ok(())
    }

    fn close_statement(&self, statement: u32) -> Result<(), SqliteError> {
        let mut statements = self.statements.lock().expect("statement lock poisoned");
        statements
            .remove(&statement)
            .ok_or_else(|| invalid_handle("statement", statement))?;
        Ok(())
    }
}

impl Clone for PreparedStatement {
    fn clone(&self) -> Self {
        Self {
            db: self.db,
            sql: self.sql.clone(),
        }
    }
}

static MANAGER: OnceLock<Manager> = OnceLock::new();

fn manager() -> &'static Manager {
    MANAGER.get_or_init(Manager::default)
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
        manager().open(&path)
    }

    fn prepare(db: u32, sql: String) -> Result<u32, SqliteError> {
        manager().prepare(db, &sql)
    }

    fn query(statement: u32) -> Result<Vec<SqliteRow>, SqliteError> {
        manager().query(statement)
    }

    fn close(db: u32) -> Result<(), SqliteError> {
        manager().close_db(db)
    }

    fn release(statement: u32) -> Result<(), SqliteError> {
        manager().close_statement(statement)
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::Manager;

    #[test]
    fn end_to_end_query_works() {
        let manager = Manager::default();
        let db = manager.open(":memory:").expect("open should work");

        let create = manager
            .prepare(db, "create table items (id integer, name text)")
            .expect("prepare create should work");
        manager.query(create).expect("create should run");
        manager.close_statement(create).expect("close should work");

        let insert = manager
            .prepare(db, "insert into items values (1, 'apple'), (2, 'pear')")
            .expect("prepare insert should work");
        manager.query(insert).expect("insert should run");

        let select = manager
            .prepare(db, "select id, name from items order by id")
            .expect("prepare select should work");
        let rows = manager.query(select).expect("query should run");

        assert_eq!(rows.len(), 2);
        manager.close_db(db).expect("close db should work");
    }
}
