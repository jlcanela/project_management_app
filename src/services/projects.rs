use std::sync::Arc;

use crate::AppState;
use actix_web::{
    error, get,
    http::{header::ContentType, StatusCode},
    post,
    web::{self, Data, Json, ReqData},
    HttpResponse, Responder,
};

use crate::services::*;

use derive_more::From;
use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};

#[derive(Deserialize)]
struct CreateProjectBody {
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize, FromRow, Debug, PartialEq, Eq)]
struct Project {
    id: i32,
    name: String,
    description: String,
}

pub type Result<T> = std::result::Result<T, ProjectError>;

#[allow(dead_code)]
#[derive(Debug, From)]
pub enum ProjectError {
    Unknown,
    AuthFailed,
    #[from]
    Io(std::io::Error),
    #[from]
    Parse(std::num::ParseIntError),
    #[from]
    Sqlx(sqlx::Error),
    #[from]
    Serde(serde_json::Error),
    #[from]
    TokenError(TokenError),
}

impl core::fmt::Display for ProjectError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for ProjectError {}

impl error::ResponseError for ProjectError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ProjectError::Parse(_) => StatusCode::BAD_REQUEST,
            ProjectError::AuthFailed => StatusCode::FORBIDDEN,
            ProjectError::Unknown
            | ProjectError::Io(_)
            | ProjectError::Sqlx(_)
            | ProjectError::Serde(_)
            | ProjectError::TokenError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[post("/api/projects")]
async fn create_project(state: Data<Arc<AppState>>, body: Json<CreateProjectBody>) -> impl Responder {
    let project = body.into_inner();

    match sqlx::query_as::<_, Project>(
        "INSERT INTO projects (name, description, owned_by, created_by, updated_by)
        VALUES ($1, $2, 2, 2, 2)
        RETURNING id, name, description",
    )
    .bind(project.name)
    .bind(project.description)
    .fetch_one(&state.db)
    .await
    {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(error) => HttpResponse::InternalServerError().json(format!("Error: {:?}", error)),
    }
}

//use actix_web::{test, App};

#[cfg(test)]
mod tests {

    use super::*;
    use crate::repository::{Migrate, Repository};
    use crate::validator;
    use actix_http::Request;
    use actix_web::dev::Service;
    use actix_web::http::header::ContentType;
    use actix_web::test;
    use actix_web::App;
    use actix_web_httpauth::middleware::HttpAuthentication;
    use dotenv::dotenv;
    use serde_json::Value;
    use sqlx::any::AnyPoolOptions;
    use sqlx::{Any, Pool};

    async fn create_app_data() -> Result<AppState> {
        dotenv().ok();

        let db_url = "sqlite::memory:";

        let pool: Pool<Any> = {
            AnyPoolOptions::new()
            .max_connections(1)
            .connect(&db_url)
            .await.unwrap()
        };

        let repo = Repository::new(&pool, "./sqlite-migrations").await;
        
        let migration_error = repo.migrate().await.map_err(|err| err.to_string());
        assert_eq!(migration_error, Ok(()));

        Ok(AppState {
            db: pool.clone(),
            permission: Permission::default(),
        })
    }

    async fn create_test_app() -> impl Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error>
     {
        dotenv().ok();
        sqlx::any::install_default_drivers();
        let bearer_middleware = HttpAuthentication::bearer(validator);
       
        let app_data = Arc::new(create_app_data().await.unwrap());

        test::init_service(
            App::new()
                .app_data(Data::new(app_data.clone()))
                .service(
                    web::scope("")
                        .wrap(bearer_middleware.clone())
                        .service(list_projects),
                ),
        )
        .await
    }

    fn view_project_req(token_claims: TokenClaims) -> Request {
        let token = TokenService::generate_token(token_claims).unwrap();
        
        let req = test::TestRequest::get().uri("/api/projects")
            .insert_header(ContentType::plaintext())
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        req
    }

    #[actix_web::test]
    async fn admin_cannot_view_projects() {
        let app = create_test_app().await;

        let token_claims = TokenClaims { 
            id: 2,
            roles: vec!["Administrator".to_string()]
        };
        
        let req = view_project_req(token_claims);
        let projects: Vec<Project> = test::call_and_read_body_json(&app, req).await;

        assert_eq!(projects, vec![]);    
    }

