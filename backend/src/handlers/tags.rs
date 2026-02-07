use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use super::projects::{AppState, ErrorResponse};

#[derive(Debug, Serialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: String,
}

pub async fn list_tags(
    State(state): State<AppState>,
) -> Result<Json<Vec<Tag>>, (StatusCode, Json<ErrorResponse>)> {
    let tags = sqlx::query_as!(
        Tag,
        r#"SELECT id as "id!", name, color FROM tags ORDER BY name"#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch tags: {}", e),
            }),
        )
    })?;

    Ok(Json(tags))
}

pub async fn create_tag(
    State(state): State<AppState>,
    Json(payload): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<Tag>), (StatusCode, Json<ErrorResponse>)> {
    let tag = sqlx::query_as!(
        Tag,
        r#"INSERT INTO tags (name, color) VALUES (?, ?) RETURNING id as "id!", name, color"#,
        payload.name,
        payload.color
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        let status = if e.to_string().contains("UNIQUE constraint failed") {
            StatusCode::CONFLICT
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (
            status,
            Json(ErrorResponse {
                error: format!("Failed to create tag: {}", e),
            }),
        )
    })?;

    Ok((StatusCode::CREATED, Json(tag)))
}

pub async fn delete_tag(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query!(r#"DELETE FROM tags WHERE id = ?"#, id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to delete tag: {}", e),
                }),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Tag not found".to_string(),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}
