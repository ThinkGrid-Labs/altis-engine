use warp::Filter;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    token: String,
}

pub fn routes(jwt_secret: String, expiration_seconds: u64) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let secret = warp::any().map(move || jwt_secret.clone());
    let expiration = warp::any().map(move || expiration_seconds);

    warp::path!("v1" / "auth" / "guest")
        .and(warp::post())
        .and(secret)
        .and(expiration)
        .map(login_guest)
}

fn login_guest(secret: String, expiration_seconds: u64) -> impl warp::Reply {
    let my_claims = Claims {
        sub: format!("guest-{}", Uuid::new_v4()),
        role: "guest".to_owned(),
        exp: (Utc::now() + Duration::seconds(expiration_seconds as i64)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(secret.as_bytes())
    ).expect("Token creation failed");

    warp::reply::json(&AuthResponse { token })
}

pub fn with_auth(jwt_secret: String) -> impl Filter<Extract = (Claims,), Error = warp::Rejection> + Clone {
    let secret_filter = warp::any().map(move || jwt_secret.clone());

    warp::header::<String>("authorization")
        .and(secret_filter)
        .and_then(|auth_header: String, secret: String| async move {
            if !auth_header.starts_with("Bearer ") {
                return Err(warp::reject::custom(AuthError));
            }
            let token = &auth_header[7..];

            let token_data = decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &Validation::new(Algorithm::HS256),
            )
            .map_err(|_| warp::reject::custom(AuthError))?;

            Ok(token_data.claims)
        })
}

#[derive(Debug)]
struct AuthError;
impl warp::reject::Reject for AuthError {}
