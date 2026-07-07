
use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::Json,
    extract::Path,
};
use axum_macros::debug_handler;
use diesel::prelude::*;
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection, RunQueryDsl,
};

use bb8::PooledConnection;

mod schema;
pub mod models;
use super::appstate;
use crate::{auth};

use diesel::pg::Pg;
use diesel::PgTextExpressionMethods;

// we can also write a custom extractor that grabs a connection from the pool
// which setup is appropriate depends on your application
pub struct DatabaseConnection(bb8::PooledConnection<'static, AsyncDieselConnectionManager<AsyncPgConnection>>,);

impl<S> FromRequestParts<S> for DatabaseConnection
where
    S: Send + Sync,
    appstate::DieselPool: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = appstate::DieselPool::from_ref(state);

        let conn = pool.get_owned().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}


pub async fn get_beamlines(state: &appstate::AppState,) -> Vec<models::Beamline>
{
    let pool = appstate::DieselPool::from_ref(state);
    let mut conn = pool.get_owned().await.unwrap();
    
    let beamlines = schema::beamlines::table.select(models::Beamline::as_select())
        .load(&mut conn)
        .await
        .unwrap_or(Vec::new());
    return beamlines;
}


/*
#[axum_macros::debug_handler]
pub async fn list_users(
    State(state): State<AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<models::User>>, (StatusCode, String)> 
{
    let res = schema::users::table
        .select(models::User::as_select())
        .load(&mut conn)
        .await
        .map_err(internal_error)?;
    Ok(Json(res))
}
*/

