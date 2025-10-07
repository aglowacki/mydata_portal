#!/bin/sh

psql -d mydata -f 000_create_beamlines.sql
psql -d mydata -f 000_create_experiment_roles.sql
psql -d mydata -f 000_create_proposals.sql
psql -d mydata -f 000_create_scan_type.sql
psql -d mydata -f 000_create_syncotron_runs.sql
psql -d mydata -f 000_create_user_access_control.sql
psql -d mydata -f 000_create_bio_sample_conditions.sql
psql -d mydata -f 000_create_bio_sample_fixatives.sql
psql -d mydata -f 000_create_bio_sample_types.sql
psql -d mydata -f 000_create_sample_origins.sql
psql -d mydata -f 000_create_sample_sources.sql
psql -d mydata -f 000_create_sample_sub_origins.sql

psql -d mydata -f 001_create_users.sql
psql -d mydata -f 001_create_bio_sample_fixations.sql

psql -d mydata -f 002_create_beamline_contacts.sql
psql -d mydata -f 002_create_datasets.sql
psql -d mydata -f 002_create_bio_sample_type_origin_sub_origin_links.sql

psql -d mydata -f 003_create_analysis.sql
psql -d mydata -f 003_create_experimenter_proposal_links.sql
psql -d mydata -f 003_create_proposal_dataset_links.sql
psql -d mydata -f 003_create_bio_samples.sql