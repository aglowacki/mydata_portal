CREATE TABLE experimenters (
datasest_id integer REFERENCES datasets (id),
user_badge integer REFERENCES users (badge),
experiment_role_id integer REFERENCES experiment_roles (id)
);
