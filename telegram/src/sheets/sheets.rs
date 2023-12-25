use chrono_tz::Europe::Belgrade;
use chrono_tz::Tz;
use google_sheets4 as sheets4;
use serde_json::json;
use sheets4::{
    chrono::DateTime,
    chrono::Utc,
    hyper::{self, client::HttpConnector},
    hyper_rustls::{self, HttpsConnector},
    oauth2::ServiceAccountAuthenticator,
    Sheets,
};
use std::env;

use crate::sheets::service_acc_data;

pub fn get_cet_date() -> DateTime<Tz> {
    let now = Utc::now();
    let now_serbia = now.with_timezone(&Belgrade);

    now_serbia
}

// Returns month and year in format: 01.2023
fn get_number_of_month_and_year() -> String {
    let date = get_cet_date();

    let result = date.format("%m.%Y").to_string();

    result
}

fn get_current_row_by_day() -> i32 {
    let date = get_cet_date();

    let day = date
        .format("%d")
        .to_string()
        .parse::<i32>()
        .unwrap_or_default();

    day + 1
}

pub async fn get_client() -> Sheets<HttpsConnector<HttpConnector>> {
    let creds = service_acc_data::get_creds_from_env();
    let sa = ServiceAccountAuthenticator::builder(creds)
        .build()
        .await
        .expect("There was an error, trying to build connection with authenticator");

    let hub = Sheets::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
        sa,
    );

    hub
}

pub async fn write_new_spend(sum: i32, description: String) {
    let sheets = get_client().await;

    let spreadsheet_id = env::var("GOOGLE_SHEET_ID").expect("GOOGLE_SHEET_ID not set");

    let table_name = get_number_of_month_and_year();
    let current_row = get_current_row_by_day();
    let range = format!("{}!H{}:I{}", table_name, current_row, current_row);

    let current_day_values = sheets
        .spreadsheets()
        .values_get(&spreadsheet_id, &range)
        .value_render_option("FORMULA")
        .doit()
        .await;

    let values = match current_day_values {
        Ok(v) => v.1.values.unwrap_or_default(),
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    };

    let default_values = vec![json!(""), json!("")];

    let un_values = values.get(0).unwrap_or(&default_values);

    let current_desc = un_values
        .get(0)
        .unwrap_or(&json!(""))
        .to_string()
        .replace("\"", "");
    let current_sum = un_values
        .get(1)
        .unwrap_or(&json!(""))
        .to_string()
        .replace("\"", "");

    let new_desc: String = match current_desc.as_str() {
        "" => description,
        _ => format!("{}++{}", current_desc, description),
    };

    let new_sum: String = match current_sum.as_str() {
        "" => format!("={}", sum),
        _ => format!("{}+{}", current_sum, sum),
    };

    let new_values = vec![vec![json!(new_desc), json!(new_sum)]];

    let value_range = sheets4::api::ValueRange {
        range: Some(range.to_string()),
        major_dimension: Some("ROWS".to_string()),
        values: Some(new_values),
    };

    let result = sheets
        .spreadsheets()
        .values_update(value_range, &spreadsheet_id, &range)
        .value_input_option("USER_ENTERED")
        .doit()
        .await;

    match result {
        Ok(_) => println!("Success!"),
        Err(e) => println!("Error: {:?}", e),
    }
}
