CREATE TABLE data_analysis (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
path varchar(2000) UNIQUE NOT NULL,
dataset_id integer NOT NULL REFERENCES datasets (id),
analysis_submit_time TIMESTAMP NOT NULL,
processing_start_time TIMESTAMP,
processing_end_time TIMESTAMP
);
