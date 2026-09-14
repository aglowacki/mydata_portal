//! Proxy layer for the APS beamline-scheduling ("scheduler") API.
//!
//! The scheduler host requires an `Authorization` header whose value is kept in
//! the `SVC_AUTH_STR` environment variable. That value is a secret and must never
//! reach the browser, so every scheduler call is proxied through the backend: the
//! frontend hits `/api/scheduler/...`, we inject `SVC_AUTH_STR`, forward the
//! request to the scheduler host, and stream the JSON body back.
//!
//! Every route is restricted to Admin/Staff (a "beamline scientist" maps to the
//! Staff access level) via the caller's JWT claims.

use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use once_cell::sync::Lazy;
use std::env;

use crate::auth;

// Base URL of the scheduler host. Overridable via SCHEDULER_HOST so the same
// build can point at dev/prod without a recompile.
static SCHEDULER_HOST: Lazy<String> = Lazy::new(||
{
    env::var("SCHEDULER_HOST").unwrap_or_else(|_| "https://beam-api-dev.aps.anl.gov".to_string())
});

// A single reqwest client reused across requests (connection pooling).
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

// A beamline scientist is represented by the Staff access level in this system.
fn is_admin_or_staff(claims: &auth::Claims) -> bool
{
    claims.uac == defines::STR_ADMIN || claims.uac == defines::STR_STAFF
}

// Forward a GET to the scheduler host with the service Authorization header and
// return its status + JSON body to the caller. `path` is the scheduler-relative
// path (with or without a leading slash).
async fn proxy_get(claims: &auth::Claims, path: &str) -> Response
{
    if !is_admin_or_staff(claims)
    {
        return (
            StatusCode::FORBIDDEN,
            "Need to be Admin or a beamline scientist (Staff) to use the scheduler.".to_string(),
        ).into_response();
    }

    let auth_str = match env::var("SVC_AUTH_STR")
    {
        Ok(val) => val,
        Err(_) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "SVC_AUTH_STR is not configured on the server.".to_string(),
        ).into_response(),
    };

    let url = format!(
        "{}/{}",
        SCHEDULER_HOST.trim_end_matches('/'),
        path.trim_start_matches('/'),
    );

    let resp = match HTTP_CLIENT
        .get(&url)
        .header(header::ACCEPT, "*/*")
        .header(header::AUTHORIZATION, auth_str)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => return (
            StatusCode::BAD_GATEWAY,
            format!("Scheduler request failed: {err}"),
        ).into_response(),
    };

    // Preserve the upstream status code where possible.
    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let body = match resp.text().await
    {
        Ok(body) => body,
        Err(err) => return (
            StatusCode::BAD_GATEWAY,
            format!("Failed to read scheduler response: {err}"),
        ).into_response(),
    };

    (status, [(header::CONTENT_TYPE, "application/json")], body).into_response()
}

// --- Activities ------------------------------------------------------------

#[axum_macros::debug_handler]
pub async fn activity_by_run_and_beamline(
    Path((syncotron_run, beamline_id)): Path<(String, String)>,
    claims: auth::Claims,
) -> Response
{
    let path = format!(
        "beamline-scheduling/sched-api/activity/findByRunNameAndBeamlineId/{syncotron_run}/{beamline_id}"
    );
    proxy_get(&claims, &path).await
}

#[axum_macros::debug_handler]
pub async fn activity_by_id(
    Path(activity_id): Path<String>,
    claims: auth::Claims,
) -> Response
{
    let path = format!("beamline-scheduling/sched-api/activity/findByActivityId/{activity_id}");
    proxy_get(&claims, &path).await
}

// --- Beamlines -------------------------------------------------------------

#[axum_macros::debug_handler]
pub async fn beamlines_by_id(
    Path(beamline_id): Path<String>,
    claims: auth::Claims,
) -> Response
{
    let path = format!("beamline-scheduling/sched-api/beamline/findAllBeamlinesByBeamlineId/{beamline_id}");
    proxy_get(&claims, &path).await
}

#[axum_macros::debug_handler]
pub async fn active_beamlines(claims: auth::Claims) -> Response
{
    proxy_get(&claims, "beamline-scheduling/sched-api/beamline/findAllActiveBeamlines").await
}

#[axum_macros::debug_handler]
pub async fn authorized_beamlines(claims: auth::Claims) -> Response
{
    proxy_get(&claims, "beamline-scheduling/sched-api/userBeamlineAuthorizedEdit/getAuthorizedBeamlines").await
}

// --- Beamtime requests -----------------------------------------------------

#[axum_macros::debug_handler]
pub async fn beamtime_requests(
    Path((syncotron_run, beamline_id)): Path<(String, String)>,
    claims: auth::Claims,
) -> Response
{
    let path = format!(
        "beamline-scheduling/sched-api/beamtimeRequests/findBeamtimeRequestsByRunAndBeamline/{syncotron_run}/{beamline_id}"
    );
    proxy_get(&claims, &path).await
}

// --- Runs ------------------------------------------------------------------

#[axum_macros::debug_handler]
pub async fn all_runs(claims: auth::Claims) -> Response
{
    proxy_get(&claims, "beamline-scheduling/sched-api/run/getAllRuns").await
}

#[axum_macros::debug_handler]
pub async fn current_run(claims: auth::Claims) -> Response
{
    proxy_get(&claims, "beamline-scheduling/sched-api/run/getCurrentRun").await
}

#[axum_macros::debug_handler]
pub async fn run_by_year(
    Path(year): Path<String>,
    claims: auth::Claims,
) -> Response
{
    let path = format!("beamline-scheduling/sched-api/run/getRunByRunYear/{year}");
    proxy_get(&claims, &path).await
}
