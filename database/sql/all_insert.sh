#!/bin/sh

psql -d mydata -f 100_insert_user_access_control.sql 
psql -d mydata -f 101_insert_beamlines.sql 
psql -d mydata -f 102_insert_users_staff.sql
psql -d mydata -f 103_insert_beamline_contacts.sql
