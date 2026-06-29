CREATE TABLE bio_sample_dataset_links (
dataset_id integer PRIMARY KEY REFERENCES datasets (id),
bio_sample_id integer NOT NULL REFERENCES bio_samples (id)
);
