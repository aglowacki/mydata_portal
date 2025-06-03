use serde_json::json;
use std::fmt::Display;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use std::env;
use once_cell::sync::Lazy;
use ldap3::{LdapConnAsync, Scope, SearchEntry};


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
pub struct Claims 
{
    employee_id: String,
    mail: String,
    department: String,
    employee_type: String,
    sn: String,
    exp: usize,
}

impl Claims
{
    pub fn get_badge(&self) -> i32
    {
        return self.employee_id.parse::<i32>().unwrap();
    }
}

#[derive(Debug, Serialize)]
pub struct AuthBody 
{
    access_token: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload 
{
    client_id: String,
    client_secret: String,
}

#[derive(Debug)]
pub enum AuthError 
{
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}


impl<S> FromRequestParts<S> for Claims where S: Send + Sync,
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
        
        if token_data.claims.employee_id == "0"
        {
            return Err(AuthError::MissingCredentials);
        }

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

static KEYS: Lazy<Keys> = Lazy::new(|| 
{
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

async fn auth(username: &str, password: &str, claims: &mut Claims) -> Result<bool, ldap3::result::LdapError> 
{
    let mut dn = String::new();
    let svc_user = env::var("SVC_USER").unwrap();
    let svc_pass = env::var("SVC_PASS").unwrap();
    let full_attr_str = env::var("AD_ATTR_VEC").unwrap();
    let attr_parts = full_attr_str.split(":");
    let attrs_vec = attr_parts.collect::<Vec<&str>>();
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
            &attrs_vec
        )
        .await?;
    
    while let Some(entry) = stream.next().await? 
    {
        let se = SearchEntry::construct(entry);
        println!("{:?}", se);
        dn = se.dn;
        
        if se.attrs.contains_key("employeeNumber")
        {
            claims.employee_id = se.attrs["employeeNumber"][0].to_owned();
        }
        else if se.attrs.contains_key("employeeID")
        {
            claims.employee_id = se.attrs["employeeID"][0].to_owned();
        }
        if se.attrs.contains_key("department")
        {
            claims.department = se.attrs["department"][0].to_owned();
        }
        if se.attrs.contains_key("employeeType")
        {
            claims.employee_type =  se.attrs["employeeType"][0].to_owned();
        }
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

pub async fn authorize(Json(payload): Json<AuthPayload>) -> Result<Json<AuthBody>, AuthError> 
{
    // Check if the user sent the credentials
    if payload.client_id.is_empty() || payload.client_secret.is_empty() 
    {
        return Err(AuthError::MissingCredentials);
    }
    let mut claims = Claims
    {
        employee_id : "0".to_owned(),
        mail : "0".to_owned(),
        department : "0".to_owned(),
        employee_type : "0".to_owned(),
        sn : "0".to_owned(),
        // Mandatory expiry time as UTC timestamp
        exp : 2000000000,

    }; 
    // Here you can check the user credentials from a database
    if false == auth(&payload.client_id, &payload.client_secret, &mut claims).await.unwrap_or(false)
    {
        return Err(AuthError::WrongCredentials);
    }
    if claims.employee_id == "0"
    {
        return Err(AuthError::WrongCredentials);
    }
    //println!("Claims {}", claims);
    // Create the authorization token
    let token: String = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    // Send the authorized token
    Ok(Json(AuthBody::new(token)))
}
