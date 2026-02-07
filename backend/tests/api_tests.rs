use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    Router,
};
use serde_json::{json, Value};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower::util::ServiceExt;

use kanban_board::handlers::projects::{AppState, create_project, list_projects, get_project, update_project, delete_project};
use kanban_board::middleware::api_key_auth;
use axum::{
    routing::{get, post, put, delete},
    middleware::from_fn,
};

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .expect("Failed to create in-memory database");

    let schema = include_str!("../schema.sql");
    
    for statement in schema.split(';').map(|s| s.trim()).filter(|s| !s.is_empty() && !s.starts_with("PRAGMA")) {
        sqlx::raw_sql(statement).execute(&pool).await.expect("Failed to execute schema");
    }

    pool
}

fn create_test_router(pool: SqlitePool) -> Router {
    let state = AppState { db: pool };

    let protected_routes = Router::new()
        .route("/api/projects", post(create_project))
        .route("/api/projects", get(list_projects))
        .route("/api/projects/{id}", get(get_project))
        .route("/api/projects/{id}", put(update_project))
        .route("/api/projects/{id}", delete(delete_project))
        .with_state(state.clone())
        .layer(from_fn(api_key_auth));

    Router::new()
        .route("/health", get(health_check))
        .merge(protected_routes)
        .with_state(state)
}

async fn health_check() -> axum::Json<Value> {
    axum::Json(json!({ "status": "ok" }))
}

#[tokio::test]
async fn test_health_endpoint() {
    let pool = setup_test_db().await;
    let app = create_test_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(body_json["status"], "ok");
}

#[tokio::test]
async fn test_auth_middleware_missing_api_key() {
    std::env::set_var("KANBAN_API_KEY", "test-secret-key");
    
    let pool = setup_test_db().await;
    let app = create_test_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/projects")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(body_json["error"], "Invalid or missing API key");
}

#[tokio::test]
async fn test_auth_middleware_wrong_api_key() {
    std::env::set_var("KANBAN_API_KEY", "test-secret-key");
    
    let pool = setup_test_db().await;
    let app = create_test_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/projects")
                .header("X-API-Key", "wrong-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(body_json["error"], "Invalid or missing API key");
}

#[tokio::test]
async fn test_auth_middleware_valid_api_key() {
    std::env::set_var("KANBAN_API_KEY", "test-secret-key");
    
    let pool = setup_test_db().await;
    let app = create_test_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/projects")
                .header("X-API-Key", "test-secret-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_project() {
    std::env::set_var("KANBAN_API_KEY", "test-secret-key");
    
    let pool = setup_test_db().await;
    let app = create_test_router(pool);

    let request_body = json!({
        "name": "Test Project"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/projects")
                .header("X-API-Key", "test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(body_json["name"], "Test Project");
    assert!(body_json["id"].is_number());
    assert!(body_json["created_at"].is_string());
}

#[tokio::test]
async fn test_list_projects() {
    std::env::set_var("KANBAN_API_KEY", "test-secret-key");
    
    let pool = setup_test_db().await;
    
    sqlx::query!("INSERT INTO projects (name) VALUES (?)", "Project 1")
        .execute(&pool)
        .await
        .unwrap();
    
    sqlx::query!("INSERT INTO projects (name) VALUES (?)", "Project 2")
        .execute(&pool)
        .await
        .unwrap();

    let app = create_test_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/projects")
                .header("X-API-Key", "test-secret-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    
    assert!(body_json.is_array());
    let projects = body_json.as_array().unwrap();
    assert_eq!(projects.len(), 2);
    
    let names: Vec<&str> = projects.iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Project 1"));
    assert!(names.contains(&"Project 2"));
}

#[tokio::test]
async fn test_get_project() {
    std::env::set_var("KANBAN_API_KEY", "test-secret-key");
    
    let pool = setup_test_db().await;
    
    let project = sqlx::query!("INSERT INTO projects (name) VALUES (?) RETURNING id", "Test Project")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    let project_id = project.id;

    let app = create_test_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/projects/{}", project_id))
                .header("X-API-Key", "test-secret-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(body_json["id"], project_id);
    assert_eq!(body_json["name"], "Test Project");
    assert!(body_json["columns"].is_array());
}

#[tokio::test]
async fn test_get_project_not_found() {
    std::env::set_var("KANBAN_API_KEY", "test-secret-key");
    
    let pool = setup_test_db().await;
    let app = create_test_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/projects/999")
                .header("X-API-Key", "test-secret-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(body_json["error"], "Project not found");
}

#[tokio::test]
async fn test_update_project() {
    std::env::set_var("KANBAN_API_KEY", "test-secret-key");
    
    let pool = setup_test_db().await;
    
    let project = sqlx::query!("INSERT INTO projects (name) VALUES (?) RETURNING id", "Old Name")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    let project_id = project.id;

    let app = create_test_router(pool);

    let request_body = json!({
        "name": "New Name"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/api/projects/{}", project_id))
                .header("X-API-Key", "test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(body_json["id"], project_id);
    assert_eq!(body_json["name"], "New Name");
}

#[tokio::test]
async fn test_update_project_not_found() {
    std::env::set_var("KANBAN_API_KEY", "test-secret-key");
    
    let pool = setup_test_db().await;
    let app = create_test_router(pool);

    let request_body = json!({
        "name": "New Name"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/projects/999")
                .header("X-API-Key", "test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(body_json["error"], "Project not found");
}

#[tokio::test]
async fn test_delete_project() {
    std::env::set_var("KANBAN_API_KEY", "test-secret-key");
    
    let pool = setup_test_db().await;
    
    let project = sqlx::query!("INSERT INTO projects (name) VALUES (?) RETURNING id", "To Delete")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    let project_id = project.id;

    let app = create_test_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/projects/{}", project_id))
                .header("X-API-Key", "test-secret-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_project_not_found() {
    std::env::set_var("KANBAN_API_KEY", "test-secret-key");
    
    let pool = setup_test_db().await;
    let app = create_test_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/projects/999")
                .header("X-API-Key", "test-secret-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(body_json["error"], "Project not found");
}

#[tokio::test]
async fn test_create_project_with_default_columns() {
    std::env::set_var("KANBAN_API_KEY", "test-secret-key");
    
    let pool = setup_test_db().await;
    
    let request_body = json!({
        "name": "Project with Columns"
    });

    let app = create_test_router(pool.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/projects")
                .header("X-API-Key", "test-secret-key")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body).unwrap();
    let project_id = body_json["id"].as_i64().unwrap();

    let columns = sqlx::query!("SELECT name FROM columns WHERE project_id = ? ORDER BY sort_order", project_id)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(columns.len(), 5);
    assert_eq!(columns[0].name, "Backlog");
    assert_eq!(columns[1].name, "To Do");
    assert_eq!(columns[2].name, "In Progress");
    assert_eq!(columns[3].name, "Testing");
    assert_eq!(columns[4].name, "Done");
}
