CREATE TABLE experimenter_proposal_links (
id INT NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
user_badge integer NOT NULL REFERENCES users (badge),
proposal_id integer NOT NULL REFERENCES proposals (id),
experiment_role_id integer NOT NULL REFERENCES experiment_roles (id)
);