#[axum_macros::debug_handler]
pub async fn authorize_user(
    State(state): State<appstate::AppState>,
    DatabaseConnection(mut conn): DatabaseConnection,
    Json(payload): Json<auth::AuthPayload>,
) -> Result<Json<auth::AuthBody>, auth::AuthError>
{
    if payload.client_id.is_empty() || payload.client_secret.is_empty() 
    {
        return Err(auth::AuthError::MissingCredentials);
    }
    let result = auth::authorize_ldap(&payload.client_id, &payload.client_secret).await;
    match result
    { 
        Ok(mut claims) =>
        {
            let result = schema::users::table
            .inner_join(schema::user_access_controls::table.on(schema::user_access_controls::id.eq(schema::users::user_access_control_id)))
            .filter(schema::users::badge.eq(claims.get_badge()))
            .load::<(models::User, models::UserAccessControl)>(&mut conn)
            .await.map_err(internal_error).unwrap_or(Vec::new());
        
            if result.len() > 0 
            {
                claims.uac = result[0].1.level.clone();
                // Send the authorized token
                match claims.encode_to_string()
                {
                    Ok(token) => Ok(Json(auth::AuthBody::new(token))),
                    Err(_) => Err(auth::AuthError::WrongCredentials),
                }
            }
            else 
            {
                return Err(auth::AuthError::WrongCredentials)
            }
            
        },
        Err(_) => return Err(auth::AuthError::WrongCredentials),
    }
}
/*
async fn is_admin_or_staff(claims: &auth::Claims, conn: &mut PooledConnection<'static, AsyncDieselConnectionManager<AsyncPgConnection>>) -> bool
{
    let asking_user: Vec<models::User> = schema::users::table.select(models::User::as_select())
    .inner_join(schema::user_access_controls::table.on(schema::user_access_controls::id.eq(schema::users::user_access_control_id)))
    .filter(schema::user_access_controls::level.eq("Admin").or(schema::user_access_controls::level.eq("Staff")))
    .filter(schema::users::badge.eq(claims.get_badge()))
    .load(conn)
    .await
    .map_err(internal_error).unwrap_or(Vec::new());
    
    if asking_user.len() > 0
    {
        return true;   
    }
    else 
    {
        return false;    
    }
}

pub async fn set_user_access_control(claims: &mut auth::Claims, conn: &mut PooledConnection<'static, AsyncDieselConnectionManager<AsyncPgConnection>>) -> bool
{
    let asking_user: Vec<models::UserAccessControl> = schema::user_access_controls::table.select(models::UserAccessControl::as_select())
    .inner_join(schema::users::table.on(schema::user_access_controls::id.eq(schema::users::user_access_control_id)))
    .filter(schema::users::badge.eq(claims.get_badge()))
    .load(conn)
    .await
    .map_err(internal_error).unwrap_or(Vec::new());
    
    if asking_user.len() > 0
    {
        claims.uac = asking_user[0].level.clone();
        println!("{}", &asking_user[0].level);
        return true;
    }
    else 
    {
        return false;    
    }
}
*/
#[axum_macros::debug_handler]
pub async fn get_user_proposals(
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<models::Proposal>>, (StatusCode, String)> 
{
   
    let res = schema::proposals::table.select(models::Proposal::as_select())
    .inner_join(schema::experimenter_proposal_links::table.on(schema::proposals::id.eq(schema::experimenter_proposal_links::proposal_id)))
    .filter(schema::experimenter_proposal_links::user_badge.eq(claims.get_badge()))
    .distinct()
    .load(&mut conn)
    .await
    .map_err(internal_error)?;

    Ok(Json(res))
}

/// Return every proposal. Restricted to Admin/Staff so they can attach samples
/// to any proposal from the sample form.
#[axum_macros::debug_handler]
pub async fn get_all_proposals(
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<models::Proposal>>, (StatusCode, String)>
{
    if claims.uac == defines::STR_ADMIN || claims.uac == defines::STR_STAFF
    {
        let res = schema::proposals::table.select(models::Proposal::as_select())
        .load(&mut conn)
        .await
        .map_err(internal_error)?;

        Ok(Json(res))
    }
    else
    {
        let err_msg = "Need to be Admin or Staff to get all proposals.".to_string();
        Err((StatusCode::FORBIDDEN, err_msg))
    }
}

#[axum_macros::debug_handler]
pub async fn get_user_proposals_as(
    Path(user_id): Path<i32>,
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<models::Proposal>>, (StatusCode, String)> 
{
    if claims.uac == defines::STR_ADMIN || claims.uac == defines::STR_STAFF
    {   
        let res: Vec<_> = schema::proposals::table.select(models::Proposal::as_select())
        .inner_join(schema::experimenter_proposal_links::table.on(schema::proposals::id.eq(schema::experimenter_proposal_links::proposal_id)))
        .filter(schema::experimenter_proposal_links::user_badge.eq(user_id))
        .distinct()
        .load(&mut conn)
        .await
        .map_err(internal_error)?;
        
        Ok(Json(res))    
    }
    else 
    {
        let err_msg = "Need to be Admin or Staff to get proposals by other user.".to_string();
        Err((StatusCode::FORBIDDEN, err_msg))
    }
}

#[axum_macros::debug_handler]
pub async fn get_user_proposals_with_datasets(
    Path(user_id): Path<i32>,
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<models::ProposalWithDatasets>>, (StatusCode, String)> 
{
    if claims.uac == defines::STR_ADMIN || claims.uac == defines::STR_STAFF
    {   
        let user_proposals: Vec<_> = schema::proposals::table.select(models::Proposal::as_select())
        .inner_join(schema::experimenter_proposal_links::table.on(schema::proposals::id.eq(schema::experimenter_proposal_links::proposal_id)))
        .filter(schema::experimenter_proposal_links::user_badge.eq(user_id))
        .distinct()
        .load(&mut conn)
        .await
        .map_err(internal_error)?;
        
        let mut proposals_with_datasets: Vec<models::ProposalWithDatasets> = Vec::new();
        for proposal in user_proposals        
        {
            let datasets = schema::datasets::table
            .inner_join(schema::proposal_dataset_links::table.on(schema::datasets::id.eq(schema::proposal_dataset_links::dataset_id)))
            .inner_join(schema::beamlines::table.on(schema::beamlines::id.eq(schema::datasets::beamline_id)))
            .inner_join(schema::syncotron_runs::table.on(schema::syncotron_runs::id.eq(schema::datasets::syncotron_run_id)))
            .filter(schema::proposal_dataset_links::proposal_id.eq(proposal.id))
            .distinct()
            .load::<(models::Dataset, models::ProposalDatasetLink, models::Beamline, models::SyncotronRun)>(&mut conn)
            .await
            .map_err(internal_error).unwrap_or(Vec::new());
        
            let mut dwd: Vec<models::DatasetWithDetails> = Vec::new();
            for dset in datasets
            {
                dwd.push(models::DatasetWithDetails {
                    id: dset.0.id.clone(),
                    path: dset.0.path.clone(),
                    acquisition_timestamp: dset.0.acquisition_timestamp,
                    beamline: dset.2.acronym,
                    syncotron_run: dset.3.name, 
                });
            }
            proposals_with_datasets.push( models::ProposalWithDatasets { proposal, datasets: dwd } );
        }
        
        Ok(Json(proposals_with_datasets))

    }
    else
    {
        let err_msg = "Need to be Admin or Staff to get proposals by other user.".to_string();
        Err((StatusCode::FORBIDDEN, err_msg))
    }
}

/// Look up the logged-in user's proposals by a property of their datasets:
/// beamline acronym, beamline old_acronym, or syncotron run name. `field`
/// selects which column to match; `value` is the exact value to match.
#[axum_macros::debug_handler]
pub async fn search_user_proposals(
    Path((field, value)): Path<(String, String)>,
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<models::Proposal>>, (StatusCode, String)>
{
    // Restrict to proposals the user is associated with, joined through their
    // datasets to the beamline and syncotron run they were collected on.
    let mut query = schema::proposals::table
        .inner_join(schema::experimenter_proposal_links::table.on(schema::proposals::id.eq(schema::experimenter_proposal_links::proposal_id)))
        .inner_join(schema::proposal_dataset_links::table.on(schema::proposals::id.eq(schema::proposal_dataset_links::proposal_id)))
        .inner_join(schema::datasets::table.on(schema::datasets::id.eq(schema::proposal_dataset_links::dataset_id)))
        .inner_join(schema::beamlines::table.on(schema::beamlines::id.eq(schema::datasets::beamline_id)))
        .inner_join(schema::syncotron_runs::table.on(schema::syncotron_runs::id.eq(schema::datasets::syncotron_run_id)))
        .filter(schema::experimenter_proposal_links::user_badge.eq(claims.get_badge()))
        .select(models::Proposal::as_select())
        .distinct()
        .into_boxed();

    match field.as_str()
    {
        "beamline_acronym" => { query = query.filter(schema::beamlines::acronym.eq(value)); }
        "beamline_old_acronym" => { query = query.filter(schema::beamlines::old_acronym.eq(value)); }
        "syncotron_run" => { query = query.filter(schema::syncotron_runs::name.eq(value)); }
        _ => { return Err((StatusCode::BAD_REQUEST, format!("Unknown search field: {}", field))); }
    }

    let res = query.load(&mut conn).await.map_err(internal_error)?;

    Ok(Json(res))
}

/// Search proposals by dataset properties (syncotron run name and/or beamline
/// acronym) and, for Admin/Staff, by experimenter. Regular users only ever see
/// proposals they are associated with; Admin/Staff search across all proposals.
/// All filters are optional and combined with AND.
#[axum_macros::debug_handler]
pub async fn search_proposals(
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
    axum::extract::Query(params): axum::extract::Query<models::ProposalSearchParams>,
) -> Result<Json<Vec<models::Proposal>>, (StatusCode, String)>
{
    let is_admin = claims.uac == defines::STR_ADMIN || claims.uac == defines::STR_STAFF;

    // Join proposals to their experimenters and, via their datasets, to the
    // beamline and syncotron run those datasets were collected on.
    let mut query = schema::proposals::table
        .inner_join(schema::experimenter_proposal_links::table.on(schema::proposals::id.eq(schema::experimenter_proposal_links::proposal_id)))
        .inner_join(schema::users::table.on(schema::users::badge.eq(schema::experimenter_proposal_links::user_badge)))
        .inner_join(schema::proposal_dataset_links::table.on(schema::proposals::id.eq(schema::proposal_dataset_links::proposal_id)))
        .inner_join(schema::datasets::table.on(schema::datasets::id.eq(schema::proposal_dataset_links::dataset_id)))
        .inner_join(schema::beamlines::table.on(schema::beamlines::id.eq(schema::datasets::beamline_id)))
        .inner_join(schema::syncotron_runs::table.on(schema::syncotron_runs::id.eq(schema::datasets::syncotron_run_id)))
        .select(models::Proposal::as_select())
        .distinct()
        .into_boxed();

    // Non-admins are restricted to their own proposals; the experimenter filter
    // is ignored for them.
    if !is_admin
    {
        query = query.filter(schema::experimenter_proposal_links::user_badge.eq(claims.get_badge()));
    }

    if let Some(run) = params.run.as_deref().map(str::trim).filter(|s| !s.is_empty())
    {
        query = query.filter(schema::syncotron_runs::name.eq(run.to_string()));
    }

    if let Some(acronym) = params.beamline_acronym.as_deref().map(str::trim).filter(|s| !s.is_empty())
    {
        query = query.filter(schema::beamlines::acronym.eq(acronym.to_string()));
    }

    if is_admin
    {
        if let Some(exp) = params.experimenter.as_deref().map(str::trim).filter(|s| !s.is_empty())
        {
            if let Ok(badge) = exp.parse::<i32>()
            {
                query = query.filter(schema::experimenter_proposal_links::user_badge.eq(badge));
            }
            else
            {
                // Match a substring of the username, first name, or last name.
                let pattern = format!("%{}%", exp);
                query = query.filter(
                    schema::users::username.ilike(pattern.clone())
                        .or(schema::users::first_name.ilike(pattern.clone()))
                        .or(schema::users::last_name.ilike(pattern)));
            }
        }
    }

    let res = query.load(&mut conn).await.map_err(internal_error)?;

    Ok(Json(res))
}

/// Distinct values used to power the proposal search form's autocomplete: the
/// syncotron run names and beamline acronyms found on proposals the caller can
/// see (their own, or all for Admin/Staff), plus the list of experimenters
/// (Admin/Staff only).
#[axum_macros::debug_handler]
pub async fn get_proposal_search_options(
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<models::ProposalSearchOptions>, (StatusCode, String)>
{
    let is_admin = claims.uac == defines::STR_ADMIN || claims.uac == defines::STR_STAFF;

    // Distinct syncotron run names on datasets the caller can see.
    let mut runs_query = schema::datasets::table
        .inner_join(schema::syncotron_runs::table.on(schema::syncotron_runs::id.eq(schema::datasets::syncotron_run_id)))
        .inner_join(schema::proposal_dataset_links::table.on(schema::proposal_dataset_links::dataset_id.eq(schema::datasets::id)))
        .inner_join(schema::experimenter_proposal_links::table.on(schema::experimenter_proposal_links::proposal_id.eq(schema::proposal_dataset_links::proposal_id)))
        .select(schema::syncotron_runs::name)
        .distinct()
        .into_boxed();
    if !is_admin
    {
        runs_query = runs_query.filter(schema::experimenter_proposal_links::user_badge.eq(claims.get_badge()));
    }
    let mut runs: Vec<String> = runs_query.load::<String>(&mut conn).await.map_err(internal_error)?
        .into_iter().map(|r| r.trim().to_string()).filter(|r| !r.is_empty()).collect();
    runs.sort();
    runs.dedup();

    // Distinct beamline acronyms on datasets the caller can see.
    let mut beamlines_query = schema::datasets::table
        .inner_join(schema::beamlines::table.on(schema::beamlines::id.eq(schema::datasets::beamline_id)))
        .inner_join(schema::proposal_dataset_links::table.on(schema::proposal_dataset_links::dataset_id.eq(schema::datasets::id)))
        .inner_join(schema::experimenter_proposal_links::table.on(schema::experimenter_proposal_links::proposal_id.eq(schema::proposal_dataset_links::proposal_id)))
        .select(schema::beamlines::acronym)
        .distinct()
        .into_boxed();
    if !is_admin
    {
        beamlines_query = beamlines_query.filter(schema::experimenter_proposal_links::user_badge.eq(claims.get_badge()));
    }
    let mut beamline_acronyms: Vec<String> = beamlines_query.load::<String>(&mut conn).await.map_err(internal_error)?
        .into_iter().map(|b| b.trim().to_string()).filter(|b| !b.is_empty()).collect();
    beamline_acronyms.sort();
    beamline_acronyms.dedup();

    // Experimenters are Admin/Staff only: every distinct user linked to a proposal.
    let experimenters = if is_admin
    {
        let mut rows: Vec<(i32, String, String, String)> = schema::experimenter_proposal_links::table
            .inner_join(schema::users::table.on(schema::users::badge.eq(schema::experimenter_proposal_links::user_badge)))
            .select((schema::users::badge, schema::users::first_name, schema::users::last_name, schema::users::username))
            .distinct()
            .load::<(i32, String, String, String)>(&mut conn)
            .await
            .map_err(internal_error)?;
        rows.sort_by(|a, b| a.2.to_lowercase().cmp(&b.2.to_lowercase()));
        rows.into_iter()
            .map(|(badge, first, last, username)| models::ExperimenterOption {
                badge,
                name: format!("{} {} ({})", first.trim(), last.trim(), username.trim()),
            })
            .collect()
    }
    else
    {
        Vec::new()
    };

    Ok(Json(models::ProposalSearchOptions { runs, beamline_acronyms, experimenters }))
}

#[axum_macros::debug_handler]
pub async fn get_syncotron_runs(
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<models::SyncotronRun>>, (StatusCode, String)> 
{
   
    let res = schema::syncotron_runs::table.select(models::SyncotronRun::as_select())
    .load(&mut conn)
    .await
    .map_err(internal_error)?;

    Ok(Json(res))
}
/*
// depricated for get_bio_sample_meta_data_groups
#[axum_macros::debug_handler]
pub async fn get_bio_sample_types(
    State(state): State<appstate::AppState>,
    //claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<models::BioSampleType>>, (StatusCode, String)> 
{
    let res = schema::bio_sample_types::table.select(models::BioSampleType::as_select())
    .distinct()
    .load(&mut conn)
    .await
    .map_err(internal_error)?;

    Ok(Json(res))
}
*/
#[axum_macros::debug_handler]
pub async fn get_bio_sample_meta_data_groups(
    State(state): State<appstate::AppState>,
    //claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<models::BioSampleMetaDataGrouping>, (StatusCode, String)> 
{

    let conditions: Vec<_> = schema::bio_sample_conditions::table.select(models::BioSampleCondition::as_select())
    .distinct()
    .load(&mut conn)
    .await
    .map_err(internal_error)?;
    
    let fixations: Vec<_> = schema::bio_sample_fixations::table.select(models::BioSampleFixation::as_select())
    .distinct()
    .load(&mut conn)
    .await
    .map_err(internal_error)?;
    
    let fixatives: Vec<_> = schema::bio_sample_fixatives::table.select(models::BioSampleFixative::as_select())
    .distinct()
    .load(&mut conn)
    .await
    .map_err(internal_error)?;

    let sample_types: Vec<_> = schema::bio_sample_types::table.select(models::BioSampleType::as_select())
    .distinct()
    .load(&mut conn)
    .await
    .map_err(internal_error)?;

    let sample_origins: Vec<_> = schema::sample_origins::table.select(models::SampleOrigin::as_select())
    .distinct()
    .load(&mut conn)
    .await
    .map_err(internal_error)?;

    let sample_sub_origins: Vec<_> = schema::sample_sub_origins::table.select(models::SampleSubOrigin::as_select())
    .distinct()
    .load(&mut conn)
    .await
    .map_err(internal_error)?;

    let samples_sources: Vec<_> = schema::sample_sources::table.select(models::SampleSource::as_select())
    .distinct()
    .load(&mut conn)
    .await
    .map_err(internal_error)?;
    
    let sample_type_origin_links: Vec<_> = schema::bio_sample_type_origin_sub_origin_links::table.select(models::BioSampleTypeOriginSubOriginLink::as_select())
    .distinct()
    .load(&mut conn)
    .await
    .map_err(internal_error)?;


    Ok(Json(models::BioSampleMetaDataGrouping{conditions, fixations, fixatives, sample_types, sample_origins, sample_sub_origins, samples_sources, sample_type_origin_links}))


}

/// Insert a new bio sample or update an existing one (when `id` is supplied).
/// Performs basic validation and verifies the requesting user is associated
/// with the target proposal (admins/staff may write to any proposal). Always
/// returns a JSON status the frontend can act on.
#[axum_macros::debug_handler]
pub async fn upsert_bio_sample(
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
    Json(payload): Json<models::BioSampleUpsert>,
) -> Json<models::BioSampleUpsertResponse>
{
    let fail = |msg: &str| Json(models::BioSampleUpsertResponse {
        success: false,
        id: None,
        message: msg.to_string(),
    });

    // ---- basic validation ----
    if payload.sample.name.trim().is_empty()
    {
        return fail("Sample name is required.");
    }
    if payload.sample.proposal_id <= 0
    {
        return fail("A proposal must be selected.");
    }
    if payload.sample.type_id <= 0
    {
        return fail("A sample type must be selected.");
    }
    if payload.sample.origin_id <= 0
    {
        return fail("A sample origin must be selected.");
    }
    if payload.sample.condition_id <= 0
    {
        return fail("A sample condition must be selected.");
    }
    if payload.sample.fixation_id <= 0
    {
        return fail("A sample fixation must be selected.");
    }
    if payload.dataset_ids.is_empty()
    {
        return fail("Select at least one dataset to assign this sample to.");
    }

    // ---- authorization: user must be on the proposal (unless admin/staff) ----
    if claims.uac != defines::STR_ADMIN && claims.uac != defines::STR_STAFF
    {
        let owned: i64 = match schema::experimenter_proposal_links::table
            .filter(schema::experimenter_proposal_links::user_badge.eq(claims.get_badge()))
            .filter(schema::experimenter_proposal_links::proposal_id.eq(payload.sample.proposal_id))
            .count()
            .get_result(&mut conn)
            .await
        {
            Ok(c) => c,
            Err(err) => return fail(&format!("Failed to verify proposal access: {}", err)),
        };

        if owned == 0
        {
            return fail("You are not associated with the selected proposal.");
        }
    }

    // ---- the selected datasets must belong to the proposal ----
    let valid_count: i64 = match schema::proposal_dataset_links::table
        .filter(schema::proposal_dataset_links::proposal_id.eq(payload.sample.proposal_id))
        .filter(schema::proposal_dataset_links::dataset_id.eq_any(&payload.dataset_ids))
        .count()
        .get_result(&mut conn)
        .await
    {
        Ok(c) => c,
        Err(err) => return fail(&format!("Failed to verify datasets: {}", err)),
    };
    if valid_count != payload.dataset_ids.len() as i64
    {
        return fail("One or more selected datasets do not belong to the proposal.");
    }

    // ---- insert or update the sample row ----
    let (sample_id, base_msg) = match payload.id
    {
        Some(id) =>
        {
            let res = diesel::update(schema::bio_samples::table.find(id))
                .set(&payload.sample)
                .returning(schema::bio_samples::id)
                .get_result::<i32>(&mut conn)
                .await;
            match res
            {
                Ok(updated_id) => (updated_id, format!("Sample {} updated successfully.", updated_id)),
                Err(diesel::result::Error::NotFound) =>
                    return fail(&format!("No sample found with id {}.", id)),
                Err(err) => return fail(&format!("Failed to update sample: {}", err)),
            }
        }
        None =>
        {
            let res = diesel::insert_into(schema::bio_samples::table)
                .values(&payload.sample)
                .returning(schema::bio_samples::id)
                .get_result::<i32>(&mut conn)
                .await;
            match res
            {
                Ok(new_id) => (new_id, format!("Sample created successfully (id {}).", new_id)),
                Err(err) => return fail(&format!("Failed to create sample: {}", err)),
            }
        }
    };

    // ---- reconcile dataset links (one sample per dataset) ----
    // Drop this sample's links that are no longer selected.
    if let Err(err) = diesel::delete(
            schema::bio_sample_dataset_links::table
                .filter(schema::bio_sample_dataset_links::bio_sample_id.eq(sample_id))
                .filter(schema::bio_sample_dataset_links::dataset_id.ne_all(&payload.dataset_ids)))
        .execute(&mut conn)
        .await
    {
        return fail(&format!("Sample saved but failed to update dataset links: {}", err));
    }

    // Link (or reassign) each selected dataset to this sample.
    for ds_id in &payload.dataset_ids
    {
        let res = diesel::insert_into(schema::bio_sample_dataset_links::table)
            .values((
                schema::bio_sample_dataset_links::dataset_id.eq(*ds_id),
                schema::bio_sample_dataset_links::bio_sample_id.eq(sample_id),
            ))
            .on_conflict(schema::bio_sample_dataset_links::dataset_id)
            .do_update()
            .set(schema::bio_sample_dataset_links::bio_sample_id.eq(sample_id))
            .execute(&mut conn)
            .await;
        if let Err(err) = res
        {
            return fail(&format!("Sample saved but failed to link dataset {}: {}", ds_id, err));
        }
    }

    Json(models::BioSampleUpsertResponse {
        success: true,
        id: Some(sample_id),
        message: format!("{} Linked to {} dataset(s).", base_msg, payload.dataset_ids.len()),
    })
}

/// Return the datasets linked to a proposal, with display details and the id of
/// the sample currently assigned to each (if any). Used by the sample form's
/// dataset picker. Restricted to the proposal's experimenters and Admin/Staff.
#[axum_macros::debug_handler]
pub async fn get_proposal_datasets(
    Path(proposal_id): Path<i32>,
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<models::ProposalDataset>>, (StatusCode, String)>
{
    if claims.uac != defines::STR_ADMIN && claims.uac != defines::STR_STAFF
    {
        let owned: i64 = schema::experimenter_proposal_links::table
            .filter(schema::experimenter_proposal_links::user_badge.eq(claims.get_badge()))
            .filter(schema::experimenter_proposal_links::proposal_id.eq(proposal_id))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(internal_error)?;
        if owned == 0
        {
            return Err((StatusCode::FORBIDDEN, "You are not associated with the selected proposal.".to_string()));
        }
    }

    let rows = schema::datasets::table
        .inner_join(schema::proposal_dataset_links::table.on(schema::datasets::id.eq(schema::proposal_dataset_links::dataset_id)))
        .inner_join(schema::beamlines::table.on(schema::beamlines::id.eq(schema::datasets::beamline_id)))
        .inner_join(schema::syncotron_runs::table.on(schema::syncotron_runs::id.eq(schema::datasets::syncotron_run_id)))
        .left_join(schema::bio_sample_dataset_links::table.on(schema::bio_sample_dataset_links::dataset_id.eq(schema::datasets::id)))
        .filter(schema::proposal_dataset_links::proposal_id.eq(proposal_id))
        .select((
            models::Dataset::as_select(),
            models::Beamline::as_select(),
            models::SyncotronRun::as_select(),
            schema::bio_sample_dataset_links::bio_sample_id.nullable(),
        ))
        .distinct()
        .load::<(models::Dataset, models::Beamline, models::SyncotronRun, Option<i32>)>(&mut conn)
        .await
        .map_err(internal_error)?;

    let datasets = rows.into_iter().map(|(d, b, s, sample_id)| models::ProposalDataset {
        id: d.id,
        path: d.path,
        acquisition_timestamp: d.acquisition_timestamp,
        beamline: b.acronym,
        syncotron_run: s.name,
        bio_sample_id: sample_id,
    }).collect();

    Ok(Json(datasets))
}

/// Return every bio sample associated with a proposal. The rows hold lookup ids
/// (type, origin, condition, fixation, ...) which the frontend resolves to names
/// using the bio-sample metadata groups. Restricted to the proposal's
/// experimenters and Admin/Staff.
#[axum_macros::debug_handler]
pub async fn get_proposal_bio_samples(
    Path(proposal_id): Path<i32>,
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<models::BioSample>>, (StatusCode, String)>
{
    if claims.uac != defines::STR_ADMIN && claims.uac != defines::STR_STAFF
    {
        let owned: i64 = schema::experimenter_proposal_links::table
            .filter(schema::experimenter_proposal_links::user_badge.eq(claims.get_badge()))
            .filter(schema::experimenter_proposal_links::proposal_id.eq(proposal_id))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(internal_error)?;
        if owned == 0
        {
            return Err((StatusCode::FORBIDDEN, "You are not associated with the selected proposal.".to_string()));
        }
    }

    let samples: Vec<models::BioSample> = schema::bio_samples::table
        .filter(schema::bio_samples::proposal_id.eq(proposal_id))
        .select(models::BioSample::as_select())
        .load(&mut conn)
        .await
        .map_err(internal_error)?;

    Ok(Json(samples))
}


/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
