CREATE TABLE data_analysis (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
dataset_id integer REFERENCES datasets (id),
data_store_id integer REFERENCES data_store (id),
analysis_submit_time TIMESTAMP NOT NULL,
processing_start_time TIMESTAMP,
processing_end_time TIMESTAMP
);
