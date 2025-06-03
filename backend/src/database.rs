
use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::Json,
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
    /*
    let res = schema::experimenters::table
    .inner_join(schema::proposals::table.on(schema::proposals::id.eq(schema::experimenters::proposal_id)))
    .inner_join(schema::users::table.on(schema::experimenters::user_badge.eq(schema::users::badge)))
    .select(schema::proposals::table)
    .load::(&mut conn)
    .await
    .map_err((internal_error))?;
*/
    let res = schema::proposals::table.select(models::Proposal::as_select())
    .inner_join(schema::experimenters::table.on(schema::proposals::id.eq(schema::experimenters::proposal_id)))
    .inner_join(schema::users::table.on(schema::experimenters::user_badge.eq(schema::users::badge)))
    .filter(schema::users::badge.eq(claims.get_badge()))
    .load(&mut conn)
    .await
    .map_err(internal_error)?;

    /*
    let res = schema::proposals::table.select(models::Proposal::as_select())
    .load(&mut conn)
    .await
    .map_err((internal_error))?;
*/
    Ok(Json(res))
}


/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
