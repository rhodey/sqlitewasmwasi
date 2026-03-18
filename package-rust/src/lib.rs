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
pub trait ToParam {
    fn to_param(&self) -> Value;
}

pub const NO_PARAMS: [&dyn ToParam; 0] = [];
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

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Integer(value.into())
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Real(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Real(value.into())
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

impl ToParam for Value {
    fn to_param(&self) -> Value {
        self.clone()
    }
}

impl ToParam for i64 {
    fn to_param(&self) -> Value {
        Value::Integer(*self)
    }
}

impl ToParam for i32 {
    fn to_param(&self) -> Value {
        Value::Integer((*self).into())
    }
}

impl ToParam for f64 {
    fn to_param(&self) -> Value {
        Value::Real(*self)
    }
}

impl ToParam for f32 {
    fn to_param(&self) -> Value {
        Value::Real((*self).into())
    }
}

impl ToParam for str {
    fn to_param(&self) -> Value {
        Value::Text(self.to_string())
    }
}

impl ToParam for String {
    fn to_param(&self) -> Value {
        Value::Text(self.clone())
    }
}

impl ToParam for Vec<u8> {
    fn to_param(&self) -> Value {
        Value::Blob(self.clone())
    }
}

impl ToParam for [u8] {
    fn to_param(&self) -> Value {
        Value::Blob(self.to_vec())
    }
}

impl<T> ToParam for &T
where
    T: ToParam + ?Sized,
{
    fn to_param(&self) -> Value {
        (*self).to_param()
    }
}

pub struct Statement {
    stmt: bindings::wasm::sqlite_wasi::sqlite::Statement,
}

impl Statement {
    pub fn run(&self, params: &[&dyn ToParam]) -> Result<RunInfo, Error> {
        let params = params_to_values(params);
        self.stmt.run(Some(&params)).map_err(Into::into)
    }

    pub fn one(&self, params: &[&dyn ToParam]) -> Result<Option<Row>, Error> {
        let params = params_to_values(params);
        let row = self.stmt.one(Some(&params)).map_err(Error::from)?;
        Ok(row.map(row_to_object))
    }

    pub fn all(&self, params: &[&dyn ToParam]) -> Result<Vec<Row>, Error> {
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
    pub fn exec(&self, sql: &str, params: &[&dyn ToParam]) -> Result<u64, Error> {
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

fn params_to_values(params: &[&dyn ToParam]) -> Vec<Value> {
    params.iter().map(|param| param.to_param()).collect()
}
