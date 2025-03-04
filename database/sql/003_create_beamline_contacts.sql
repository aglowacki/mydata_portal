CREATE TABLE beamline_contacts (
user_badge integer REFERENCES users (badge),
beamline_id integer REFERENCES beamlines (id)
);
