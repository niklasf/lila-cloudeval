use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use shakmaty::{Chess, PositionError};
use terarkdb::Error as DbError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("database error: {0}")]
    DbError(#[from] DbError),
    #[error("bad request: {0}")]
    PositionError(#[from] PositionError<Chess>),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            match self {
                Error::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                Error::PositionError(_) => StatusCode::BAD_REQUEST,
            },
            self.to_string(),
        )
            .into_response()
    }
}
