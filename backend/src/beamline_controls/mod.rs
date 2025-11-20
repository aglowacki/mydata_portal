
use axum::{
    extract::{FromRef, FromRequestParts, Query, State},
    http::{request::Parts, StatusCode},
    response::Json,
    extract::Path,
};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use redis::Commands;

use super::appstate;
use crate::{auth};


use defines;

#[derive(Debug, Serialize, Clone)]
pub struct Plan
{
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct LogLine
{
    time: f32,
    msg: String
}


#[axum_macros::debug_handler]
pub async fn get_available_scans(
    Path(beamline_id): Path<String>,
    State(state): State<appstate::AppState>,
    //claims: auth::Claims
) -> Result<String, (StatusCode, String)> 
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
    let get_id = format!("{}{}", defines::KEY_AVAILABLE_SCANS, beamline_id);
    let str_plans: String = conn.get(get_id).expect("{msg: \"Error getting logs\"}");
    Ok(str_plans) 
}

#[axum_macros::debug_handler]
pub async fn get_beamline_log(
    Path(beamline_id): Path<String>,
    Query(params) : Query<HashMap<String ,isize>>,
    State(state): State<appstate::AppState>,
    //claims: auth::Claims
) -> Result<Json<Vec<LogLine>>, (StatusCode, String)> 
{
    let range_start = params.get("range_start").copied().unwrap_or(-50); // get last 50 logs
    let range_end = params.get("range_end").copied().unwrap_or(-1); 
    let mut conn = state.redis_client.get_connection().unwrap();
    let get_id = format!("{}{}", defines::KEY_LOGS, beamline_id);
    let items: Vec<String> = conn.lrange(get_id, range_start, range_end).expect("Error getting logs");
    let mut beamline_logs: Vec<LogLine> = Vec::new();
    for val in items.iter() 
    {
        let ll: LogLine = serde_json::from_str(val).expect("Error parsing log line.");
        beamline_logs.push(ll);
    }
    Ok(Json(beamline_logs)) 
}