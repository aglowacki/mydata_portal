-- Migration for an already-populated database. Adds the bio-sample hydration
-- state (Dehydrated / Frozen hydrated), the nullable bio_samples column that
-- references it, its FK index, and the trigger that requires it for fixed
-- samples. Fresh builds get all of this from 000/003/005/007; this file is only
-- for the existing mydata database.
--
-- The new column is nullable, so existing rows need no backfill. The trigger
-- only affects future inserts/updates.

BEGIN;

CREATE TABLE bio_sample_hydration_states (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
name varchar(255) UNIQUE NOT NULL
);

INSERT INTO bio_sample_hydration_states (name) VALUES
('Dehydrated'),
('Frozen hydrated');

ALTER TABLE bio_samples
  ADD COLUMN hydration_state_id integer REFERENCES bio_sample_hydration_states (id);

CREATE INDEX bio_samples_hydration_state_id_idx ON bio_samples (hydration_state_id);

CREATE OR REPLACE FUNCTION bio_samples_require_hydration_when_fixed()
RETURNS trigger AS $$
BEGIN
  IF NEW.hydration_state_id IS NULL
     AND EXISTS (SELECT 1 FROM bio_sample_fixations f
                 WHERE f.id = NEW.fixation_id AND f.name <> 'None') THEN
    RAISE EXCEPTION 'hydration_state_id is required when the sample fixation is not None';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER bio_samples_hydration_check
BEFORE INSERT OR UPDATE ON bio_samples
FOR EACH ROW EXECUTE FUNCTION bio_samples_require_hydration_when_fixed();

COMMIT;
