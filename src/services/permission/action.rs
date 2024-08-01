use std::str::FromStr;
use cedar_policy::{EntityId, EntityTypeName, EntityUid};

pub enum Action {
    ViewProject
}

impl From<Action> for EntityUid {
    fn from(action: Action) -> Self {
        let action_str = match action {
            Action::ViewProject => "ViewProject",
            // Add other variants here as needed
        };

        let a_eid = EntityId::from_str(action_str).unwrap();
        let a_name = EntityTypeName::from_str("Action").unwrap();
        EntityUid::from_type_name_and_id(a_name, a_eid)
    }
}
