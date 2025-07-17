#![deny(unsafe_code)]

mod api;

use api::{ApiClient, DbExport, DbExportBookmark};
use chrono::{DateTime, SecondsFormat};
use worker::{self, console_error, console_log, event, Date, Env, ScheduleContext, ScheduledEvent};

const CLOUDFLARE_ACCOUNT_ID_VAR: &str = "ACCOUNT_ID";
const CLOUDFLARE_API_TOKEN_VAR: &str = "API_TOKEN";
const DB_ID_VAR: &str = "DB_ID";
const R2_BINDING: &str = "R2";

fn db_backup_key() -> String {
    // Because we're targeting WASM, we need to use the datetime functionality provided by the
    // Workers runtime.
    let millis_since_epoch = Date::now().as_millis();

    let timestamp = DateTime::from_timestamp_millis(millis_since_epoch as i64)
        .expect("DateTime milliseconds since epoch is out of range.")
        .to_rfc3339_opts(SecondsFormat::Secs, true);

    format!("db-backup-{timestamp}.sql")
}

async fn run(env: Env) -> anyhow::Result<()> {
    let backup_key = db_backup_key();

    let account_id = env
        .var(CLOUDFLARE_ACCOUNT_ID_VAR)
        .expect("Missing account ID environment variable");

    let api_token = env
        .var(CLOUDFLARE_API_TOKEN_VAR)
        .expect("Missing API token environment variable");

    let db_id = env
        .var(DB_ID_VAR)
        .expect("Missing DB ID environment variable");

    let client = ApiClient::new(api_token.to_string(), account_id.to_string());

    console_log!("Starting DB export...");

    let mut export_bookmark: Option<DbExportBookmark> = None;

    let signed_url = loop {
        let DbExport {
            signed_url,
            bookmark,
        } = client
            .poll_db_export(&db_id.to_string(), export_bookmark.clone())
            .await?;

        export_bookmark = Some(bookmark);

        if let Some(url) = signed_url {
            break url;
        } else {
            console_log!("Continuing to poll for DB export...");
        }
    };

    let db_export_resp = reqwest::get(signed_url).await?.error_for_status()?;
    let db_export_bytes = db_export_resp.bytes().await?;

    console_log!("Uploading DB export to R2: {backup_key}");

    let bucket = env.bucket(R2_BINDING)?;
    bucket
        .put(backup_key, db_export_bytes.to_vec())
        .execute()
        .await?;

    Ok(())
}

#[event(scheduled)]
async fn scheduled(_event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    console_error_panic_hook::set_once();

    if let Err(e) = run(env).await {
        console_error!("Error: {e}");
    }
}
