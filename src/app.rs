use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers;
use crate::state::AppState;

pub fn build_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::pages::marketing_page))
        .route("/marketing", get(handlers::pages::marketing_page))
        .route("/app", get(handlers::pages::app_page))
        .route("/api/marco", get(handlers::utility::marco))
        .route("/api/redis-test", get(handlers::utility::redis_test))
        .route(
            "/api/getCurrentBlock",
            get(handlers::blockchain::get_current_block_handler),
        )
        .route("/api/db", post(handlers::db::db_create))
        .route("/api/db", get(handlers::db::db_list))
        .route("/api/db/:id", get(handlers::db::db_read))
        .route("/api/db/:id", put(handlers::db::db_update))
        .route("/api/db/:id", delete(handlers::db::db_delete))
        .route("/api/monitor", get(handlers::monitor::monitor_list))
        .route("/api/monitor/add", post(handlers::monitor::monitor_add))
        .route(
            "/api/monitor/:address",
            delete(handlers::monitor::monitor_remove),
        )
        .route("/api/monitor/alerts", get(handlers::monitor::monitor_alerts))
        .route(
            "/api/monitor/alerts/stream",
            get(handlers::monitor::monitor_alerts_stream),
        )
        .route("/api/docs", get(handlers::docs::api_docs_json))
        .route("/docs", get(handlers::docs::api_docs_html))
        .with_state(app_state)
}


