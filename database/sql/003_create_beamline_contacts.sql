CREATE TABLE "BeamlineContacts" (
user_id integer REFERENCES "Users" (badge),
beamline_id integer REFERENCES "Beamlines" (id)
);
