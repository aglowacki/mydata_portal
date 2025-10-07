#!/bin/sh

psql -d mydata -f 100_insert_bio_sample_conditions.sql
psql -d mydata -f 100_insert_bio_sample_fixatives.sql
psql -d mydata -f 100_insert_bio_sample_types.sql
psql -d mydata -f 100_insert_sample_origins.sql
psql -d mydata -f 100_insert_sample_sources.sql
psql -d mydata -f 100_insert_sample_sub_origins.sql

psql -d mydata -f 101_insert_bio_sample_fixations.sql

psql -d mydata -f 102_insert_bio_sample_type_origin_sub_origin_links.sql