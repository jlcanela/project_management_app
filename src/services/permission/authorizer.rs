use std::fs;

use super::{action::*, TokenClaims, TokenError};

// use cedar_policy::PrincipalConstraint::{Any, Eq, In, Is, IsIn};
use cedar_policy::{
    Authorizer, Context, Decision, EntityId, EntityTypeName, EntityUid, Policy, PolicySet, Request,
    RequestBuilder,
    Schema, /*SlotId, Template,*/
            //ValidationMode, ValidationResult, Validator,
};

use derive_more::From;
use dotenv::dotenv;
use std::str::FromStr;

pub type Result<T> = std::result::Result<T, AuthorizerError>;

#[allow(dead_code)]
#[derive(Debug, From)]
pub enum AuthorizerError {
    #[from]
    RequestValidationError(cedar_policy_validator::RequestValidationError),
    #[from]
    TokenError(TokenError),
}

impl core::fmt::Display for AuthorizerError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

trait Condition {
    fn condition(&self) -> Option<String>;
}

impl Condition for Policy {
    fn condition(&self) -> Option<String> {
        let json = self.to_json().expect("should convert policy to JSON");
        let o = json.as_object()?;
        o.get("conditions").map(|c| c.to_string())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResourceAuthorizationResult {
    Allow,
    Deny,
    Residual(Vec<Policy>),
}

#[derive(Debug, Clone)]
pub struct Permission {
    policies: PolicySet,
    schema: Schema,
}

impl Default for Permission {
    fn default() -> Self {
        let policies = fs::read_to_string("cedar-policies/projects/policies.cedar")
            .expect("Should have been able to read the 'policies' file");
        let policies = PolicySet::from_str(&policies).unwrap();
        let schema = fs::read_to_string("cedar-policies/projects/projects.cedarschema")
            .expect("Should have been able to read the 'schema' file");
        let (schema, warnings) = Schema::from_str_natural(&schema).unwrap();

        for w in warnings {
            println!("{:?}", w);
        }

        Self {
            policies,
            schema: schema,
        }
    }
}

impl Permission {
    fn new(policies: &str) -> Self {
        let policies = PolicySet::from_str(policies).unwrap();
        let schema = fs::read_to_string("cedar-policies/projects/projects.cedarschema")
            .expect("Should have been able to read the 'schema' file");
        let (schema, warnings) = Schema::from_str_natural(&schema).unwrap();

        for w in warnings {
            println!("{:?}", w);
        }

        Self { policies, schema }
    }

    fn project() -> EntityUid {
        let r_eid = EntityId::from_str("1").unwrap(); // does not go through the parser
        let r_name: EntityTypeName = EntityTypeName::from_str("Project").unwrap(); // through parse_name(s)
        EntityUid::from_type_name_and_id(r_name, r_eid)
    }

    pub fn is_authorized<T>(
        &self,
        token_claims: &TokenClaims,
        action: Action,
        _resource: &T,
    ) -> Result<bool> {
        let authorizer = Authorizer::new();

        let p = token_claims.user().map(|u| u.uid())?;
        let a: EntityUid = action.into();
        let r = Permission::project();

        let request: Request = Request::new(
            Some(p),
            Some(a),
            Some(r),
            Context::empty(),
            Some(&self.schema),
        )?;

        let entities = token_claims.entities(Some(&self.schema)).unwrap();

        let ans = authorizer.is_authorized(&request, &self.policies, &entities);

        for reason in ans.diagnostics().reason() {
            //print all the annotations
            for (key, value) in self.policies.policy(reason).unwrap().annotations() {
                println!("PolicyID: {}\tKey:{} \tValue:{}", reason, key, value);
            }
        }
        Ok(ans.decision() == Decision::Allow)
    }

    pub fn get_policies(
        &self,
        token_claims: &TokenClaims,
        action: Action,
    ) -> Result<ResourceAuthorizationResult> {
        dotenv().ok();

        let authorizer = Authorizer::new();

        let user = token_claims.user()?;
        let request: Request = RequestBuilder::default()
            .principal(Some(user.uid()))
            .action(Some(action.into()))
            .build();

        let entities = token_claims.entities(Some(&self.schema)).unwrap();

        let ans = authorizer.is_authorized_partial(&request, &self.policies, &entities);
        match ans.decision() {
            Some(Decision::Allow) => Ok(ResourceAuthorizationResult::Allow),
            Some(Decision::Deny) => Ok(ResourceAuthorizationResult::Deny),
            None => {
                let policies: Vec<Policy> = ans.may_be_determining().collect();
                Ok(ResourceAuthorizationResult::Residual(policies))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::services::TokenClaims;

    use super::*;

    const ADMIN_POLICY: &str = r#"@id("AdministratorPolicy")
permit(
  principal in Role::"Administrator",
  action in [Action::"CreateParty", Action::"CreateRole", Action::"AssignRole", Action::"RemoveRole"], 
  resource
);"#;

    const PROJECTLEAD_POLICY: &str = r#"@id("ProjectLeadPolicy")
permit(
  principal in Role::"ProjectLead",
  action in [Action::"CreateProject", Action::"ListProject"],
  resource in Group::"Projects"
);"#;

    const PROJECTLEAD_PROJECT_POLICY: &str = r#"@id("ProjectLeadPolicy.Project")
permit(
  principal,
  action in [Action::"ViewProject", Action::"AuditProject", Action::"AssignPartyrole", Action::"ShareProject"],
  resource
)
when { principal == resource.owner };"#;

    const DEVELOPER_POLICY: &str = r#"
@id("DeveloperPolicy")
permit(
  principal in Role::"Developer",
  action in [Action::"ListProject", Action::"ViewProject"],
  resource
)
when { principal in resource.assigned_to };"#;

    fn evaluate(
        token_claims: TokenClaims,
        policies: &str,
        action: Action,
    ) -> Result<ResourceAuthorizationResult> {
        dotenv().ok();

        let permission = Permission::new(policies);
        permission.get_policies(&token_claims, action)
    }

    #[test]
    fn admin_policy_is_denied() {
        let token_claims = TokenClaims {
            id: 1,
            roles: vec!["Administrator".to_string()],
        };
        let ans = evaluate(token_claims, ADMIN_POLICY, Action::ViewProject);
        assert!(ans.is_ok(), "ans is ok");
        assert_eq!(
            &ans.unwrap(),
            &ResourceAuthorizationResult::Deny,
            "Expected Ok(Deny)"
        );
    }

    #[test]
    fn projectlead_policy_is_denied() {
        let token_claims = TokenClaims {
            id: 1,
            roles: vec!["ProjectLead".to_string()],
        };
        let ans = evaluate(token_claims, PROJECTLEAD_POLICY, Action::ViewProject);
        assert!(ans.is_ok(), "ans is ok");
        assert_eq!(
            &ans.unwrap(),
            &ResourceAuthorizationResult::Deny,
            "Expected Ok(Deny)"
        );
    }

    #[test]
    fn projectlead_project_policy_is_residual() {
        let expected = r#"[{"kind":"when","body":{"&&":{"left":{"Value":true},"right":{"==":{"left":{"Value":{"__entity":{"type":"User","id":"1"}}},"right":{".":{"left":{"unknown":[{"Value":"resource"}]},"attr":"owner"}}}}}}}]"#.to_string();

        let token_claims = TokenClaims {
            id: 1,
            roles: vec!["ProjectLead".to_string()],
        };

        let ans = evaluate(
            token_claims,
            PROJECTLEAD_PROJECT_POLICY,
            Action::ViewProject,
        );
        assert!(ans.is_ok(), "ans is ok");

        match ans.unwrap() {
            ResourceAuthorizationResult::Allow => assert!(false),
            ResourceAuthorizationResult::Deny => assert!(false),
            ResourceAuthorizationResult::Residual(v) => {
                assert_eq!(v.len(), 1);
                let id = v[0].annotation("id");
                assert_eq!(id, Some("ProjectLeadPolicy.Project"));
                assert_eq!(v[0].condition(), Some(expected));
            }
        }
    }

    #[test]
    fn all_policies_is_residual() {
        let token_claims = TokenClaims {
            id: 1,
            roles: vec!["ProjectLead".to_string(), "Developer".to_string()],
        };

        let all_policies = format!(
            "{}\n{}\n{}\n{}\n",
            ADMIN_POLICY, PROJECTLEAD_PROJECT_POLICY, PROJECTLEAD_POLICY, DEVELOPER_POLICY
        );

        let ans = evaluate(token_claims, &all_policies, Action::ViewProject);
        match ans {
            Err(_) => assert!(false),
            Ok(ResourceAuthorizationResult::Allow) => assert!(false),
            Ok(ResourceAuthorizationResult::Deny) => assert!(false),
            Ok(ResourceAuthorizationResult::Residual(v)) => {
                let mut policies: Vec<String> = v
                    .iter()
                    .filter_map(|p| p.annotation("id").map(|id| id.to_string()))
                    .collect();
                policies.sort();
                assert_eq!(
                    policies,
                    vec!("DeveloperPolicy", "ProjectLeadPolicy.Project",)
                );
            }
        }
    }
}
