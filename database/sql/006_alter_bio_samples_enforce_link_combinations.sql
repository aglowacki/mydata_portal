-- Migration for an already-populated database. Brings an existing bio_samples
-- table in line with 003_create_bio_samples.sql, which now enforces that the
-- (type, origin, sub_origin) and (origin, tissue_source) combinations are ones
-- the link tables declare valid, instead of letting the FKs be set
-- independently. Fresh builds get these constraints inline and do not need
-- this file.
--
-- Both composite FKs use the default MATCH SIMPLE semantics, so each is checked
-- only when its nullable column is populated:
--   * sub_origin_id NULL      -> (type, origin, sub_origin) FK skipped
--   * tissue_source_id NULL   -> (origin, tissue_source)   FK skipped

BEGIN;

-- Non-animal origins (plants/fungi/bacteria) have no valid tissue source, so
-- the column must be allowed to be NULL for those samples.
ALTER TABLE bio_samples ALTER COLUMN tissue_source_id DROP NOT NULL;

-- Add the constraints as NOT VALID first: this enforces them for all future
-- inserts/updates immediately while skipping the scan of existing rows, so the
-- migration cannot fail partway on legacy data.
ALTER TABLE bio_samples
  ADD CONSTRAINT bio_samples_type_origin_sub_origin_fkey
  FOREIGN KEY (type_id, origin_id, sub_origin_id)
  REFERENCES bio_sample_type_origin_sub_origin_links (bio_sample_type_id, origin_id, sub_origin_id)
  NOT VALID;

ALTER TABLE bio_samples
  ADD CONSTRAINT bio_samples_origin_tissue_source_fkey
  FOREIGN KEY (origin_id, tissue_source_id)
  REFERENCES sample_origin_tissue_source_links (origin_id, tissue_source_id)
  NOT VALID;

COMMIT;

-- Before validating, find any existing rows that violate the new combinations.
-- These must be corrected (or their tissue_source_id set to NULL) first, or the
-- VALIDATE statements below will error.
--
--   SELECT b.id, b.type_id, b.origin_id, b.sub_origin_id
--   FROM bio_samples b
--   WHERE b.sub_origin_id IS NOT NULL
--     AND NOT EXISTS (
--       SELECT 1 FROM bio_sample_type_origin_sub_origin_links l
--       WHERE l.bio_sample_type_id = b.type_id
--         AND l.origin_id = b.origin_id
--         AND l.sub_origin_id = b.sub_origin_id);
--
--   SELECT b.id, b.origin_id, b.tissue_source_id
--   FROM bio_samples b
--   WHERE b.tissue_source_id IS NOT NULL
--     AND NOT EXISTS (
--       SELECT 1 FROM sample_origin_tissue_source_links l
--       WHERE l.origin_id = b.origin_id
--         AND l.tissue_source_id = b.tissue_source_id);
--
-- Once the above return no rows, validate the existing data:
--
--   ALTER TABLE bio_samples VALIDATE CONSTRAINT bio_samples_type_origin_sub_origin_fkey;
--   ALTER TABLE bio_samples VALIDATE CONSTRAINT bio_samples_origin_tissue_source_fkey;
