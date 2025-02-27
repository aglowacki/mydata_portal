CREATE TABLE "Datasets" (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
beamline_id integer REFERENCES "Beamlines" (id),
run_id integer REFERENCES "SyncotronRuns" (id),
scan_type_id integer REFERENCES "ScanType" (id),
datastore_id integer REFERENCES "DataStore" (id),
AcquisitionDateTime TIMESTAMP NOT NULL
);
