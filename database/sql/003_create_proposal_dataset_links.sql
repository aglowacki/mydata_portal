CREATE TABLE proposal_dataset_links (
dataset_id integer NOT NULL REFERENCES datasets (id),
proposal_id integer NOT NULL REFERENCES proposals (id),
PRIMARY KEY(dataset_id, proposal_id)
);
