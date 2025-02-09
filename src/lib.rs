use worker::{self, event, Env, ScheduleContext, ScheduledEvent};

#[event(scheduled)]
async fn scheduled(_event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    console_error_panic_hook::set_once();
}
