CREATE TABLE streaming_info (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
beamline_id integer UNIQUE NOT NULL REFERENCES beamlines (id),
streaming_cache_path VARCHAR (400) NOT NULL
);

INSERT INTO streaming_info (beamline_id, streaming_cache_path) VALUES
((SELECT id FROM beamlines WHERE acronym = '2-ID-D'), '/local/cache/2idd'),
((SELECT id FROM beamlines WHERE acronym = '2-ID-E'), '/local/cache/2ide'),
((SELECT id FROM beamlines WHERE acronym = 'BNP'), '/local/cache/bnp'),
((SELECT id FROM beamlines WHERE acronym = '8-BM-B'), '/local/cache/8bmb');
