//! Run with
//!
//! ```not_rust
//! cargo run -p mydata-backend
//! kill or ctrl-c
//! ```

use std::time::Duration;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, RequestPartsExt, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::time::sleep;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::env;
use ldap3::{LdapConnAsync, Scope, SearchEntry};
//use ldap3::result::Result;
use ::bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;

use bb8_redis::bb8;

static KEYS: Lazy<Keys> = Lazy::new(|| 
{
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

#[tokio::main]
async fn main() 
{
    // Enable tracing.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| 
                {
                "mydata_backend=debug,tower_http=debug,axum=trace".into()
                }),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let manager = RedisConnectionManager::new("redis://localhost").unwrap();
    let pool = bb8::Pool::builder().build(manager).await.unwrap();

    {
        // ping the database before starting
        let mut conn = pool.get().await.unwrap();
        //let keys : Vec<String> = con.hkeys("access_token:*")?;
        ///conn.keys::<&str,()> ("access_token:*")
        conn.set::<&str, &str, ()>("foo", "bar").await.unwrap();
        let result: String = conn.get("foo").await.unwrap();
        assert_eq!(result, "bar");
    }
    tracing::debug!("successfully connected to redis and pinged it");

    // Create a regular axum app.
    let app = Router::new()
        .route("/slow", get(|| sleep(Duration::from_secs(5))))
        .route("/forever", get(std::future::pending::<()>))
        .route("/protected", get(protected))
        .route("/authorize", post(authorize))
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
        ));

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());    

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() 
{
    let ctrl_c = async 
    {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async 
    {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! 
    {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn protected(claims: Claims) -> Result<String, AuthError> 
{
    // Send the protected data to the user
    Ok(format!(
        "Welcome to the protected area :)\nYour data:\n{claims}",
    ))
}

async fn auth(username: &str, password: &str, claims: &mut Claims) -> Result<bool, ldap3::result::LdapError> 
{
    let mut dn = String::new();
    let svc_user = env::var("SVC_USER").unwrap();
    let svc_pass = env::var("SVC_PASS").unwrap();
    let ad_url = env::var("AD_URL").unwrap();
    let ad_search_dn = env::var("AD_SEARCH_DN").unwrap();
    let mut ad_filter = String::new();
    ad_filter.push_str("(&(objectClass=person)(cn=");
    ad_filter.push_str(username);
    ad_filter.push_str("*))");
    println!("ad_url {}", ad_url);
    let (conn, mut ldap) = LdapConnAsync::new(&ad_url).await?;
    ldap3::drive!(conn);
    let _ = ldap.simple_bind(&svc_user, &svc_pass).await.unwrap();
    let mut stream = ldap
        .streaming_search(
            &ad_search_dn,
            Scope::Subtree,
            &ad_filter,
            vec!["cn", "sn", "mail", "employeeNumber"]
        )
        .await?;
    
    while let Some(entry) = stream.next().await? 
    {
        let se = SearchEntry::construct(entry);
        println!("{:?}", se);
        dn = se.dn;
        
        
        claims.employeeID = se.attrs["employeeNumber"][0].to_owned();
        claims.mail = se.attrs["mail"][0].to_owned();
        claims.sn = se.attrs["sn"][0].to_owned();
        // Mandatory expiry time as UTC timestamp
        claims.exp = 2000000000; // May 2033

        break;
    }
    let _res = stream.finish().await;
    let msgid = stream.ldap_handle().last_id();
    ldap.abandon(msgid).await?;

    let res = ldap.simple_bind(&dn, &password).await.unwrap();
    let _ = ldap.unbind();
    if res.rc == 0
    //if eq
    {
        Ok(true)
    }
    else 
    {
        Ok(false)
    }
}

async fn authorize(Json(payload): Json<AuthPayload>) -> Result<Json<AuthBody>, AuthError> 
{
    // Check if the user sent the credentials
    if payload.client_id.is_empty() || payload.client_secret.is_empty() 
    {
        return Err(AuthError::MissingCredentials);
    }
    let mut claims = Claims
    {
        employeeID : "0".to_owned(),
        mail : "0".to_owned(),
        sn : "0".to_owned(),
        // Mandatory expiry time as UTC timestamp
        exp : 2000000000,

    }; 
    // Here you can check the user credentials from a database
    if false == auth(&payload.client_id, &payload.client_secret, &mut claims).await.unwrap_or(false)
    {
        return Err(AuthError::WrongCredentials);
    }
    //println!("Claims {}", claims);
    // Create the authorization token
    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    // Send the authorized token
    Ok(Json(AuthBody::new(token)))
}

impl Display for Claims 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "Email: {}\nName: {}", self.mail, self.sn)
    }
}

impl AuthBody 
{
    fn new(access_token: String) -> Self 
    {
        Self 
        {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> 
    {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

impl IntoResponse for AuthError 
{
    fn into_response(self) -> Response 
    {
        let (status, error_message) = match self 
        {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

struct Keys 
{
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys 
{
    fn new(secret: &[u8]) -> Self 
    {
        Self 
        {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims 
{
    employeeID: String,
    mail: String,
    sn: String,
    exp: usize,
}

#[derive(Debug, Serialize)]
struct AuthBody 
{
    access_token: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
struct AuthPayload 
{
    client_id: String,
    client_secret: String,
}

#[derive(Debug)]
enum AuthError 
{
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}
