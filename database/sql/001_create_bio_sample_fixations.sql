CREATE TABLE bio_sample_fixations (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
name varchar(512) NOT NULL,
fixative_id integer NOT NULL  REFERENCES bio_sample_fixatives (id)
);
