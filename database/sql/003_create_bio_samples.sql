CREATE TABLE bio_samples (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
proposal_id integer NOT NULL REFERENCES proposals (id),
name varchar(1000) NOT NULL,
type_id integer NOT NULL REFERENCES bio_sample_types (id),
origin_id integer NOT NULL REFERENCES sample_origins (id),
sub_origin_id integer REFERENCES sample_sub_origins (id),
tissue_source_id integer REFERENCES tissue_sources (id),
thickness integer,
cell_line varchar(256),
is_cancer bool,
condition_id integer NOT NULL REFERENCES bio_sample_conditions (id),
treatment_details varchar(2000),
fixation_id integer NOT NULL REFERENCES bio_sample_fixations (id),
-- Hydration state (Dehydrated / Frozen hydrated). Nullable: only required when
-- the sample is fixed (fixation is not 'None'), enforced by the
-- bio_samples_hydration_check trigger (007_create_bio_sample_hydration_trigger.sql).
hydration_state_id integer REFERENCES bio_sample_hydration_states (id),
expected_elemental_content_change varchar(2000),
notes varchar(3000),
-- Enforce that the (type, origin, sub_origin) triple is one the
-- bio_sample_type_origin_sub_origin_links table declares valid, rather than
-- letting the three FKs be set independently. sub_origin_id is nullable; under
-- the default MATCH SIMPLE semantics this composite FK is skipped when
-- sub_origin_id IS NULL, so samples without a sub-origin are still allowed.
CONSTRAINT bio_samples_type_origin_sub_origin_fkey
  FOREIGN KEY (type_id, origin_id, sub_origin_id)
  REFERENCES bio_sample_type_origin_sub_origin_links (bio_sample_type_id, origin_id, sub_origin_id),
-- Enforce that the (origin, tissue_source) pair is one the
-- sample_origin_tissue_source_links table declares valid. tissue_source_id is
-- nullable (e.g. plants/fungi/bacteria have no tissue source); under MATCH
-- SIMPLE this composite FK is skipped when tissue_source_id IS NULL.
CONSTRAINT bio_samples_origin_tissue_source_fkey
  FOREIGN KEY (origin_id, tissue_source_id)
  REFERENCES sample_origin_tissue_source_links (origin_id, tissue_source_id)
);
