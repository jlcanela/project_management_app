//Admin
@id("AdminPolicy")
permit(
  principal in Role::"Administrator",
  action in [Action::"CreateParty", Action::"CreateRole", Action::"AssignRole", Action::"RemoveRole"], 
  resource
);

// ProjectLead can CreateProject and ListProject
@id("ProjectLeadPolicy")
permit(
  principal in Role::"ProjectLead",
  action in [Action::"CreateProject", Action::"ListProject"],
  resource
);

// ProjectLead can ManageProject
@id("ProjectLeadPolicy.Project")
permit(
  principal in Role::"ProjectLead",
  action in [Action::"ViewProject", Action::"AuditProject", Action::"AssignPartyrole", Action::"ShareProject"],
  resource
)
when { principal == resource.owner };
// when project is owned 


// Developer can ViewProject
@id("DeveloperPolicy")
permit(
  principal in Role::"Developer",
  action in [Action::"ListProject", Action::"ViewProject"],
  resource
)
when { principal in resource.assigned_to };
// when assigned to project

//Anybody
//-Principal: user_id
//-Action: ListProject + ViewProject
//-Resource: project_id
//When sharing the project
// permit(
//   principal in Role::"User",
//   action in [Action::"ListProject", Action::"ViewProject"], 
//   resource in Group::"AllProjects"
// );

