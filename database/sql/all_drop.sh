#!/bin/sh

psql -d mydata -f 900_drop_analysis.sql 
psql -d mydata -f 901_drop_dataset_user_relations.sql 
psql -d mydata -f 902_drop_experiment_roles.sql 
psql -d mydata -f 903_drop_datasets.sql 
psql -d mydata -f 904_drop_datastore.sql 
psql -d mydata -f 905_drop_scan_type.sql 
psql -d mydata -f 906_drop_syncotron_runs.sql 
psql -d mydata -f 907_drop_beamline_contacts.sql 
psql -d mydata -f 908_drop_users.sql 
psql -d mydata -f 909_drop_beamlines.sql             
psql -d mydata -f 910_drop_user_types.sql

