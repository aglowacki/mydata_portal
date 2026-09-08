-- DB backstop for the rule "a hydration state is required when the sample is
-- fixed". A CHECK constraint cannot look up the fixation name in another table,
-- so this is enforced with a BEFORE INSERT/UPDATE trigger. The app layer
-- (frontend form + upsert_bio_sample) validates the same rule and produces the
-- user-facing message; this trigger guarantees the invariant for any other
-- write path. "Fixed" means the referenced bio_sample_fixations row is not the
-- single 'None' row.

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