    #[actix_web::test]
    async fn projectlead_can_view_projects() {
        let app = create_test_app().await;

        let token_claims = TokenClaims { 
            id: 3,
            roles: vec!["ProjectLead".to_string()]
        };
        
        let req = view_project_req(token_claims);
        let projects: Vec<Project> = test::call_and_read_body_json(&app, req).await;

        assert_eq!(projects, vec![
            Project { id: 1, name: "my project".to_string(), description: "this project".to_string() },
            Project { id: 2, name: "my other project".to_string(), description: "that project".to_string() },
        ]);    
    }

    #[actix_web::test]
    async fn developer_can_view_projects() {
        let app = create_test_app().await;

        let token_claims = TokenClaims { 
            id: 4,
            roles: vec!["Developer".to_string()]
        };
        
        let req = view_project_req(token_claims);
        let projects: Vec<Project> = test::call_and_read_body_json(&app, req).await;

        assert_eq!(projects, vec![
            Project { id: 2, name: "my other project".to_string(), description: "that project".to_string() },
        ]);  

    }

}

#[get("/")]
async fn index_get() -> impl Responder {
    HttpResponse::Ok().body("Ok")
}

#[post("/")]
async fn index_post() -> impl Responder {
    HttpResponse::Ok().body("Ok")
}

#[get("/api/projects")]
async fn list_projects(
    state: Data<Arc<AppState>>,
    token_claims: Option<ReqData<TokenClaims>>,
) -> Result<String> {
    match token_claims {
        Some(token_claims) => {
            let tk = TokenClaims {
                id: token_claims.id,
                roles: token_claims.roles.clone(),
            };
            let ans = state.permission.get_policies(&tk, Action::ViewProject);

            let policies = match ans {
                Ok(ResourceAuthorizationResult::Deny) => return Ok("[]".to_string()),
                Ok(ResourceAuthorizationResult::Residual(policies)) => {
                    let ids = policies.iter().filter_map(|p| p.annotation("id").map(|id| id.to_string())).collect();
                    ids
                },
                _ => vec![]
            };

            let projectlead_project = policies.contains(&"ProjectLeadPolicy.Project".to_string());
            let developer_project = policies.contains(&"DeveloperPolicy".to_string());

            let sql_result =
                sqlx::query_as::<_, Project>("SELECT id, name, description FROM projects
                LEFT JOIN assignments ON assignments.project_id = projects.id                -- developer policy
                LEFT JOIN party_role ON assignments.party_role_id = party_role.party_role_id -- developer policy
                WHERE (NOT $1 OR projects.owned_by = $2) AND                                 -- project lead policy
                (NOT $3 OR party_role.party_id = $2);                                        -- developer policy
                ")
                    .bind(projectlead_project)
                    .bind(tk.id.clone())
                    .bind(developer_project)
                    .fetch_all(&state.db)
                    .await;

            let err = match &sql_result {
                Err(err) => err.to_string(),
                Ok(_) => "".to_string()
            };
            let projects = sql_result?;

            let json = serde_json::to_string(&projects)?;
            Ok(json)
        }
        _ => Err(ProjectError::AuthFailed),
    }
}

#[get("/api/projects/{id}")]
async fn get_project(
    state: Data<Arc<AppState>>,
    token_claims: Option<ReqData<TokenClaims>>,
    path: web::Path<String>,
) -> Result<String> {
    match token_claims {
        Some(token_claims) => {
            let tk = TokenClaims {
                id: token_claims.id,
                roles: token_claims.roles.clone(),
            };
            let project_id: String = path.into_inner();
            let id = project_id.parse::<i64>()?;
            let project = sqlx::query_as::<_, Project>(
                "SELECT id, name, description from projects
                WHERE id = $1",
            )
            .bind(id)
            .fetch_one(&state.db)
            .await?;

            if state
                .permission
                .is_authorized(&tk, Action::ViewProject, &project)
                .is_err()
            {
                return Err(ProjectError::AuthFailed);
            }

            let json = serde_json::to_string(&project)?;
            Ok(json)
        }
        _ => Err(ProjectError::AuthFailed),
    }
}

#[get("/status")]
async fn status(_state: Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(r#"{ "status": "Ok" }"#)
}
