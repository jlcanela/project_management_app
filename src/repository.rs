
use actix_web::{
    error, http::{header::ContentType, StatusCode}, HttpResponse
};


use std::path::Path;

use sqlx::{
    self, Any, Pool
};
use derive_more::From;

pub type Result<T> = std::result::Result<T, RepositoryError>;

#[allow(dead_code)]
#[derive(Debug, From)]
pub enum RepositoryError {
    Unknown,
    #[from]
    Sqlx(sqlx::Error),
    #[from]
    MigrateError(sqlx::migrate::MigrateError),
    #[from]
    IoError(std::io::Error),

}

impl core::fmt::Display for RepositoryError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for RepositoryError {}

impl error::ResponseError for RepositoryError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            RepositoryError::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            RepositoryError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RepositoryError::MigrateError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RepositoryError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub trait Migrate {
    async fn migrate(&self) -> Result<()>;
}

pub struct Repository {
    pool: Pool<Any>,
    migration_path: String,
    //pool: Either<Pool<Postgres>, Pool<Sqlite>>,
}

impl Migrate for Repository {
    async fn migrate(&self) -> Result<()> {
        use sqlx::migrate::Migrator;
        
        let path = Path::new(&self.migration_path);
        let migrator = Migrator::new(path).await?;
        migrator.run(&self.pool).await?;
        Ok(())
    }
}

impl Repository {
    pub async fn new(pool: &Pool<Any>, migration_path: &str) -> Repository {
        Repository { pool: pool.clone(), migration_path: migration_path.to_string() }
    }

}
