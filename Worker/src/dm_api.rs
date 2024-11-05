mod defines{
    include!("defines.rs");
}

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Error;
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
    let env_dm_url = std::env::var(defines::ENV_DM_URL).expect(format!("{} must be set",defines::ENV_DM_URL).as_str());
    let env_dm_user = std::env::var(defines::ENV_DM_USER).expect(format!("{} must be set",defines::ENV_DM_USER).as_str());
    let env_dm_pass = std::env::var(defines::ENV_DM_PASS).expect(format!("{} must be set",defines::ENV_DM_PASS).as_str());
    let env_dm_auth = std::env::var(defines::ENV_DM_AUTH).expect(format!("{} must be set",defines::ENV_DM_AUTH).as_str());
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