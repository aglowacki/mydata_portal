
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

mod schema;
mod models;
use crate::auth;
/* 
#[derive(serde::Serialize, Selectable, Queryable)]
#[diesel(table_name = schema::users)]
pub struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
}

#[derive(serde::Deserialize, Insertable)]
#[diesel(table_name = users)]
struct NewUser {
    name: String,
    hair_color: Option<String>,
}
*/
pub type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
/*
async fn create_user(
    State(pool): State<Pool>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;

    let res = diesel::insert_into(users::table)
        .values(new_user)
        .returning(User::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(internal_error)?;
    Ok(Json(res))
}
*/
// we can also write a custom extractor that grabs a connection from the pool
// which setup is appropriate depends on your application
pub struct DatabaseConnection(bb8::PooledConnection<'static, AsyncDieselConnectionManager<AsyncPgConnection>>,);

impl<S> FromRequestParts<S> for DatabaseConnection
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = Pool::from_ref(state);

        let conn = pool.get_owned().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}
#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
}

impl FromRef<AppState> for Pool {
    fn from_ref(state: &AppState) -> Pool {
        state.pool.clone()
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
#[axum_macros::debug_handler]
pub async fn get_user_proposals(
    State(state): State<AppState>,
    claims: auth::Claims,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<models::Proposal>>, (StatusCode, String)> 
{
   
    let res = schema::proposals::table.select(models::Proposal::as_select())
    .inner_join(schema::experimenters::table.on(schema::proposals::id.eq(schema::experimenters::proposal_id)))
    .filter(schema::experimenters::user_badge.eq(claims.get_badge()))
    .distinct()
    .load(&mut conn)
    .await
    .map_err(internal_error)?;

    Ok(Json(res))
}

#[axum_macros::debug_handler]
pub async fn get_user_proposals_as(
    Path((user_id)): Path<(i32)>,
    State(state): State<AppState>,
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
    let asking_user: Vec<models::User> = schema::users::table.select(models::User::as_select())
    .inner_join(schema::user_access_controls::table.on(schema::user_access_controls::id.eq(schema::users::user_access_control_id)))
    .filter(schema::user_access_controls::level.eq("Admin").or(schema::user_access_controls::level.eq("Staff")))
    .filter(schema::users::badge.eq(claims.get_badge()))
    .load(&mut conn)
    .await
    .map_err(internal_error)?;
    
    if asking_user.len() > 0
    {
        let res = schema::proposals::table.select(models::Proposal::as_select())
        .inner_join(schema::experimenters::table.on(schema::proposals::id.eq(schema::experimenters::proposal_id)))
        .filter(schema::experimenters::user_badge.eq(user_id))
        .distinct()
        .load(&mut conn)
        .await
        .map_err(internal_error)?;

        Ok(Json(res))
    }
    else 
    {
        let v = Vec::new();
        Ok(Json(v))
    }
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
