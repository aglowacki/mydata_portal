CREATE TABLE sample_origin_tissue_source_links (
id INT NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
origin_id integer NOT NULL REFERENCES sample_origins (id),
tissue_source_id integer NOT NULL REFERENCES tissue_sources (id),
UNIQUE (origin_id, tissue_source_id)
);
