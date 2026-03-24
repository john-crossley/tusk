use serde::Serialize;

const SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Serialize)]
pub struct Response<T> {
    schema_version: u8,
    command: &'static str,
    data: T,
}

impl<T> Response<T> {
    pub fn new(command: &'static str, data: T) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            command,
            data,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse<T> {
    schema_version: u8,
    command: &'static str,
    error: T,
}

impl<T> ErrorResponse<T> {
    pub fn new(command: &'static str, error: T) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            command,
            error,
        }
    }
}