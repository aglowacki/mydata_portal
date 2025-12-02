
#![allow(unused)]
#![allow(clippy::all)]


use chrono::NaiveDateTime;
use chrono::DateTime;
use chrono::offset::Utc;
use diesel::{Queryable, Identifiable, Selectable, QueryableByName, Associations};
use serde::Serialize;
use crate::database::schema::{beamline_contacts,
                                beamlines,
                                bio_sample_conditions,
                                bio_sample_fixations,
                                bio_sample_fixatives,
                                bio_sample_type_origin_sub_origin_links,
                                bio_sample_types,
                                bio_samples,
                                data_analysis,
                                datasets,
                                experiment_roles,
                                experimenter_proposal_links,
                                proposal_dataset_links,
                                proposals,
                                sample_origins,
                                sample_sources,
                                sample_sub_origins,
                                scan_types,
                                syncotron_runs,
                                user_access_controls,
                                users
                                };


#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName)]
#[diesel(primary_key(id))]
#[diesel(table_name = experiment_roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ExperimentRole {
    pub id: i32,
    pub role: String,
}


#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName)]
#[diesel(primary_key(id))]
#[diesel(table_name = scan_types)]
pub struct ScanType {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Queryable, Debug, Selectable, Identifiable, QueryableByName, serde::Serialize)]
#[diesel(primary_key(id))]
#[diesel(table_name = syncotron_runs)]
pub struct SyncotronRun {
    pub id: i32,
    pub name: String,
    pub start_timestamp: DateTime<Utc>,
    pub end_timestamp: DateTime<Utc>,
}

#[derive(Queryable, Debug, Selectable, QueryableByName, Identifiable)]
#[diesel(primary_key(id))]
#[diesel(table_name = user_access_controls)]
pub struct UserAccessControl {
    pub id: i32,
    pub level: String,
    pub description: String,
}

#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName)]
#[diesel(primary_key(id))]
#[diesel(table_name = beamlines)]
pub struct Beamline {
    pub id: i32,
    pub name: String,
    pub acronym: String,
    pub old_acronym: Option<String>,
    pub division: String,
    pub link: String,
}

#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName, serde::Serialize)]
#[diesel(primary_key(id))]
#[diesel(table_name = bio_sample_conditions)]
pub struct BioSampleCondition {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName, serde::Serialize)]
#[diesel(belongs_to(BioSampleFixative, foreign_key=fixative_id))]
#[diesel(primary_key(id))]
#[diesel(table_name = bio_sample_fixations)]
pub struct BioSampleFixation {
    pub id: i32,
    pub name: String,
    pub fixative_id: i32,
}

#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName, serde::Serialize)]
#[diesel(primary_key(id))]
#[diesel(table_name = bio_sample_fixatives)]
pub struct BioSampleFixative {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName, serde::Serialize)]
#[diesel(belongs_to(BioSampleType, foreign_key=bio_sample_type_id))]
#[diesel(belongs_to(SampleOrigin, foreign_key=origin_id))]
#[diesel(belongs_to(SampleSubOrigin, foreign_key=sub_origin_id))]
#[diesel(primary_key(id))]
#[diesel(table_name = bio_sample_type_origin_sub_origin_links)]
pub struct BioSampleTypeOriginSubOriginLink {
    pub id: i32,
    pub bio_sample_type_id: i32,
    pub origin_id: i32,
    pub sub_origin_id: i32,
}

#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName, serde::Serialize)]
#[diesel(primary_key(id))]
#[diesel(table_name = bio_sample_types)]
pub struct BioSampleType {
    pub id: i32,
    pub type_name: String,
}

#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName, serde::Serialize)]
#[diesel(primary_key(id))]
#[diesel(table_name = sample_origins)]
pub struct SampleOrigin {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName, serde::Serialize)]
#[diesel(primary_key(id))]
#[diesel(table_name = sample_sub_origins)]
pub struct SampleSubOrigin {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName, serde::Serialize)]
#[diesel(primary_key(id))]
#[diesel(table_name = sample_sources)]
pub struct SampleSource {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(belongs_to(BioSampleType, foreign_key=type_id))]
#[diesel(belongs_to(SampleOrigin, foreign_key=origin_id))]
#[diesel(belongs_to(SampleSubOrigin, foreign_key=sub_origin_id))]
#[diesel(belongs_to(SampleSource, foreign_key=source_id))]
#[diesel(belongs_to(Proposal, foreign_key=proposal_id))]
#[diesel(belongs_to(BioSampleCondition, foreign_key=condition_id))]
#[diesel(belongs_to(BioSampleFixation, foreign_key=fixation_id))]
#[diesel(primary_key(id))]
#[diesel(table_name = bio_samples)]
pub struct BioSample {
    pub id: i32,
    pub proposal_id: i32,
    pub name: String,
    pub type_id: i32,
    pub origin_id: i32,
    pub sub_origin_id: i32,
    pub source_id: i32,
    pub thickness: i32,
    pub cell_line: Option<String>,
    pub is_cancer: bool,
    pub condition_id: i32,
    pub treatment_details: Option<String>,
    pub fixation_id: i32,
    pub expected_elemental_content_change: Option<String>,
    pub notes: Option<String>,
}

