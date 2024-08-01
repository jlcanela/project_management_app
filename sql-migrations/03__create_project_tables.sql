-- Step 1: Create the Tables without created_by and updated_by Fields
CREATE TABLE parties (
    party_id SERIAL PRIMARY KEY,
    first_name text,
    last_name text,
    created_at timestamp DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE role_type (
    role_type_id SERIAL PRIMARY KEY,
    name text UNIQUE,
    created_at timestamp DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE registered_identifier (
    registered_identifier_id SERIAL PRIMARY KEY,
    party_id int references parties(party_id),
    external_id text,
    id_type text,
    id_provider text,
    created_at timestamp DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE party_role (
    party_role_id SERIAL PRIMARY KEY,
    party_id int references parties(party_id),
    role_type_id int references role_type(role_type_id),
    CONSTRAINT party_role_unique UNIQUE(party_id, role_type_id),
    created_at timestamp DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE projects (
    id SERIAL PRIMARY KEY,
    name text,
    description text,
    owned_by int references party_role(party_role_id),
    created_at timestamp DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE assignments (
    party_role_id int references party_role(party_role_id),
    project_id int references projects(id),
    created_at timestamp DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (party_role_id, project_id)
);

-- Step 2: Add the party_role "System Application"
-- Insert the "System Application" party
INSERT INTO parties (party_id, first_name, last_name)
VALUES (1, 'System', 'Application')
RETURNING party_id;

-- Store the returned party_id (let's assume it's 1 for this script)

-- Insert the "system" role type
INSERT INTO role_type (role_type_id, name)
VALUES (1, 'System')
RETURNING role_type_id;

-- Store the returned role_type_id (let's assume it's 1 for this script)

-- Create the "System Application" party_role
INSERT INTO party_role (party_role_id, party_id, role_type_id)
VALUES (1, 1, 1)
RETURNING party_role_id;

-- Store the returned party_role_id (let's assume it's 1 for this script)

-- -- Step 3: Add the created_by and updated_by Fields
ALTER TABLE parties
ADD COLUMN created_by int references party_role(party_role_id);

ALTER TABLE role_type
ADD COLUMN created_by int references party_role(party_role_id);

ALTER TABLE registered_identifier
ADD COLUMN created_by int references party_role(party_role_id);

ALTER TABLE party_role
ADD COLUMN created_by int references party_role(party_role_id);

ALTER TABLE projects
ADD COLUMN created_by int references party_role(party_role_id);

ALTER TABLE projects
ADD COLUMN updated_by int references party_role(party_role_id);

ALTER TABLE assignments
ADD COLUMN created_by int references party_role(party_role_id);

-- -- Step 4: Update the Missing Values
-- -- Update the created_by fields with the "System Application" party_role_id
UPDATE parties
SET created_by = 1;

UPDATE role_type
SET created_by = 1;

UPDATE registered_identifier
SET created_by = 1;

UPDATE party_role
SET created_by = 1;

UPDATE projects
SET created_by = 1,
    updated_by = 1;

UPDATE assignments
SET created_by = 1;

-- -- Step 5: Add the Remaining Constraints
-- ALTER TABLE parties
-- ALTER COLUMN created_by SET NOT NULL;

-- ALTER TABLE role_type
-- ALTER COLUMN created_by SET NOT NULL;

-- ALTER TABLE registered_identifier
-- ALTER COLUMN created_by SET NOT NULL;

-- ALTER TABLE party_role
-- ALTER COLUMN created_by SET NOT NULL;

-- ALTER TABLE projects
-- ALTER COLUMN created_by SET NOT NULL;

-- ALTER TABLE projects
-- ALTER COLUMN updated_by SET NOT NULL;

-- ALTER TABLE assignments
-- ALTER COLUMN created_by SET NOT NULL;
