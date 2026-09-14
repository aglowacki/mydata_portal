-- Migration for an already-populated database. Adds audit columns to
-- bio_samples recording who created the row and who last edited it (badge ->
-- users.badge), plus timestamps. Fresh builds get these inline from
-- 003_create_bio_samples.sql; this file is only for the existing mydata database.
--
-- created_by / updated_by are added NULLABLE here (unlike the NOT NULL columns a
-- fresh build gets): legacy rows predate attribution and their author is
-- unknown, so there is nothing to backfill. Every write through the
-- upsert_bio_sample handler populates them going forward. The *_at columns
-- default to now(), so existing rows get a sensible (migration-time) value.

BEGIN;

ALTER TABLE bio_samples
  ADD COLUMN created_by integer REFERENCES users (badge),
  ADD COLUMN created_at timestamptz NOT NULL DEFAULT now(),
  ADD COLUMN updated_by integer REFERENCES users (badge),
  ADD COLUMN updated_at timestamptz NOT NULL DEFAULT now();

-- Match 005_create_foreign_key_indexes.sql: index the new FK columns.
CREATE INDEX bio_samples_created_by_idx ON bio_samples (created_by);
CREATE INDEX bio_samples_updated_by_idx ON bio_samples (updated_by);

COMMIT;
