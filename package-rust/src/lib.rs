use std::collections::BTreeMap;

mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "sqlite-app",
    });
}

use bindings::wasm::sqlite_wasi::sqlite::{
    close, exec, open as open_db, prepare, SqliteError, SqliteRunInfo, SqliteValue,
};

pub type Value = SqliteValue;
pub const NO_PARAMS: [Value; 0] = [];
pub type RunInfo = SqliteRunInfo;
pub type Row = BTreeMap<String, Value>;

#[derive(Debug, Clone)]
pub struct Error {
    pub code: i32,
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (code: {})", self.message, self.code)
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

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Real(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Self::Blob(value)
    }
}

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Self {
        Self::Blob(value.to_vec())
    }
}

pub struct Statement {
    stmt: bindings::wasm::sqlite_wasi::sqlite::Statement,
}

impl Statement {
    pub fn run<P>(&self, params: &[P]) -> Result<RunInfo, Error>
    where
        P: Clone + Into<Value>,
    {
        let params = params_to_values(params);
        self.stmt.run(Some(&params)).map_err(Into::into)
    }

    pub fn one<P>(&self, params: &[P]) -> Result<Option<Row>, Error>
    where
        P: Clone + Into<Value>,
    {
        let params = params_to_values(params);
        let row = self.stmt.one(Some(&params)).map_err(Error::from)?;
        Ok(row.map(row_to_object))
    }

    pub fn all<P>(&self, params: &[P]) -> Result<Vec<Row>, Error>
    where
        P: Clone + Into<Value>,
    {
        let params = params_to_values(params);
        let rows = self.stmt.all(Some(&params)).map_err(Error::from)?;
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
    pub fn exec<P>(&self, sql: &str, params: &[P]) -> Result<u64, Error>
    where
        P: Clone + Into<Value>,
    {
        let params = params_to_values(params);
        exec(self.handle, sql, Some(&params)).map_err(Error::from)
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
            self.exec("BEGIN", &NO_PARAMS)?;
            match f(arg) {
                Ok(result) => {
                    self.exec("COMMIT", &NO_PARAMS)?;
                    Ok(result)
                }
                Err(err) => {
                    let _ = self.exec("ROLLBACK", &NO_PARAMS);
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
        .collect::<BTreeMap<_, _>>()
}

fn params_to_values<P>(params: &[P]) -> Vec<Value>
where
    P: Clone + Into<Value>,
{
    params.iter().cloned().map(Into::into).collect()
}
