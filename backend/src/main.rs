mod db;
mod handlers;
mod middleware;

use axum::{
    extract::State,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Serialize;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use axum::middleware::from_fn;

use handlers::projects::{
    AppState, create_project, list_projects, get_project, update_project, delete_project, reorder_projects,
};

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            let data_dir = std::env::current_dir()
                .expect("Failed to get current directory")
                .parent()
                .expect("Failed to get parent directory")
                .join("data");
            std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
            let url = format!("sqlite://{}/kanban.db", data_dir.display());
            tracing::info!("Using database URL: {}", url);
            url
        });

    tracing::info!("Initializing database at: {}", database_url);

    let pool = db::init_db(&database_url)
        .await
        .expect("Failed to initialize database");

    let state = AppState { db: pool };

    let protected_routes = Router::new()
        .route("/api/projects", post(create_project))
        .route("/api/projects", get(list_projects))
        .route("/api/projects/{id}", get(get_project))
        .route("/api/projects/{id}", put(update_project))
        .route("/api/projects/{id}", delete(delete_project))
        .route("/api/projects/reorder", put(reorder_projects))
        .route("/api/projects/{id}/columns", get(handlers::columns::list_columns))
        .route("/api/projects/{id}/columns", post(handlers::columns::create_column))
        .route("/api/projects/{id}/columns/reorder", put(handlers::columns::reorder_columns))
        .route("/api/columns/{id}", put(handlers::columns::update_column))
        .route("/api/columns/{id}", delete(handlers::columns::delete_column))
        .route("/api/projects/{project_id}/tasks", get(handlers::tasks::list_tasks))
        .route("/api/projects/{project_id}/tasks", post(handlers::tasks::create_task))
        .route("/api/tasks/{id}", get(handlers::tasks::get_task))
        .route("/api/tasks/{id}", put(handlers::tasks::update_task))
        .route("/api/tasks/{id}", delete(handlers::tasks::delete_task))
        .route("/api/tasks/bulk-update", put(handlers::tasks::bulk_update_tasks))
        .route("/api/tasks/{id}/subtasks", post(handlers::subtasks::create_subtask))
        .route("/api/subtasks/{id}", put(handlers::subtasks::update_subtask))
        .route("/api/subtasks/{id}", delete(handlers::subtasks::delete_subtask))
        .route("/api/tags", get(handlers::tags::list_tags))
        .route("/api/tags", post(handlers::tags::create_tag))
        .route("/api/tags/{id}", delete(handlers::tags::delete_tag))
        .route("/api/projects/{project_id}/linked-paths", get(handlers::linked_paths::list_linked_paths))
        .route("/api/projects/{project_id}/linked-paths", post(handlers::linked_paths::create_linked_path))
        .route("/api/linked-paths/{id}", delete(handlers::linked_paths::delete_linked_path))
        .route("/api/linked-paths/by-path", delete(handlers::linked_paths::delete_linked_path_by_path))
        .route("/api/linked-paths/lookup", get(handlers::linked_paths::lookup_linked_path))
        .with_state(state.clone())
        .layer(from_fn(middleware::api_key_auth));

    let app = Router::new()
        .route("/health", get(health_check))
        .merge(protected_routes)
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "30100".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect(&format!("Failed to bind to {}", addr));

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

async fn health_check(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}