#[derive(Queryable, Debug, Identifiable, Selectable, Serialize, Clone)]
#[diesel(primary_key(id))]
#[diesel(table_name = proposals)]
pub struct Proposal {
    pub id: i32,
    pub title: String,
    pub proprietaryflag: String,
    pub mailinflag: String,
    pub status: Option<String>,
}

#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName, Associations, serde::Serialize)]
#[diesel(belongs_to(UserAccessControl, foreign_key=user_access_control_id))]
#[diesel(primary_key(badge))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = users)]
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
#[diesel(belongs_to(Beamline, foreign_key=beamline_id), belongs_to(User, foreign_key=user_badge))]
#[diesel(table_name = beamline_contacts)]
pub struct BeamlineContact {
    pub user_badge: Option<i32>,
    pub beamline_id: Option<i32>,
    pub id: i32,
}

#[derive(Queryable, Debug, Identifiable, Associations, Selectable, QueryableByName, serde::Serialize)]
#[diesel(belongs_to(ScanType, foreign_key=scan_type_id), belongs_to(SyncotronRun, foreign_key=syncotron_run_id), belongs_to(Beamline, foreign_key=beamline_id))]
#[diesel(primary_key(id))]
#[diesel(table_name = datasets)]
pub struct Dataset {
    pub id: i32,
    pub path: String,
    pub acquisition_timestamp: NaiveDateTime,
    pub beamline_id: i32,
    pub syncotron_run_id: i32,
    pub scan_type_id: i32,
}

#[derive(Queryable, Debug, Identifiable, Associations)]
#[diesel(belongs_to(Dataset, foreign_key=dataset_id))]
#[diesel(primary_key(id))]
#[diesel(table_name = data_analysis)]
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
#[diesel(table_name = experimenter_proposal_links)]
pub struct ExperimenterProposalLink { 
    pub user_badge: i32,
    pub proposal_id: i32,
    pub experiment_role_id: i32,
    pub id: i32,
}


#[derive(Queryable, Debug, Associations, Identifiable, Selectable, QueryableByName)]
#[diesel(belongs_to(Dataset, foreign_key=dataset_id), belongs_to(Proposal, foreign_key=proposal_id))]
#[diesel(primary_key(dataset_id, proposal_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = proposal_dataset_links)]
pub struct ProposalDatasetLink {
    pub dataset_id: i32,
    pub proposal_id: i32,
}

// ------------------------- joined -------------------------

#[derive(serde::Serialize)]
pub struct DatasetWithDetails
{
    pub id: i32,
    pub path: String,
    pub acquisition_timestamp: NaiveDateTime,
    pub beamline: String,
    pub syncotron_run: String,
    //pub scan_type_name: String,
}

#[derive(serde::Serialize)]
pub struct ProposalWithDatasets
{
    #[serde(flatten)]
    pub proposal: Proposal,
    pub datasets: Vec<DatasetWithDetails>,
}

#[derive(serde::Serialize)]
pub struct BioSampleMetaDataGrouping
{
    //#[serde(flatten)]
    pub conditions: Vec<BioSampleCondition>,
    pub fixations: Vec<BioSampleFixation>,
    pub fixatives: Vec<BioSampleFixative>,
    pub sample_types: Vec<BioSampleType>,
    pub sample_origins: Vec<SampleOrigin>,
    pub sample_sub_origins: Vec<SampleSubOrigin>,
    pub samples_sources: Vec<SampleSource>,
    pub sample_type_origin_links: Vec<BioSampleTypeOriginSubOriginLink>
    
    //pub suboriginlinks: Vec<BioSampleTypeOriginSubOriginLink>,
}
