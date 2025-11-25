
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
mod models;
use super::appstate;
use crate::{auth};

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

#[axum_macros::debug_handler]
pub async fn get_user_proposals_as(
    Path(user_id): Path<i32>,
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<models::Proposal>>, (StatusCode, String)> 
{
    /*
    let result = schema::users::table.find(claims.get_badge()).first::<models::User>(&mut conn).await.map_err(internal_error);
    let asking_user = match result
    {
        Ok(user) => user,
        Err(error) => panic!("Problem opening the file: {error:?}"),
    }
    */

    if is_admin_or_staff(&claims, &mut conn).await
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
    if is_admin_or_staff(&claims, &mut conn).await
    {   
        let user_proposals: Vec<_> = schema::proposals::table.select(models::Proposal::as_select())
        .inner_join(schema::experimenter_proposal_links::table.on(schema::proposals::id.eq(schema::experimenter_proposal_links::proposal_id)))
        .filter(schema::experimenter_proposal_links::user_badge.eq(user_id))
        .distinct()
        .load(&mut conn)
        .await
        .map_err(internal_error)?;
        
        let mut proposals_with_datasets = Vec::new();
        for proposal in user_proposals        
        {
            let datasets = schema::datasets::table.select(models::Dataset::as_select())
            .inner_join(schema::proposal_dataset_links::table.on(schema::datasets::id.eq(schema::proposal_dataset_links::dataset_id)))
            .filter(schema::proposal_dataset_links::proposal_id.eq(proposal.id))
            .distinct()
            .load(&mut conn)
            .await
            .map_err(internal_error)?;
        
            proposals_with_datasets.push( models::ProposalWithDatasets { proposal, datasets } );
        }
        
        Ok(Json(proposals_with_datasets))

    }
    else 
    {
        let err_msg = "Need to be Admin or Staff to get proposals by other user.".to_string();
        Err((StatusCode::FORBIDDEN, err_msg))
    }
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


/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
