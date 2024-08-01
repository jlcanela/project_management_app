use cedar_policy::{
    Entities, EntitiesError, Entity, EntityAttrEvaluationError, EntityId, EntityTypeName,
    EntityUid, RestrictedExpression, Schema,
};
use derive_more::From;
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

pub type Result<T> = std::result::Result<T, TokenError>;

#[allow(dead_code)]
#[derive(Debug, From)]
pub enum TokenError {
    #[from]
    EntityAttrEvaluationError(EntityAttrEvaluationError),
    #[from]
    EntitiesError(EntitiesError),
    #[from]
    JwtError(jwt::Error),
}

impl core::fmt::Display for TokenError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for TokenError {}

// const ENTITY_TYPE_GROUP: &str = "Group";
// const ENTITY_TYPE_ROLE: &str = "Role";
const ENTITY_TYPE_USER: &str = "User";
// const ENTITY_TYPE_PROJECT: &str = "Project";

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub id: i32,
    pub roles: Vec<String>,
}

impl TokenClaims {
    pub fn user(&self) -> Result<Entity> {
        let user_type = EntityTypeName::from_str(ENTITY_TYPE_USER).unwrap();

        let euid =
            EntityUid::from_type_name_and_id(user_type.clone(), EntityId::new(self.id.to_string()));
        let attrs: HashMap<String, RestrictedExpression> = HashMap::new();
        /*from([
            ("age".to_string(), RestrictedExpression::from_str("21").unwrap()),
            ("department".to_string(), RestrictedExpression::from_str("\"CS\"").unwrap()),
        ]);*/

        let parents: HashSet<EntityUid> = self.roles_ids().collect();
        let u = Entity::new(euid, attrs, parents)?;
        Ok(u)
    }

    pub fn roles_ids(&self) -> impl Iterator<Item = EntityUid> {
        let role_type = EntityTypeName::from_str("Role").unwrap();
        self.roles
            .clone()
            .into_iter()
            .map(move |r| EntityUid::from_type_name_and_id(role_type.clone(), EntityId::new(r)))
    }

    pub fn roles(&self) -> impl Iterator<Item = Entity> {
        self.roles_ids().map(|e| Entity::with_uid(e))
    }

    pub fn entities(&self, schema: Option<&Schema>) -> Result<Entities> {
        let u = self.user()?;
        let iter = self.roles().chain(vec![u]);
        let entities = Entities::from_entities(iter, schema)?;
        Ok(entities)
    }
}

pub struct TokenService {
    jwt_secret: String,
}

impl Default for TokenService {
    fn default() -> Self {
        TokenService {
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set!"),
        }
    }
}

impl TokenService {
    fn new(jwt_secret: &str) -> TokenService {
        TokenService {
            jwt_secret: jwt_secret.to_string(),
        }
    }

    pub fn generate_token(token_claims: TokenClaims) -> Result<String> {
        let jwt_secret: Hmac<Sha256> = Hmac::new_from_slice(
            std::env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set!")
                .as_bytes(),
        )
        .unwrap();
        let token = token_claims.sign_with_key(&jwt_secret)?;
        Ok(token)
    }

    pub async fn verify(&self, bearer_token: &str) -> Result<TokenClaims> {
        let key: Hmac<Sha256> = Hmac::new_from_slice(self.jwt_secret.as_bytes()).unwrap();
        let claims: TokenClaims = bearer_token.verify_with_key(&key)?;
        Ok(claims)
    }
}
