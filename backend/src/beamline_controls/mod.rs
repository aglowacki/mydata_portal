
use axum::{
    body::Body,
    extract::{FromRef, FromRequestParts, Query, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse,Json,Response},
    extract::Path,
};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use redis::Commands;

use super::appstate;
use crate::{auth};


use defines;
use beamline_worker::command_protocols::{BeamlineCommand, BeamlineTaskQueues};

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
    let get_id = format!("{}{}", defines::KEY_BEAMLINE_AVAILABLE_SCANS, beamline_id);
    let str_plans: String = conn.get(get_id).expect("{msg: \"Error getting available scans\"}");
    Ok(str_plans) 
}

// Beamline tasks -------------------------------------------------------------------------------------------------

#[axum_macros::debug_handler]
pub async fn queue_beamline_worker_task(
    Path(beamline_id): Path<String>,
    State(state): State<appstate::AppState>,
    claims: auth::Claims,
    Json(payload): Json<BeamlineCommand>
) -> impl IntoResponse
{
    println!("{}",&payload);
    let command =  BeamlineCommand::gen_queued_from_command(&beamline_id, claims.get_username(), &payload);
    println!("{}",&command);
    let task_payload = serde_json::to_string(&command);
    match task_payload
    {
        Ok(task_str) => 
        {
            let mut conn = state.redis_client.get_connection().unwrap();
            let set_id = format!("{}{}", defines::KEY_TASK_QUEUE_WAITING, beamline_id);
            let result: redis::RedisResult<()> = conn.rpush(&set_id, &task_str);
            match result 
            {
                Ok(_) => Response::builder()
                .status(StatusCode::CREATED)
                .body(Body::from("Task queued successfully"))
                .unwrap(),
                Err(err) => Response::builder()
                .status(StatusCode::EXPECTATION_FAILED)
                .body(Body::from("Task failed to queued"))
                .unwrap(),
            }
        }
        Err(b_err) => 
        {
            Response::builder()
                .status(StatusCode::EXPECTATION_FAILED)
                .body(Body::from(b_err.to_string()))
                .unwrap()
        }
    }
}

#[axum_macros::debug_handler]
pub async fn get_queued_scans(
    Path(beamline_id): Path<String>,
    State(state): State<appstate::AppState>,
    //claims: auth::Claims
) -> Result<String, (StatusCode, String)> 
{
    let mut conn = state.redis_client.get_connection().unwrap();
    let get_id = format!("{}{}", defines::KEY_BEAMLINE_QUEUED_SCANS, beamline_id);
    let str_plans: String = conn.get(get_id).expect("{msg: \"Error getting queued scans\"}");
    Ok(str_plans) 
}


#[axum_macros::debug_handler]
pub async fn get_beamline_worker_task_queue_waiting(
    Path(beamline_id): Path<String>,
    Query(params) : Query<HashMap<String ,isize>>,
    State(state): State<appstate::AppState>,
    //claims: auth::Claims
) -> Result<Json<Vec< BeamlineCommand >>, (StatusCode, String)> 
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
    let range_start = params.get("range_start").copied().unwrap_or(0); // get last 10
    let range_end = params.get("range_end").copied().unwrap_or(-1); 
    let mut conn = state.redis_client.get_connection().unwrap();
    let get_id = format!("{}{}", defines::KEY_TASK_QUEUE_WAITING, beamline_id);
    let items: Vec<String> = conn.lrange(get_id, range_start, range_end).expect("Error getting task queue waiting");
    let mut beamline_queue: Vec<BeamlineCommand> = Vec::new();
    for val in items.iter() 
    {
        let ll: BeamlineCommand = serde_json::from_str(val).expect("Error parsing beamline command.");
        beamline_queue.push(ll);
    }
    Ok(Json(beamline_queue)) 
}

