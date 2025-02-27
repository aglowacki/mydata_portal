CREATE TABLE "Analysis" (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
dataset_id integer REFERENCES Datasets (id),
datastore_id REFERENCES DataStore (id),
AnalysisSubmitTime TIMESTAMP NOT NULL,
ProcessingStartTime TIMESTAMP,
ProcessingEndTime TIMESTAMP
);
