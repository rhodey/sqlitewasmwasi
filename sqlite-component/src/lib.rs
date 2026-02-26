use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use rusqlite::params_from_iter;
use rusqlite::types::Value;
use rusqlite::{Connection, OpenFlags};

wit_bindgen::generate!({
    path: "../wit",
    world: "sqlite-component",
});

use exports::wasm::sqlite_wasi::sqlite::{
    Guest, GuestStatement, SqliteError, SqliteRow, SqliteRunInfo, SqliteValue, Statement,
};

struct PreparedStatement {
    db: u32,
    stmt: rusqlite::Statement<'static>,
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
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_URI;
        let conn = Connection::open_with_flags(path, flags).map_err(map_error)?;
        let id = self.alloc_id();
        self.dbs.insert(id, conn);
        Ok(id)
    }

    fn prepare(&mut self, db: u32, sql: &str) -> Result<u32, SqliteError> {
        let conn = self
            .dbs
            .get_mut(&db)
            .ok_or_else(|| invalid_handle("db", db))?;
        let stmt = conn.prepare(sql).map_err(map_error)?;
        // SAFETY: `Connection` values are stored in `self.dbs` and statements are always
        // removed before their owning connection is dropped (`close_db` removes statements
        // first). This makes the widened lifetime valid within `Manager`.
        let stmt = unsafe {
            std::mem::transmute::<rusqlite::Statement<'_>, rusqlite::Statement<'static>>(stmt)
        };

        let id = self.alloc_id();
        self.statements.insert(id, PreparedStatement { db, stmt });
        Ok(id)
    }

    fn exec(
        &self,
        db: u32,
        sql: &str,
        params: Option<Vec<SqliteValue>>,
    ) -> Result<u64, SqliteError> {
        let conn = self.dbs.get(&db).ok_or_else(|| invalid_handle("db", db))?;
        let mut stmt = conn.prepare(sql).map_err(map_error)?;
        let params = values_from_sqlite(params.unwrap_or_default());
        validate_param_count(stmt.parameter_count(), params.len())?;
        stmt.execute(params_from_iter(params.iter()))
            .map(|changes| changes as u64)
            .map_err(map_error)
    }

    fn run(
        &mut self,
        statement: u32,
        params: Option<Vec<SqliteValue>>,
    ) -> Result<SqliteRunInfo, SqliteError> {
        let prepared = self
            .statements
            .get_mut(&statement)
            .ok_or_else(|| invalid_handle("statement", statement))?;
        let params = values_from_sqlite(params.unwrap_or_default());
        validate_param_count(prepared.stmt.parameter_count(), params.len())?;
        let changes = prepared
            .stmt
            .execute(params_from_iter(params.iter()))
            .map_err(map_error)? as u64;

        let conn = self
            .dbs
            .get(&prepared.db)
            .ok_or_else(|| invalid_handle("db", prepared.db))?;
        let last_insert_rowid = conn.last_insert_rowid();

        Ok(SqliteRunInfo {
            changes,
            last_insert_rowid,
        })
    }

    fn one(
        &mut self,
        statement: u32,
        params: Option<Vec<SqliteValue>>,
    ) -> Result<Option<SqliteRow>, SqliteError> {
        Ok(self.all(statement, params)?.into_iter().next())
    }

    fn all(
        &mut self,
        statement: u32,
        params: Option<Vec<SqliteValue>>,
    ) -> Result<Vec<SqliteRow>, SqliteError> {
        let prepared = self
            .statements
            .get_mut(&statement)
            .ok_or_else(|| invalid_handle("statement", statement))?;

        let params = values_from_sqlite(params.unwrap_or_default());
        validate_param_count(prepared.stmt.parameter_count(), params.len())?;
        let col_count = prepared.stmt.column_count();
        let columns = (0..col_count)
            .map(|index| {
                prepared
                    .stmt
                    .column_name(index)
                    .unwrap_or_default()
                    .to_string()
            })
            .collect::<Vec<_>>();
        let mut rows = prepared
            .stmt
            .query(params_from_iter(params.iter()))
            .map_err(map_error)?;

        let mut out = Vec::new();
        while let Some(row) = rows.next().map_err(map_error)? {
            let mut values = Vec::with_capacity(col_count);
            for index in 0..col_count {
                let value = row.get_ref(index).map_err(map_error)?;
                values.push(value_from_ref(value));
            }
            out.push(SqliteRow {
                columns: columns.clone(),
                values,
            });
        }
        Ok(out)
    }

    fn close_db(&mut self, db: u32) -> Result<(), SqliteError> {
        if !self.dbs.contains_key(&db) {
            return Err(invalid_handle("db", db));
        }
        let ids_to_remove = self
            .statements
            .iter()
            .filter_map(|(id, statement)| (statement.db == db).then_some(*id))
            .collect::<Vec<_>>();
        for id in ids_to_remove {
            self.statements.remove(&id);
        }
        self.dbs.remove(&db);
        Ok(())
    }

    fn release_statement(&mut self, statement: u32) {
        self.statements.remove(&statement);
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
    let code = err
        .sqlite_error_code()
        .map(|sqlite_code| sqlite_code as i32)
        .unwrap_or(-1);

    SqliteError {
        code,
        message: err.to_string(),
    }
}

fn invalid_handle(kind: &str, id: u32) -> SqliteError {
    SqliteError {
        code: -2,
        message: format!("unknown {kind} handle: {id}"),
    }
}