#[axum_macros::debug_handler]
pub async fn get_beamline_worker_task_queue_processing(
    Path(beamline_id): Path<String>,
    Query(params) : Query<HashMap<String ,isize>>,
    State(state): State<appstate::AppState>,
    //claims: auth::Claims
) -> Result<Json<Vec< BeamlineCommand >>, (StatusCode, String)> 
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
    let range_start = params.get("range_start").copied().unwrap_or(0);
    let range_end = params.get("range_end").copied().unwrap_or(-1); 
    let mut conn = state.redis_client.get_connection().unwrap();
    let get_id = format!("{}{}", defines::KEY_TASK_QUEUE_PROCESSING, beamline_id);
    let items: Vec<String> = conn.lrange(get_id, range_start, range_end).expect("Error getting task queue waiting");
    let mut beamline_queue: Vec<BeamlineCommand> = Vec::new();
    for val in items.iter() 
    {
        let ll: BeamlineCommand = serde_json::from_str(val).expect("Error parsing beamline command.");
        beamline_queue.push(ll);
    }
    Ok(Json(beamline_queue)) 
}

#[axum_macros::debug_handler]
pub async fn get_beamline_worker_task_queue_done(
    Path(beamline_id): Path<String>,
    Query(params) : Query<HashMap<String ,isize>>,
    State(state): State<appstate::AppState>,
    //claims: auth::Claims
) -> Result<Json<Vec< BeamlineCommand >>, (StatusCode, String)> 
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
    let range_start = params.get("range_start").copied().unwrap_or(-10); // get last 10
    let range_end = params.get("range_end").copied().unwrap_or(-1); 
    let mut conn = state.redis_client.get_connection().unwrap();
    let get_id = format!("{}{}", defines::KEY_TASK_QUEUE_DONE, beamline_id);
    let items: Vec<String> = conn.lrange(get_id, range_start, range_end).expect("Error getting task queue done");
    let mut beamline_queue: Vec<BeamlineCommand> = Vec::new();
    for val in items.iter() 
    {
        let ll: BeamlineCommand = serde_json::from_str(val).expect("Error parsing beamline command.");
        beamline_queue.push(ll);
    }
    Ok(Json(beamline_queue)) 
}

#[axum_macros::debug_handler]
pub async fn get_beamline_worker_task_queues(
    Path(beamline_id): Path<String>,
    Query(params) : Query<HashMap<String ,isize>>,
    State(state): State<appstate::AppState>,
    //claims: auth::Claims
) -> Result<Json<BeamlineTaskQueues>, (StatusCode, String)> 
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
    let range_start = params.get("range_start").copied().unwrap_or(0);
    let range_end = params.get("range_end").copied().unwrap_or(-1); 
    let mut conn = state.redis_client.get_connection().unwrap();

    let get_wait_id = format!("{}{}", defines::KEY_TASK_QUEUE_WAITING, beamline_id);
    let queued_items: Vec<String> = conn.lrange(get_wait_id, range_start, range_end).expect("Error getting task queue queued");
    let get_proc_id = format!("{}{}", defines::KEY_TASK_QUEUE_PROCESSING, beamline_id);
    let proc_items: Vec<String> = conn.lrange(get_proc_id, range_start, range_end).expect("Error getting task queue processing");
    let get_done_id = format!("{}{}", defines::KEY_TASK_QUEUE_DONE, beamline_id);
    let done_items: Vec<String> = conn.lrange(get_done_id, range_start, range_end).expect("Error getting task queue done");
    let mut beamline_queues: BeamlineTaskQueues = BeamlineTaskQueues::new(&beamline_id);
    for val in queued_items.iter() 
    {
        let ll: BeamlineCommand = serde_json::from_str(val).expect("Error parsing beamline task queues.");
        beamline_queues.queued.push(ll);
    }
    for val in proc_items.iter() 
    {
        let ll: BeamlineCommand = serde_json::from_str(val).expect("Error parsing beamline task queues.");
        beamline_queues.processing.push(ll);
    }
    for val in done_items.iter() 
    {
        let ll: BeamlineCommand = serde_json::from_str(val).expect("Error parsing beamline task queues.");
        beamline_queues.done.push(ll);
    }
    Ok(Json(beamline_queues)) 
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
    let get_id = format!("{}{}", defines::KEY_BEAMLINE_SCAN_LOGS, beamline_id);
    let items: Vec<String> = conn.lrange(get_id, range_start, range_end).expect("Error getting logs");
    let mut beamline_logs: Vec<LogLine> = Vec::new();
    for val in items.iter() 
    {
        let ll: LogLine = serde_json::from_str(val).expect("Error parsing log line.");
        beamline_logs.push(ll);
    }
    Ok(Json(beamline_logs)) 
}

