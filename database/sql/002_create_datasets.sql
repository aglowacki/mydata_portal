CREATE TABLE datasets (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
path varchar(2000) UNIQUE NOT NULL,
acquisition_timestamp TIMESTAMP NOT NULL,
beamline_id integer NOT NULL REFERENCES beamlines (id),
syncotron_run_id integer NOT NULL REFERENCES syncotron_runs (id),
scan_type_id integer NOT NULL REFERENCES scan_type (id)
);
