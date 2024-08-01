-- Step 1: Create the "Administrator" role type if it doesn't already exist
INSERT INTO role_type (name, created_by)
VALUES ('Administrator', 1)  -- Assuming 1 is the party_role_id of "System Application"
-- ON CONFLICT (name) DO NOTHING
RETURNING role_type_id;

-- Step 2: Create the "System Administrator" party if it doesn't already exist
INSERT INTO parties (first_name, last_name, created_by)
VALUES ('System', 'Administrator', 1)  
-- ON CONFLICT (first_name, last_name) DO NOTHING
RETURNING party_id;

-- Step 3: Create the party_role for "System Administrator" with "Administrator" role if it doesn't already exist
WITH admin_party AS (
    SELECT party_id 
    FROM parties 
    WHERE first_name = 'System' AND last_name = 'Administrator'
),
admin_role AS (
    SELECT role_type_id 
    FROM role_type 
    WHERE name = 'Administrator'
)
INSERT INTO party_role (party_id, role_type_id, created_by)
SELECT admin_party.party_id, admin_role.role_type_id, 1 
FROM admin_party, admin_role
-- ON CONFLICT (party_id, role_type_id) DO NOTHING
RETURNING party_role_id;

-- Step 4: Create the "Project Lead" role type
INSERT INTO role_type (name, created_by)
VALUES ('Project Lead', 1) 
RETURNING role_type_id;

-- Step 5: Assign the "Project Lead" role to the "System Administrator"
-- First, we need to get the party_id of the "System Administrator"
WITH admin_party AS (
    SELECT party_id 
    FROM parties 
    WHERE first_name = 'System' AND last_name = 'Administrator'
)
INSERT INTO party_role (party_id, role_type_id, created_by)
SELECT admin_party.party_id, 3, 1 
FROM admin_party
RETURNING party_role_id;

-- Step 6: Create the "Developer" role type
INSERT INTO role_type (name, created_by)
VALUES ('Developer', 1)  -- Assuming 1 is the party_role_id of "System Application"
RETURNING role_type_id;

-- Step 7: Assign the "Developer" role to the "System Administrator"
-- First, we need to get the party_id of the "System Administrator"
WITH admin_party AS (
    SELECT party_id 
    FROM parties 
    WHERE first_name = 'System' AND last_name = 'Administrator'
)
INSERT INTO party_role (party_id, role_type_id, created_by)
SELECT admin_party.party_id, 4, 1  -- 4 is the role_type_id for "Developer", 1 is the party_role_id of "System Application"
FROM admin_party
RETURNING party_role_id;

-- Step 8: Verify the insertions and assignments
SELECT * FROM role_type WHERE name IN ('Administrator', 'Project Lead', 'Developer');

SELECT p.first_name, p.last_name, rt.name AS role_name
FROM parties p
JOIN party_role pr ON p.party_id = pr.party_id
JOIN role_type rt ON pr.role_type_id = rt.role_type_id
WHERE p.first_name = 'System' AND p.last_name = 'Administrator'
ORDER BY rt.name;
