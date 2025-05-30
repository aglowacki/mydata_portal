CREATE TABLE beamline_contacts (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
user_badge integer REFERENCES users (badge),
beamline_id integer REFERENCES beamlines (id)
);
