#!/bin/sh

psql -d mydata -f 000_create_user_types.sql
psql -d mydata -f 001_create_beamlines.sql             
psql -d mydata -f 002_create_users.sql 
psql -d mydata -f 003_create_beamline_contacts.sql 
psql -d mydata -f 004_create_syncotron_runs.sql 
psql -d mydata -f 005_create_scan_type.sql 
psql -d mydata -f 006_create_datastore.sql 
psql -d mydata -f 007_create_datasets.sql 
psql -d mydata -f 008_create_experiment_roles.sql 
psql -d mydata -f 009_create_dataset_user_relations.sql 
psql -d mydata -f 010_create_analysis.sql 
