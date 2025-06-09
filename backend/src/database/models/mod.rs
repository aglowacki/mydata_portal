
#![allow(unused)]
#![allow(clippy::all)]


use chrono::NaiveDateTime;
use chrono::DateTime;
use chrono::offset::Utc;
use diesel::{Queryable, Identifiable, Selectable, QueryableByName, Associations};
use serde::Serialize;
use crate::database::schema::{beamline_contacts,
                                beamlines,
                                data_analysis,
                                datasets,
                                experiment_roles,
                                experimenter_proposal_links,
                                proposal_dataset_links,
                                proposals,
                                scan_types,
                                syncotron_runs,
                                user_access_controls,
                                users
                                };


#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ExperimentRole {
    pub id: i32,
    pub role: String,
}


#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(id))]
pub struct ScanType {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(id))]
pub struct SyncotronRun {
    pub id: i32,
    pub name: String,
    pub start_timestamp: DateTime<Utc>,
    pub end_timestamp: DateTime<Utc>,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(id))]
pub struct UserAccessControl {
    pub id: i32,
    pub level: String,
    pub description: String,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(id))]
pub struct Beamline {
    pub id: i32,
    pub name: String,
    pub acronym: String,
    pub old_acronym: Option<String>,
    pub division: String,
    pub link: String,
}

#[derive(Queryable, Debug, Identifiable, Selectable, Serialize, Clone)]
#[diesel(primary_key(id))]
pub struct Proposal {
    pub id: i32,
    pub title: String,
    pub proprietaryflag: String,
    pub mailinflag: String,
    pub status: Option<String>,
}

#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName, Associations, serde::Serialize)]
#[diesel(belongs_to(UserAccessControl))]
#[diesel(primary_key(badge))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub badge: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub institution: String,
    pub email: String,
    pub user_access_control_id: i32,
}

#[derive(Queryable, Debug, Identifiable, Associations)]
#[diesel(belongs_to(Beamline), belongs_to(User, foreign_key=user_badge))]
pub struct BeamlineContact {
    pub user_badge: Option<i32>,
    pub beamline_id: Option<i32>,
    pub id: i32,
}

#[derive(Queryable, Debug, Identifiable, Associations, Selectable, QueryableByName, serde::Serialize)]
#[diesel(belongs_to(ScanType), belongs_to(SyncotronRun), belongs_to(Beamline))]
#[diesel(primary_key(id))]
pub struct Dataset {
    pub id: i32,
    pub path: String,
    pub acquisition_timestamp: NaiveDateTime,
    pub beamline_id: i32,
    pub syncotron_run_id: i32,
    pub scan_type_id: i32,
}

#[derive(Queryable, Debug, Identifiable, Associations)]
#[diesel(belongs_to(Dataset))]
#[diesel(primary_key(id))]
pub struct DataAnalysi {
    pub id: i32,
    pub path: String,
    pub dataset_id: Option<i32>,
    pub analysis_submit_time: NaiveDateTime,
    pub processing_start_time: Option<NaiveDateTime>,
    pub processing_end_time: Option<NaiveDateTime>,
}

#[derive(Queryable, Debug, Associations, Identifiable, Selectable, QueryableByName)]
#[diesel(belongs_to(User, foreign_key=user_badge), belongs_to(Proposal), belongs_to(ExperimentRole))]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ExperimenterProposalLink { 
    pub user_badge: i32,
    pub proposal_id: i32,
    pub experiment_role_id: i32,
    pub id: i32,
}


#[derive(Queryable, Debug, Associations, Identifiable, Selectable, QueryableByName)]
#[diesel(belongs_to(Dataset), belongs_to(Proposal))]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProposalDatasetLink {
    pub dataset_id: i32,
    pub proposal_id: i32,
    pub id: i32,
}

// ------------------------- joined -------------------------

#[derive(serde::Serialize)]
pub struct ProposalWithDatasets
{
    #[serde(flatten)]
    pub proposal: Proposal,
    pub datasets: Vec<Dataset>,
}
