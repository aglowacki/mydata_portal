CREATE TABLE experimenter_proposal_links (
id INT NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
user_badge integer REFERENCES users (badge),
proposal_id integer REFERENCES proposals (id),
experiment_role_id integer REFERENCES experiment_roles (id)
);
