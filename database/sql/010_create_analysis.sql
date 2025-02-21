CREATE TABLE Analysis (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
dataset_id integer REFERENCES Datasets (id),
datastore_id REFERENCES DataStore (id),
AnalysisSubmitTime DATETIME NOT NULL,
ProcessingStartTime DATETIME,
ProcessingEndTime DATETIME
);
