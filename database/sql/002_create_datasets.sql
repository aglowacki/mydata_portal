CREATE TABLE datasets (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
beamline_id integer REFERENCES beamlines (id),
syncotron_run_id integer REFERENCES syncotron_runs (id),
scan_type_id integer REFERENCES scan_type (id),
data_store_id integer REFERENCES data_store (id),
acquisition_timestamp TIMESTAMP NOT NULL
);
