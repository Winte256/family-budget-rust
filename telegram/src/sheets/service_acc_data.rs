use base64::{engine::general_purpose, Engine as _};
use google_sheets4::oauth2::ServiceAccountKey;
use std::env;

pub fn get_creds_from_env() -> ServiceAccountKey {
    let encoded = env::var("GOOGLE_ACCOUNT_CREDS").expect("GOOGLE_ACCOUNT_CREDS not found");

    // decode Base64
    let bytes = general_purpose::STANDARD.decode(encoded).unwrap();

    let decoded_str = String::from_utf8(bytes).expect("wrong utf8 string");

    // deserialize JSON
    let json: Result<ServiceAccountKey, serde_json::Error> = serde_json::from_str(&decoded_str);

    json.unwrap()
}
