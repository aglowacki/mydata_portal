
use axum::{
    extract::{FromRef, FromRequestParts, Query, State},
    http::{request::Parts, StatusCode},
    response::Json,
    extract::Path,
};
use std::collections::HashMap;
use serde::Serialize;
use redis::Commands;

use super::appstate;
use crate::{auth};

const KEY_AVAILABLE_SCANS: &str = "_AVAILABLE_SCANS";

#[derive(Debug, Serialize, Clone)]
pub struct Plan
{
    name: String,
}

#[axum_macros::debug_handler]
pub async fn get_available_scans(
    Path(beamline_id): Path<String>,
    State(state): State<appstate::AppState>,
    //claims: auth::Claims
) -> Result<Json<String>, (StatusCode, String)> 
{
    /*
    let result = schema::users::table.find(claims.get_badge()).first::<models::User>(&mut conn).await.map_err(internal_error);
    let asking_user = match result
    {
        Ok(user) => user,
        Err(error) => panic!("Problem opening the file: {error:?}"),
    };
    */
    /*
    if database::is_admin_or_staff(&claims, &mut conn).await
    {   
        let mut conn = state.redis_client.get_connection().unwrap();
        let items: Vec<String> = conn.lrange(beamline_id, range_start, range_end).expect("Error getting logs");
        Ok(Json(res))    
    }
    else 
    {
        let err_msg = "Need to be Admin or Staff to get plans by other user.".to_string();
        Err((StatusCode::FORBIDDEN, err_msg))
    }
    */  
    let mut conn = state.redis_client.get_connection().unwrap();
    let get_id = beamline_id+KEY_AVAILABLE_SCANS;
    let str_plans: String = conn.get(get_id).expect("{msg: \"Error getting logs\"}");
    //let plans = vec![Plan{name: "abc".to_string()}];
    Ok(Json(str_plans)) 
}

#[axum_macros::debug_handler]
pub async fn get_beamline_log(
    Path(beamline_id): Path<String>,
    Query(params) : Query<HashMap<String ,isize>>,
    State(state): State<appstate::AppState>,
    claims: auth::Claims
) -> Result<Json<Vec<String>>, (StatusCode, String)> 
{
    let range_start = params.get("range_start").copied().unwrap_or(-50); // get last 50 logs
    let range_end = params.get("range_end").copied().unwrap_or(-1); 
    let mut conn = state.redis_client.get_connection().unwrap();
    let items: Vec<String> = conn.lrange(beamline_id, range_start, range_end).expect("Error getting logs");
    
    Ok(Json(items)) 
}