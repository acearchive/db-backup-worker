use worker::{self, event, Env, ScheduleContext, ScheduledEvent};

const CLOUDFLARE_ACCOUNT_ID_VAR: &str = "ACCOUNT_ID";
const CLOUDFLARE_API_TOKEN_VAR: &str = "API_TOKEN";
const DB_ID_VAR: &str = "DB_ID";
const R2_BINDING: &str = "R2";

#[event(scheduled)]
async fn scheduled(_event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    console_error_panic_hook::set_once();
}
