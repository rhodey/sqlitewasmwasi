use std::collections::HashMap;

mod bindings {
    wit_bindgen::generate!({
        path: "../wit",
        world: "sqlite-app",
    });
}

use bindings::wasm::sqlite_wasi::sqlite::{
    close, exec, open as open_db, prepare, SqliteError, SqliteRunInfo, SqliteValue,
};

pub type Value = SqliteValue;
pub type RunInfo = SqliteRunInfo;
pub type Row = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub struct Error {
    pub code: i32,
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

impl From<SqliteError> for Error {
    fn from(value: SqliteError) -> Self {
        Self {
            code: value.code,
            message: value.message,
        }
    }
}

pub struct Statement {
    stmt: bindings::wasm::sqlite_wasi::sqlite::Statement,
}

impl Statement {
    pub fn run(&self, params: &[Value]) -> Result<RunInfo, Error> {
        self.stmt.run(Some(params)).map_err(Into::into)
    }

    pub fn one(&self, params: &[Value]) -> Result<Option<Row>, Error> {
        let row = self.stmt.one(Some(params)).map_err(Error::from)?;
        Ok(row.map(row_to_object))
    }

    pub fn all(&self, params: &[Value]) -> Result<Vec<Row>, Error> {
        let rows = self.stmt.all(Some(params)).map_err(Error::from)?;
        Ok(rows.into_iter().map(row_to_object).collect())
    }

    pub fn release(&self) -> Result<bool, Error> {
        Ok(self.stmt.release())
    }
}

#[derive(Clone, Copy)]
pub struct Database {
    handle: u32,
}

impl Database {
    pub fn exec(&self, sql: &str, params: &[Value]) -> Result<u64, Error> {
        exec(self.handle, sql, Some(params)).map_err(Error::from)
    }

    pub fn prepare(&self, sql: &str) -> Result<Statement, Error> {
        let stmt = prepare(self.handle, sql).map_err(Error::from)?;
        Ok(Statement { stmt })
    }

    pub fn transaction<'a, F, T, A>(&'a self, mut f: F) -> impl FnMut(A) -> Result<T, Error> + 'a
    where
        F: FnMut(A) -> Result<T, Error> + 'a,
    {
        move |arg| {
            self.exec("BEGIN", &[])?;
            match f(arg) {
                Ok(result) => {
                    self.exec("COMMIT", &[])?;
                    Ok(result)
                }
                Err(err) => {
                    let _ = self.exec("ROLLBACK", &[]);
                    Err(err)
                }
            }
        }
    }

    pub fn close(&self) -> Result<(), Error> {
        close(self.handle).map_err(Error::from)
    }
}

pub fn open(uri: &str) -> Result<Database, Error> {
    let uri = if uri.starts_with("file:") {
        uri.to_string()
    } else {
        format!("file:{uri}")
    };
    let handle = open_db(&uri).map_err(Error::from)?;
    Ok(Database { handle })
}

pub fn row_value<'a>(row: &'a Row, name: &str) -> Option<&'a Value> {
    row.get(name)
}

fn row_to_object(row: bindings::wasm::sqlite_wasi::sqlite::SqliteRow) -> Row {
    row.columns
        .into_iter()
        .zip(row.values)
        .collect::<HashMap<_, _>>()
}
