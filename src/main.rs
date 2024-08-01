use actix_web::{
    dev::ServiceRequest,
    error,
    error::Error,
    http::{header::ContentType, StatusCode},
    web::{self, Data},
    App, HttpMessage, HttpResponse, HttpServer,
};

use dotenv::dotenv;
use sqlx::{self, any::AnyPoolOptions};
use sqlx::{Any, Pool};
use std::{io, sync::Arc};

use actix_web_httpauth::{
    extractors::{
        bearer::{self, BearerAuth},
        AuthenticationError,
    },
    middleware::HttpAuthentication,
};

use derive_more::From;

mod services;
use services::Permission;
use services::{status, TokenService};
use services::{create_project, get_project, list_projects};

mod repository;

#[derive(Clone)]
pub struct AppState {
    db: Pool<Any>,
    permission: Permission,
}

async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> std::result::Result<ServiceRequest, (Error, ServiceRequest)> {
    let token_service = TokenService::default();
    let token = credentials.token();
    let token_claims = token_service.verify(token).await;

    match token_claims {
        Ok(value) => {
            req.extensions_mut().insert(value);
            Ok(req)
        }
        Err(err) => {
            dbg!(&err);
            println!("{:?}", &err);
            let config = req
                .app_data::<bearer::Config>()
                .cloned()
                .unwrap_or_default()
                .scope("");
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}

pub type Result<T> = std::result::Result<T, MainError>;

#[allow(dead_code)]
#[derive(Debug, From)]
pub enum MainError {
    Unknown,
    #[from]
    Sqlx(sqlx::Error),
    #[from]
    MigrateError(sqlx::migrate::MigrateError),
    #[from]
    IoError(io::Error),
    #[from]
    RepositoryError(repository::RepositoryError),
}

impl core::fmt::Display for MainError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for MainError {}

impl error::ResponseError for MainError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            MainError::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            MainError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MainError::MigrateError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MainError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MainError::RepositoryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    // Configure the connection options

    sqlx::any::install_default_drivers();

    let pool: Pool<Any> = {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");

        AnyPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .unwrap()
    };

    let a = HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(validator);
        let app_state = Arc::new(AppState {
            db: pool.clone(),
            permission: Permission::default(),
        });
        App::new()
            .app_data(Data::new(app_state))
            .service(status)
            //.service(basic_auth)
            //.service(create_user)
            .service(
                web::scope("")
                    .wrap(bearer_middleware.clone())
                    .service(list_projects),
            )
            .service(
                web::scope("")
                    .wrap(bearer_middleware.clone())
                    .service(get_project),
            )
            .service(create_project)
        // .service(
        //     web::scope("")
        //         .wrap(bearer_middleware)
        //         .service(create_article)
        //)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;
    Ok(a)
}
