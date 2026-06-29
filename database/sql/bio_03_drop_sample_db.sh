#!/bin/sh

psql -d mydata -f 900_drop_bio_sample_dataset_links.sql
psql -d mydata -f 900_drop_bio_samples.sql
