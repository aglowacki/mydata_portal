CREATE TABLE bio_sample_type_origin_sub_origin_links (
id INT NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
bio_sample_type_id integer NOT NULL REFERENCES bio_sample_types (id), 
origin_id integer NOT NULL REFERENCES sample_origins (id),
sub_origin_id integer NOT NULL REFERENCES sample_sub_origins (id)
);
