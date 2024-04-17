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
    let env_dm_url: String = match env::var(defines::ENV_DM_URL) {
        Ok(val) => val,
        Err(_) => String::from("none")
    };

    let env_dm_user: String = match env::var(defines::ENV_DM_USER) {
        Ok(val) => val,
        Err(_) => String::from("nobody")
    };
    let env_dm_pass: String = match env::var(defines::ENV_DM_PASS) {
        Ok(val) => val,
        Err(_) => String::from(" ")
    };

    let env_dm_auth: String = match env::var(defines::ENV_DM_AUTH) {
        Ok(val) => val,
        Err(_) => String::from("nobody")
    };
    println!("Logging in user {}\n", env_dm_user);
    println!("URL: {}\n", env_dm_url);

    let auth_str: String = String::from("Basic ") + &env_dm_auth;
    
    let login_user = LoginUser {
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
        .build()?;

    let response = client.post(env_dm_url).body(body_str).send().await?;

    println!("Status: {}", response.status());
    println!("Headers:\n{:#?}", response.headers());

    let body = response.text().await?;
    println!("Body:\n{}", body);

    Ok(())
}