fn value_from_ref(value: rusqlite::types::ValueRef<'_>) -> SqliteValue {
    match value {
        rusqlite::types::ValueRef::Null => SqliteValue::Null,
        rusqlite::types::ValueRef::Integer(v) => SqliteValue::Integer(v),
        rusqlite::types::ValueRef::Real(v) => SqliteValue::Real(v),
        rusqlite::types::ValueRef::Text(v) => {
            SqliteValue::Text(String::from_utf8_lossy(v).to_string())
        }
        rusqlite::types::ValueRef::Blob(v) => SqliteValue::Blob(v.to_vec()),
    }
}

fn values_from_sqlite(values: Vec<SqliteValue>) -> Vec<Value> {
    values
        .into_iter()
        .map(|value| match value {
            SqliteValue::Null => Value::Null,
            SqliteValue::Integer(v) => Value::Integer(v),
            SqliteValue::Real(v) => Value::Real(v),
            SqliteValue::Text(v) => Value::Text(v),
            SqliteValue::Blob(v) => Value::Blob(v),
        })
        .collect()
}

fn validate_param_count(expected_count: usize, actual_count: usize) -> Result<(), SqliteError> {
    if expected_count != actual_count {
        return Err(SqliteError {
            code: -3,
            message: format!(
                "bind parameter mismatch: expected {expected_count}, got {actual_count}"
            ),
        });
    }
    Ok(())
}

struct StatementResource {
    id: u32,
    released: Cell<bool>,
}

impl StatementResource {
    fn release_inner(&self) -> bool {
        if self.released.get() {
            return false;
        }
        with_manager(|manager| manager.release_statement(self.id));
        self.released.set(true);
        true
    }
}

impl Drop for StatementResource {
    fn drop(&mut self) {
        if !self.released.get() {
            with_manager(|manager| manager.release_statement(self.id));
            self.released.set(true);
        }
    }
}

struct Component;

impl Guest for Component {
    type Statement = StatementResource;

    fn open(path: String) -> Result<u32, SqliteError> {
        with_manager(|manager| manager.open(&path))
    }

    fn exec(db: u32, sql: String, params: Option<Vec<SqliteValue>>) -> Result<u64, SqliteError> {
        with_manager(|manager| manager.exec(db, &sql, params))
    }

    fn prepare(db: u32, sql: String) -> Result<Statement, SqliteError> {
        let statement_id = with_manager(|manager| manager.prepare(db, &sql))?;
        Ok(Statement::new(StatementResource {
            id: statement_id,
            released: Cell::new(false),
        }))
    }

    fn close(db: u32) -> Result<(), SqliteError> {
        with_manager(|manager| manager.close_db(db))
    }
}

impl GuestStatement for StatementResource {
    fn run(&self, params: Option<Vec<SqliteValue>>) -> Result<SqliteRunInfo, SqliteError> {
        if self.released.get() {
            return Err(invalid_handle("statement", self.id));
        }
        with_manager(|manager| manager.run(self.id, params))
    }

    fn one(&self, params: Option<Vec<SqliteValue>>) -> Result<Option<SqliteRow>, SqliteError> {
        if self.released.get() {
            return Err(invalid_handle("statement", self.id));
        }
        with_manager(|manager| manager.one(self.id, params))
    }

    fn all(&self, params: Option<Vec<SqliteValue>>) -> Result<Vec<SqliteRow>, SqliteError> {
        if self.released.get() {
            return Err(invalid_handle("statement", self.id));
        }
        with_manager(|manager| manager.all(self.id, params))
    }

    fn release(&self) -> bool {
        self.release_inner()
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::{Manager, SqliteRow, SqliteValue};

    fn value_by_name<'a>(row: &'a SqliteRow, name: &str) -> Option<&'a SqliteValue> {
        row.columns
            .iter()
            .position(|column| column == name)
            .and_then(|index| row.values.get(index))
    }

    #[test]
    fn end_to_end_query_works() {
        let mut manager = Manager::default();
        let db = manager.open(":memory:").expect("open should work");

        manager
            .exec(db, "create table items (id integer, name text)", None)
            .expect("create should run");

        let insert = manager
            .prepare(db, "insert into items values (?, ?), (?, ?)")
            .expect("prepare insert should work");
        let info = manager
            .run(
                insert,
                Some(vec![
                    SqliteValue::Integer(1),
                    SqliteValue::Text("apple".to_string()),
                    SqliteValue::Integer(2),
                    SqliteValue::Text("pear".to_string()),
                ]),
            )
            .expect("insert should run");
        assert_eq!(info.changes, 2);
        assert!(info.last_insert_rowid >= 1);

        let select = manager
            .prepare(db, "select id, name from items where id > ? order by id")
            .expect("prepare select should work");
        let rows = manager
            .all(select, Some(vec![SqliteValue::Integer(0)]))
            .expect("query should run");

        assert_eq!(rows.len(), 2);
        for row in &rows {
            println!("id={:?}", value_by_name(row, "id"));
            println!("name={:?}", value_by_name(row, "name"));
        }

        let row = rows.first().expect("should have at least one row");
        assert_eq!(value_by_name(row, "id"), Some(&SqliteValue::Integer(1)));
        assert_eq!(
            value_by_name(row, "name"),
            Some(&SqliteValue::Text("apple".to_string()))
        );
        manager.close_db(db).expect("close db should work");
    }
}
