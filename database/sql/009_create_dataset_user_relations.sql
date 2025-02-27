CREATE TABLE "DatasetUserRelation" (
datasest_Id integer REFERENCES Datasets (id),
user_id integer REFERENCES Users (id),
experiment_role integer REFERENCES ExperimentRoles (id)
);
