CREATE TABLE proposal_dataset_links (
id INT NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
dataset_id integer NOT NULL REFERENCES datasets (id),
proposal_id integer NOT NULL REFERENCES proposals (id)
);
