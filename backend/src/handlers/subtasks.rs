use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use super::projects::{AppState, ErrorResponse};

#[derive(Debug, Serialize)]
pub struct Subtask {
    pub id: i64,
    pub task_id: i64,
    pub title: String,
    pub done: bool,
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubtaskRequest {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubtaskRequest {
    pub title: Option<String>,
    pub done: Option<bool>,
}

pub async fn create_subtask(
    State(state): State<AppState>,
    Path(task_id): Path<i64>,
    Json(payload): Json<CreateSubtaskRequest>,
) -> Result<(StatusCode, Json<Subtask>), (StatusCode, Json<ErrorResponse>)> {
    let max_sort_order = sqlx::query!(
        r#"SELECT MAX(sort_order) as max_sort FROM subtasks WHERE task_id = ?"#,
        task_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to get max sort order: {}", e),
            }),
        )
    })?;

    let sort_order = max_sort_order.max_sort.unwrap_or(0) + 1000;

    let result = sqlx::query!(
        r#"INSERT INTO subtasks (task_id, title, sort_order) VALUES (?, ?, ?) RETURNING id as "id!""#,
        task_id,
        payload.title,
        sort_order
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create subtask: {}", e),
            }),
        )
    })?;

    let subtask = Subtask {
        id: result.id,
        task_id,
        title: payload.title,
        done: false,
        sort_order,
    };

    Ok((StatusCode::CREATED, Json(subtask)))
}

pub async fn update_subtask(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateSubtaskRequest>,
) -> Result<Json<Subtask>, (StatusCode, Json<ErrorResponse>)> {
    let current = sqlx::query!(
        r#"SELECT id as "id!", task_id as "task_id!", title, done as "done!", sort_order as "sort_order!" FROM subtasks WHERE id = ?"#,
        id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch subtask: {}", e),
            }),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Subtask not found".to_string(),
            }),
        )
    })?;

    let new_title = payload.title.unwrap_or(current.title);
    let new_done = payload.done.unwrap_or(current.done != 0);
    let new_done_int = if new_done { 1 } else { 0 };

    sqlx::query!(
        r#"UPDATE subtasks SET title = ?, done = ? WHERE id = ?"#,
        new_title,
        new_done_int,
        id
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to update subtask: {}", e),
            }),
        )
    })?;

    Ok(Json(Subtask {
        id: current.id,
        task_id: current.task_id,
        title: new_title,
        done: new_done,
        sort_order: current.sort_order,
    }))
}

pub async fn delete_subtask(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query!(r#"DELETE FROM subtasks WHERE id = ?"#, id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to delete subtask: {}", e),
                }),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Subtask not found".to_string(),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}
