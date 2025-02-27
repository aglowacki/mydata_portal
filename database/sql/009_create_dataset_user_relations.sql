CREATE TABLE "DatasetUserRelation" (
datasest_id integer REFERENCES "Datasets" (id),
user_id integer REFERENCES "Users" (badge),
experiment_role integer REFERENCES "ExperimentRoles" (id)
);
