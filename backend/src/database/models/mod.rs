
#![allow(unused)]
#![allow(clippy::all)]


use chrono::NaiveDateTime;
use chrono::DateTime;
use chrono::offset::Utc;
use diesel::{Queryable, Identifiable, Selectable};
use serde::Serialize;
use crate::database::schema::{beamline_contacts,
                                beamlines,
                                data_analysis,
                                datasets,
                                experiment_roles,
                                experimenters,
                                proposals,
                                scan_types,
                                syncotron_runs,
                                user_access_controls,
                                users
                                };

#[derive(Queryable, Debug, Identifiable)]
pub struct BeamlineContact {
    pub user_badge: Option<i32>,
    pub beamline_id: Option<i32>,
    pub id: i32,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct Beamline {
    pub id: i32,
    pub name: String,
    pub acronym: String,
    pub old_acronym: Option<String>,
    pub division: String,
    pub link: String,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct DataAnalysi {
    pub id: i32,
    pub path: String,
    pub dataset_id: Option<i32>,
    pub analysis_submit_time: NaiveDateTime,
    pub processing_start_time: Option<NaiveDateTime>,
    pub processing_end_time: Option<NaiveDateTime>,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct Dataset {
    pub id: i32,
    pub path: String,
    pub acquisition_timestamp: NaiveDateTime,
    pub beamline_id: Option<i32>,
    pub syncotron_run_id: Option<i32>,
    pub scan_type_id: Option<i32>,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct ExperimentRole {
    pub id: i32,
    pub role: String,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct Experimenter {
    pub dataset_id: Option<i32>,
    pub user_badge: Option<i32>,
    pub proposal_id: Option<i32>,
    pub experiment_role_id: Option<i32>,
    pub id: i32,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct Proposal {
    pub id: i32,
    pub title: String,
    pub proprietaryflag: String,
    pub mailinflag: String,
    pub status: Option<String>,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct ScanType {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct SyncotronRun {
    pub id: i32,
    pub name: String,
    pub start_timestamp: DateTime<Utc>,
    pub end_timestamp: DateTime<Utc>,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct UserAccessControl {
    pub id: i32,
    pub level: String,
    pub description: String,
}

#[derive(Queryable, Debug, Identifiable, Selectable, serde::Serialize)]
#[diesel(primary_key(badge))]
pub struct User {
    pub badge: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub institution: String,
    pub email: String,
    pub user_access_control_id: Option<i32>,
}

