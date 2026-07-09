
#![allow(unused)]
#![allow(clippy::all)]


use chrono::NaiveDateTime;
use chrono::DateTime;
use chrono::offset::Utc;
use diesel::{Queryable, Identifiable, Selectable, QueryableByName, Associations, Insertable, AsChangeset};
use serde::{Serialize, Deserialize};
use crate::database::schema::{beamline_contacts,
                                beamlines,
                                bio_sample_conditions,
                                bio_sample_dataset_links,
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


#[derive(Queryable, Debug, Identifiable, Selectable, QueryableByName, serde::Serialize)]
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

/// One entry for the beamline selector dropdown. `acronym` is what the page uses
/// as its `beamline_id`; `name` is the human-readable label.
#[derive(Serialize, Debug)]
pub struct BeamlineInfo {
    pub acronym: String,
    pub name: String,
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

#[derive(Queryable, Debug, Identifiable, Selectable, serde::Serialize)]
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
    pub sub_origin_id: Option<i32>,
    pub source_id: Option<i32>,
    pub thickness: Option<i32>,
    pub cell_line: Option<String>,
    pub is_cancer: Option<bool>,
    pub condition_id: i32,
    pub treatment_details: Option<String>,
    pub fixation_id: i32,
    pub expected_elemental_content_change: Option<String>,
    pub notes: Option<String>,
}

/// Columns written when inserting or updating a `bio_samples` row. The `id`
/// column is omitted (GENERATED ALWAYS AS IDENTITY); the optional fields are
/// only filled in for the relevant sample types in the frontend form.
#[derive(Insertable, AsChangeset, Deserialize, Debug)]
#[diesel(table_name = bio_samples)]
#[diesel(treat_none_as_null = true)]
pub struct NewBioSample {
    pub proposal_id: i32,
    pub name: String,
    pub type_id: i32,
    pub origin_id: i32,
    pub sub_origin_id: Option<i32>,
    pub source_id: Option<i32>,
    pub thickness: Option<i32>,
    pub cell_line: Option<String>,
    pub is_cancer: Option<bool>,
    pub condition_id: i32,
    pub treatment_details: Option<String>,
    pub fixation_id: i32,
    pub expected_elemental_content_change: Option<String>,
    pub notes: Option<String>,
}

/// Links a dataset (scan) to the bio sample it captured. The primary key is
/// `dataset_id`, enforcing at most one sample per dataset.
#[derive(Queryable, Debug, Identifiable, Insertable, Selectable, Associations)]
#[diesel(belongs_to(BioSample, foreign_key=bio_sample_id))]
#[diesel(belongs_to(Dataset, foreign_key=dataset_id))]
#[diesel(primary_key(dataset_id))]
#[diesel(table_name = bio_sample_dataset_links)]
pub struct BioSampleDatasetLink {
    pub dataset_id: i32,
    pub bio_sample_id: i32,
}

/// Payload sent by the sample form. When `id` is present the matching row is
/// updated, otherwise a new sample is inserted. `dataset_ids` are the datasets
/// the sample information applies to.
#[derive(Deserialize, Debug)]
pub struct BioSampleUpsert {
    pub id: Option<i32>,
    pub dataset_ids: Vec<i32>,
    #[serde(flatten)]
    pub sample: NewBioSample,
}

/// Status returned to the frontend after an upsert attempt.
#[derive(Serialize)]
pub struct BioSampleUpsertResponse {
    pub success: bool,
    pub id: Option<i32>,
    pub message: String,
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

/// Query parameters for the proposal search form. All fields are optional; an
/// omitted or empty field is not used as a filter. `experimenter` is only
/// honored for Admin/Staff (a badge number, or a substring of the user's
/// username / first / last name).
#[derive(Deserialize, Debug)]
pub struct ProposalSearchParams {
    pub run: Option<String>,
    pub beamline_acronym: Option<String>,
    pub experimenter: Option<String>,
}

/// One experimenter suggestion for the search form's autocomplete. `badge` is
/// what the search actually filters on; `name` is the human-readable label.
#[derive(Serialize, Debug)]
pub struct ExperimenterOption {
    pub badge: i32,
    pub name: String,
}

/// Autocomplete values for the proposal search form, scoped like the search
/// itself: runs and beamline acronyms come from the proposals the caller can
/// see, and experimenters are only populated for Admin/Staff.
#[derive(Serialize, Debug)]
pub struct ProposalSearchOptions {
    pub runs: Vec<String>,
    pub beamline_acronyms: Vec<String>,
    pub experimenters: Vec<ExperimenterOption>,
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

/// A dataset belonging to a proposal, with display details and the id of the
/// sample currently linked to it (if any), used by the sample form's dataset
/// picker.
#[derive(serde::Serialize)]
pub struct ProposalDataset
{
    pub id: i32,
    pub path: String,
    pub acquisition_timestamp: NaiveDateTime,
    pub beamline: String,
    pub syncotron_run: String,
    pub bio_sample_id: Option<i32>,
}

#[derive(serde::Serialize)]
pub struct ProposalWithDatasets
{
    #[serde(flatten)]
    pub proposal: Proposal,
    pub datasets: Vec<DatasetWithDetails>,
}

/// One experimenter linked to a proposal, with the user's details and their role
/// on that proposal. Returned by get_proposal_experimenters.
#[derive(serde::Serialize)]
pub struct ProposalExperimenter
{
    pub badge: i32,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub institution: String,
    pub role: String,
    pub experiment_role_id: i32,
}

/// Payload for linking a user to a proposal (Admin/Staff only).
#[derive(Deserialize, Debug)]
pub struct AddExperimenterPayload
{
    pub proposal_id: i32,
    pub user_badge: i32,
    pub experiment_role_id: i32,
}

/// Payload for removing a user from a proposal (Admin/Staff only).
#[derive(Deserialize, Debug)]
pub struct RemoveExperimenterPayload
{
    pub proposal_id: i32,
    pub user_badge: i32,
}

/// Generic status returned after a mutation (add/remove) attempt.
#[derive(Serialize)]
pub struct MutationResponse
{
    pub success: bool,
    pub message: String,
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
