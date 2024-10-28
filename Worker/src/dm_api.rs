mod defines{
    include!("defines.rs");
}

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Error;
use std::env;
use serde::{Deserialize, Serialize};
//use serde_json::Result;

#[derive(Serialize, Deserialize)]
struct LoginUser {
    username: String,
    password: String
}


#[tokio::main]
pub async fn get_session_id() -> Result<(), Error> 
{
    let seenv_dm_urlcret = std::env::var(defines::ENV_DM_URL).expect(defines::ENV_DM_URL+" must be set");
    let env_dm_user = std::env::var(defines::ENV_DM_USER).expect(defines::ENV_DM_USER+" must be set");
    let env_dm_pass = std::env::var(defines::ENV_DM_PASS).expect(defines::ENV_DM_PASS+" must be set");
    let env_dm_auth = std::env::var(defines::ENV_DM_AUTH).expect(defines::ENV_DM_AUTH+" must be set");
    println!("Logging in user {}\n", env_dm_user);
    println!("URL: {}\n", env_dm_url);

    let auth_str: String = String::from("Basic ") + &env_dm_auth;
    
    let login_user = LoginUser 
    {
        username: env_dm_user,
        password: env_dm_pass,
    };
    let body_str = serde_json::to_string(&login_user).unwrap();
    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("my-data-agent/1.0"));
    headers.insert("Content-Type", HeaderValue::from_static("application/x-www-form-urlencoded"));
    headers.insert("Authorization", HeaderValue::from_str(&auth_str).expect("REASON"));
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(true)
        .build()?;

    let response = client.post(env_dm_url).body(body_str).send().await?;

    println!("Status: {}", response.status());
    println!("Headers:\n{:#?}", response.headers());

    let body_result = response.text().await?;
    println!("Body:\n{}", body_result);

    Ok(())
}