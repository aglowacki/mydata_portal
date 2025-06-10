CREATE TABLE beamline_contacts (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
user_badge integer NOT NULL REFERENCES users (badge),
beamline_id integer NOT NULL REFERENCES beamlines (id)
);
