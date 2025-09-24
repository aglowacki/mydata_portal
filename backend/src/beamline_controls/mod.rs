
use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::Json,
    extract::Path,
};
use serde::Serialize;

use super::appstate;
use crate::{auth};


#[derive(Debug, Serialize, Clone)]
pub struct Plan
{
    name: String,

}

#[axum_macros::debug_handler]
pub async fn get_available_scans(
    Path(user_id): Path<i32>,
    State(state): State<appstate::AppState>,
    claims: auth::Claims
) -> Result<Json<Vec<Plan>>, (StatusCode, String)> 
{
    /*
    let result = schema::users::table.find(claims.get_badge()).first::<models::User>(&mut conn).await.map_err(internal_error);
    let asking_user = match result
    {
        Ok(user) => user,
        Err(error) => panic!("Problem opening the file: {error:?}"),
    }
    */
/*
    if is_admin_or_staff(&claims, &mut conn).await
    {   
        
        Ok(Json(res))    
    }
    else 
    {
        let err_msg = "Need to be Admin or Staff to get plans by other user.".to_string();
        Err((StatusCode::FORBIDDEN, err_msg))
    }
    */
    let plans = vec![Plan{name: "abc".to_string()}];
    Ok(Json(plans)) 
}
