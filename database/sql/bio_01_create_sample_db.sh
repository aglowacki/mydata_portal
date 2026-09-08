#!/bin/sh

psql -d mydata -f 000_create_bio_sample_conditions.sql
psql -d mydata -f 000_create_bio_sample_fixatives.sql
psql -d mydata -f 000_create_bio_sample_types.sql
psql -d mydata -f 000_create_sample_origins.sql
psql -d mydata -f 000_create_tissue_sources.sql
psql -d mydata -f 000_create_sample_sub_origins.sql
psql -d mydata -f 000_create_bio_sample_hydration_states.sql

psql -d mydata -f 001_create_bio_sample_fixations.sql

psql -d mydata -f 002_create_bio_sample_type_origin_sub_origin_links.sql
psql -d mydata -f 002_create_sample_origin_tissue_source_links.sql

psql -d mydata -f 003_create_bio_samples.sql

psql -d mydata -f 004_create_bio_sample_dataset_links.sql

psql -d mydata -f 007_create_bio_sample_hydration_trigger.sql