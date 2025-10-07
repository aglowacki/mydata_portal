CREATE TABLE bio_samples (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
proposal_id integer NOT NULL REFERENCES proposals (id),
name varchar(1000) NOT NULL,
type_id integer NOT NULL REFERENCES bio_sample_types (id),
origin_id integer NOT NULL REFERENCES sample_origins (id),
sub_origin_id integer REFERENCES sample_sub_origins (id),
source_id integer NOT NULL REFERENCES sample_sources (id),
thickness integer,
cell_line varchar(256),
is_cancer bool,
condition_id integer NOT NULL REFERENCES bio_sample_conditions (id),
treatment_details varchar(2000),
fixation_id integer NOT NULL REFERENCES bio_sample_fixations (id),
expected_elemental_content_change varchar(2000),
notes varchar(3000)
);
