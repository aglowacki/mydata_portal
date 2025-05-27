#!/bin/sh

psql -d mydata -f 000_create_beamlines.sql
psql -d mydata -f 000_create_experiment_roles.sql
psql -d mydata -f 000_create_proposals.sql
psql -d mydata -f 000_create_scan_type.sql
psql -d mydata -f 000_create_syncotron_runs.sql
psql -d mydata -f 000_create_user_access_control.sql

psql -d mydata -f 001_create_users.sql

psql -d mydata -f 002_create_beamline_contacts.sql
psql -d mydata -f 002_create_datasets.sql

psql -d mydata -f 003_create_analysis.sql
psql -d mydata -f 003_create_experimenters.sql

