-- Postgres does not automatically index foreign key columns (only primary
-- keys and unique constraints get an index for free). These indexes back
-- the joins/filters the app does on child tables, and speed up the
-- referential-integrity check Postgres runs against children when a
-- referenced parent row is deleted.
--
-- Columns that are already the leading column of a primary key or unique
-- constraint (e.g. bio_sample_dataset_links.dataset_id,
-- proposal_dataset_links.dataset_id) are already indexed and are skipped
-- here.

CREATE INDEX users_user_access_control_id_idx ON users (user_access_control_id);

CREATE INDEX bio_sample_fixations_fixative_id_idx ON bio_sample_fixations (fixative_id);

CREATE INDEX beamline_contacts_user_badge_idx ON beamline_contacts (user_badge);
CREATE INDEX beamline_contacts_beamline_id_idx ON beamline_contacts (beamline_id);

CREATE INDEX datasets_beamline_id_idx ON datasets (beamline_id);
CREATE INDEX datasets_syncotron_run_id_idx ON datasets (syncotron_run_id);
CREATE INDEX datasets_scan_type_id_idx ON datasets (scan_type_id);

CREATE INDEX bio_sample_type_origin_sub_origin_links_type_id_idx ON bio_sample_type_origin_sub_origin_links (bio_sample_type_id);
CREATE INDEX bio_sample_type_origin_sub_origin_links_origin_id_idx ON bio_sample_type_origin_sub_origin_links (origin_id);
CREATE INDEX bio_sample_type_origin_sub_origin_links_sub_origin_id_idx ON bio_sample_type_origin_sub_origin_links (sub_origin_id);

CREATE INDEX data_analysis_dataset_id_idx ON data_analysis (dataset_id);

CREATE INDEX experimenter_proposal_links_user_badge_idx ON experimenter_proposal_links (user_badge);
CREATE INDEX experimenter_proposal_links_proposal_id_idx ON experimenter_proposal_links (proposal_id);
CREATE INDEX experimenter_proposal_links_experiment_role_id_idx ON experimenter_proposal_links (experiment_role_id);

CREATE INDEX proposal_dataset_links_proposal_id_idx ON proposal_dataset_links (proposal_id);

CREATE INDEX bio_samples_proposal_id_idx ON bio_samples (proposal_id);
CREATE INDEX bio_samples_type_id_idx ON bio_samples (type_id);
CREATE INDEX bio_samples_origin_id_idx ON bio_samples (origin_id);
CREATE INDEX bio_samples_sub_origin_id_idx ON bio_samples (sub_origin_id);
CREATE INDEX bio_samples_source_id_idx ON bio_samples (source_id);
CREATE INDEX bio_samples_condition_id_idx ON bio_samples (condition_id);
CREATE INDEX bio_samples_fixation_id_idx ON bio_samples (fixation_id);

CREATE INDEX bio_sample_dataset_links_bio_sample_id_idx ON bio_sample_dataset_links (bio_sample_id);
