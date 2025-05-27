CREATE TABLE datasets (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
path varchar(2000) NOT NULL,
acquisition_timestamp TIMESTAMP NOT NULL,
beamline_id integer REFERENCES beamlines (id),
syncotron_run_id integer REFERENCES syncotron_runs (id),
scan_type_id integer REFERENCES scan_type (id)
);
