-- Step 1: Create the "Administrator" role type if it doesn't already exist
INSERT INTO role_type (role_type_id, name, created_by)
VALUES (2, 'Administrator', 1)  -- Assuming 1 is the party_role_id of "System Application"
-- ON CONFLICT (name) DO NOTHING
RETURNING role_type_id;

-- -- Step 2: Create the "System Administrator" party if it doesn't already exist
INSERT INTO parties (party_id, first_name, last_name, created_by)
VALUES (2, 'System', 'Administrator', 1);  
-- -- ON CONFLICT (first_name, last_name) DO NOTHING


-- -- -- Step 3: Create the party_role for "System Administrator" with "Administrator" role if it doesn't already exist

INSERT INTO party_role (party_role_id, party_id, role_type_id, created_by)
VALUES (2, 2, 2, 1);

-- -- -- Step 4: Create the "Project Lead" role type
INSERT INTO role_type (role_type_id, name, created_by)
VALUES (3, 'Project Lead', 1);

INSERT INTO parties (party_id, first_name, last_name, created_by)
VALUES (3, 'Jane', 'Doe', 1);  

-- -- Step 5: Assign the "Project Lead" role to the "System Administrator"
-- -- First, we need to get the party_id of the "System Administrator"
INSERT INTO party_role (party_role_id, party_id, role_type_id, created_by)
VALUES (3, 3, 3, 1);

INSERT INTO projects (id, name, description, owned_by, created_by, updated_by)
VALUES (1, 'my project', 'this project', 3, 1, 1);

INSERT INTO projects (id, name, description, owned_by, created_by, updated_by)
VALUES (2, 'my other project', 'that project', 3, 1, 1);

-- Step 6: Create the "Developer" role type
INSERT INTO role_type (role_type_id, name, created_by)
VALUES (4, 'Developer', 1);
-- Assuming 1 is the party_role_id of "System Application"

INSERT INTO parties (party_id, first_name, last_name, created_by)
VALUES (4, 'John', 'Smith', 1);  

INSERT INTO party_role (party_role_id, party_id, role_type_id, created_by)
VALUES (4, 4, 4, 1);


INSERT INTO assignments (party_role_id, project_id, created_by)
VALUES (4, 2, 1);

-- -- Step 7: Assign the "Developer" role to the "System Administrator"
-- -- First, we need to get the party_id of the "System Administrator"

-- INSERT INTO party_role (party_role_id, party_id, role_type_id, created_by)
-- VALUES (4, 2, 4, 1);
-- -- Step 8: Verify the insertions and assignments
-- SELECT * FROM role_type WHERE name IN ('Administrator', 'Project Lead', 'Developer');

-- SELECT p.first_name, p.last_name, rt.name AS role_name
-- FROM parties p
-- JOIN party_role pr ON p.party_id = pr.party_id
-- JOIN role_type rt ON pr.role_type_id = rt.role_type_id
-- WHERE p.first_name = 'System' AND p.last_name = 'Administrator'
-- ORDER BY rt.name;
