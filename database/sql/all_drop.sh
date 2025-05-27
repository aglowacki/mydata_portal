#!/bin/sh

psql -d mydata -f 900_drop_analysis.sql 
psql -d mydata -f 900_drop_experimenters.sql 

psql -d mydata -f 901_drop_beamline_contacts.sql 
psql -d mydata -f 901_drop_datasets.sql 

psql -d mydata -f 902_drop_users.sql 

psql -d mydata -f 999_drop_scan_type.sql 
psql -d mydata -f 999_drop_syncotron_runs.sql 
psql -d mydata -f 999_drop_experiment_roles.sql 
psql -d mydata -f 999_drop_beamlines.sql          
psql -d mydata -f 999_drop_proposals.sql    
psql -d mydata -f 999_drop_user_access_control.sql
