#!/bin/sh

psql -d mydata -f 100_insert_user_access_control.sql 
psql -d mydata -f 101_insert_beamlines.sql 
psql -d mydata -f 102_insert_users_staff.sql
psql -d mydata -f 103_insert_beamline_contacts.sql
psql -d mydata -f 104_insert_experiment_roles.sql
psql -d mydata -f 105_insert_scan_types.sql
