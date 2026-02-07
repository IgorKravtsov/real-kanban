use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use super::projects::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkedPath {
    pub id: i64,
    pub project_id: i64,
    pub path: String,
    pub hostname: Option<String>,
    pub default_column_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkedPathRequest {
    pub path: String,
    pub hostname: Option<String>,
    pub default_column_id: Option<i64>,
}

pub async fn list_linked_paths(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<LinkedPath>>, (StatusCode, String)> {
    let paths = sqlx::query_as!(
        LinkedPath,
        r#"SELECT id as "id!", project_id as "project_id!", path, hostname, default_column_id, created_at
           FROM linked_paths
           WHERE project_id = ?
           ORDER BY path"#,
        project_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(paths))
}

pub async fn create_linked_path(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    Json(payload): Json<CreateLinkedPathRequest>,
) -> Result<Json<LinkedPath>, (StatusCode, String)> {
    let result = sqlx::query!(
        r#"INSERT INTO linked_paths (project_id, path, hostname, default_column_id)
           VALUES (?, ?, ?, ?)
           ON CONFLICT(path) DO UPDATE SET
             project_id = excluded.project_id,
             hostname = excluded.hostname,
             default_column_id = excluded.default_column_id"#,
        project_id,
        payload.path,
        payload.hostname,
        payload.default_column_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let id = result.last_insert_rowid();

    let path = sqlx::query_as!(
        LinkedPath,
        r#"SELECT id as "id!", project_id as "project_id!", path, hostname, default_column_id, created_at
           FROM linked_paths
           WHERE id = ? OR path = ?"#,
        id,
        payload.path
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(path))
}

pub async fn delete_linked_path(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query!("DELETE FROM linked_paths WHERE id = ?", id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_linked_path_by_path(
    State(state): State<AppState>,
    Json(payload): Json<DeleteByPathRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query!("DELETE FROM linked_paths WHERE path = ?", payload.path)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct DeleteByPathRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct LookupQuery {
    pub path: String,
    pub hostname: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LookupResponse {
    pub linked_path: LinkedPath,
    pub project_name: String,
}

pub async fn lookup_linked_path(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<LookupQuery>,
) -> Result<Json<LookupResponse>, (StatusCode, String)> {
    let path = &query.path;
    
    let result = sqlx::query!(
        r#"SELECT lp.id, lp.project_id, lp.path, lp.hostname, lp.default_column_id, lp.created_at, p.name as project_name
           FROM linked_paths lp
           JOIN projects p ON p.id = lp.project_id
           WHERE ? LIKE lp.path || '%'
           ORDER BY LENGTH(lp.path) DESC
           LIMIT 1"#,
        path
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match result {
        Some(row) => Ok(Json(LookupResponse {
            linked_path: LinkedPath {
                id: row.id,
                project_id: row.project_id,
                path: row.path,
                hostname: row.hostname,
                default_column_id: row.default_column_id,
                created_at: row.created_at,
            },
            project_name: row.project_name,
        })),
        None => Err((StatusCode::NOT_FOUND, "No linked path found".to_string())),
    }
}
