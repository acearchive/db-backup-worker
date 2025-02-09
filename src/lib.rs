#![deny(unsafe_code)]

mod api;

use api::ApiClient;
use worker::{self, console_error, console_log, event, Env, ScheduleContext, ScheduledEvent};

const CLOUDFLARE_ACCOUNT_ID_VAR: &str = "ACCOUNT_ID";
const CLOUDFLARE_API_TOKEN_VAR: &str = "API_TOKEN";
const DB_ID_VAR: &str = "DB_ID";
const R2_BINDING: &str = "R2";

async fn run(env: Env) -> anyhow::Result<()> {
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

    console_log!("foobar");

    client.start_db_export(&db_id.to_string()).await?;

    Ok(())
}

#[event(scheduled)]
async fn scheduled(_event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    console_error_panic_hook::set_once();

    if let Err(e) = run(env).await {
        console_error!("Error: {e}");
    }
}