#[axum_macros::debug_handler]
pub async fn get_beamline_worker_heartbeat(
    Path(beamline_id): Path<String>,
    State(state): State<appstate::AppState>,
    //claims: auth::Claims
) -> Result<Json<String>, (StatusCode, String)>
{
    let mut conn = state.redis_client.get_connection().unwrap();
    let get_id = format!("{}{}", defines::KEY_WORKER_HEARTBEAT, beamline_id);
    let heartbeat: String = conn.get(get_id).expect("Error getting logs");
    Ok(Json(heartbeat))
}

// XRF streaming cache -------------------------------------------------------------------------------------------------

/// One element's 2-D XRF map read from the on-disk zarr cache, plus the lists of
/// datasets and elements available so the frontend can offer dropdowns. `data` is
/// row-major (`height` rows of `width` values); never-written pixels (NaN on disk)
/// are serialized as `null`.
#[derive(Serialize)]
pub struct XrfMapResponse
{
    pub dataset: String,
    pub element: String,
    pub width: u64,
    pub height: u64,
    pub datasets: Vec<String>,
    pub elements: Vec<String>,
    pub data: Vec<Option<f64>>,
}

/// Read a single element's live XRF map from the beamline's streaming cache on disk.
///
/// The cache directory (resolved from the in-memory `streaming_info` map by the
/// beamline's acronym) holds one zarr store per scan at `<dataset>.zarr`, each with
/// one 2-D `f64` array per fitted element. Optional `?dataset=` / `?element=` query
/// params select which to return; they default to the most recently modified dataset
/// and its first element. The actual zarr/filesystem reads are blocking, so they run
/// on a blocking thread.
#[axum_macros::debug_handler]
pub async fn get_xrf_streaming_cache(
    Path(beamline_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<appstate::AppState>,
    //claims: auth::Claims
) -> Result<Json<XrfMapResponse>, (StatusCode, String)>
{
    let dir = match state.streaming_cache.get(&beamline_id)
    {
        Some(d) => d.clone(),
        None => return Err((StatusCode::NOT_FOUND, format!("No streaming cache configured for beamline {}", beamline_id))),
    };
    let req_dataset = params.get("dataset").cloned();
    let req_element = params.get("element").cloned();

    let result = tokio::task::spawn_blocking(move ||
    {
        read_xrf_map(&dir, req_dataset.as_deref(), req_element.as_deref())
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("XRF read task failed: {}", e)))?;

    result.map(Json)
}

/// Blocking worker for [`get_xrf_streaming_cache`]: enumerate the `<dataset>.zarr`
/// stores and element arrays, pick the requested (or default) ones, and read the
/// chosen element's whole 2-D map off disk. Errors carry an HTTP status so the
/// handler can forward them directly.
fn read_xrf_map(
    dir: &str,
    req_dataset: Option<&str>,
    req_element: Option<&str>,
) -> Result<XrfMapResponse, (StatusCode, String)>
{
    use std::fs;
    use std::path::PathBuf;
    use std::time::SystemTime;

    let err500 = |msg: String| (StatusCode::INTERNAL_SERVER_ERROR, msg);

    // --- enumerate the <dataset>.zarr stores in the cache directory ---
    let entries = fs::read_dir(dir)
        .map_err(|e| err500(format!("reading cache dir {}: {}", dir, e)))?;
    let mut stores: Vec<(String, PathBuf, SystemTime)> = Vec::new();
    for entry in entries.flatten()
    {
        let path = entry.path();
        if !path.is_dir() { continue; }
        let name = match path.file_name().and_then(|n| n.to_str())
        {
            Some(n) if n.ends_with(".zarr") => n.trim_end_matches(".zarr").to_string(),
            _ => continue,
        };
        let mtime = entry.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
        stores.push((name, path, mtime));
    }
    if stores.is_empty()
    {
        return Err((StatusCode::NOT_FOUND, format!("No XRF datasets found in {}", dir)));
    }
    let mut datasets: Vec<String> = stores.iter().map(|(n, _, _)| n.clone()).collect();
    datasets.sort();

    // --- choose the dataset: requested, else most recently modified ---
    let (dataset, store_path) = match req_dataset
    {
        Some(req) => stores.iter().find(|(n, _, _)| n == req)
            .map(|(n, p, _)| (n.clone(), p.clone()))
            .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Dataset {} not found in {}", req, dir)))?,
        None =>
        {
            let newest = stores.iter().max_by_key(|(_, _, t)| *t).unwrap();
            (newest.0.clone(), newest.1.clone())
        }
    };

    // --- enumerate elements: each is a child dir of the store root holding its row
    //     chunks under a `c/` directory. The store carries no zarr metadata, so the
    //     raw chunk files are read directly (see `read_element_grid`). ---
    let mut elements: Vec<String> = fs::read_dir(&store_path)
        .map_err(|e| err500(format!("reading store {}: {}", store_path.display(), e)))?
        .flatten()
        .filter(|e| { let p = e.path(); p.is_dir() && p.join("c").is_dir() })
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    elements.sort();
    if elements.is_empty()
    {
        return Err((StatusCode::NOT_FOUND, format!("No elements found in dataset {}", dataset)));
    }
    let element = match req_element
    {
        Some(req) => elements.iter().find(|e| e.as_str() == req).cloned()
            .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Element {} not found in dataset {}", req, dataset)))?,
        None => elements[0].clone(),
    };

    // --- read the chosen element's row chunks into a 2-D grid ---
    let (height, width, data) = read_element_grid(&store_path.join(&element))?;

    Ok(XrfMapResponse { dataset, element, width, height, datasets, elements, data })
}

/// Read one element's live map from its raw zarr chunks. The element directory holds
/// `c/<row>/<col>` files, each a run of little-endian `f64` values for that row
/// segment (the store has no metadata, so there is nothing else to consult). Returns
/// `(height, width, data)` with `data` row-major and never-written pixels as `None`.
/// `height` is one past the highest row index present; `width` is the widest row.
fn read_element_grid(elem_dir: &std::path::Path)
    -> Result<(u64, u64, Vec<Option<f64>>), (StatusCode, String)>
{
    use std::fs;
    let err500 = |msg: String| (StatusCode::INTERNAL_SERVER_ERROR, msg);

    let c_dir = elem_dir.join("c");
    // Numeric row-chunk directories under `c/` (rows may be sparse for a live scan).
    let mut rows: Vec<u64> = fs::read_dir(&c_dir)
        .map_err(|e| err500(format!("reading {}: {}", c_dir.display(), e)))?
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|n| n.parse::<u64>().ok())
        .collect();
    if rows.is_empty()
    {
        return Err((StatusCode::NOT_FOUND, format!("No rows found in {}", elem_dir.display())));
    }
    rows.sort();

    // Decode each row's chunk file(s) into a Vec<f64>, tracking the widest row. A row
    // may in principle be split across column chunks (`0`, `1`, ...); concatenate them
    // in numeric order, though in practice there is a single chunk per row.
    let mut decoded: Vec<(u64, Vec<f64>)> = Vec::with_capacity(rows.len());
    let mut width = 0usize;
    for row in &rows
    {
        let row_dir = c_dir.join(row.to_string());
        let mut cols: Vec<u64> = match fs::read_dir(&row_dir)
        {
            Ok(rd) => rd.flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .filter_map(|n| n.parse::<u64>().ok())
                .collect(),
            Err(_) => continue,
        };
        cols.sort();

        let mut vals: Vec<f64> = Vec::new();
        for col in cols
        {
            let bytes = match fs::read(row_dir.join(col.to_string())) { Ok(b) => b, Err(_) => continue };
            for k in (0..bytes.len()).step_by(8)
            {
                if k + 8 <= bytes.len()
                {
                    vals.push(f64::from_le_bytes([
                        bytes[k], bytes[k + 1], bytes[k + 2], bytes[k + 3],
                        bytes[k + 4], bytes[k + 5], bytes[k + 6], bytes[k + 7],
                    ]));
                }
            }
        }
        if vals.len() > width { width = vals.len(); }
        decoded.push((*row, vals));
    }
    if width == 0
    {
        return Err((StatusCode::NOT_FOUND, format!("No chunk data in {}", elem_dir.display())));
    }

    let height = rows.last().copied().unwrap() + 1;
    let mut data: Vec<Option<f64>> = vec![None; (height as usize) * width];
    for (row, vals) in decoded
    {
        let base = (row as usize) * width;
        for (c, v) in vals.into_iter().enumerate()
        {
            if c < width
            {
                data[base + c] = if v.is_nan() { None } else { Some(v) };
            }
        }
    }

    Ok((height, width as u64, data))
}