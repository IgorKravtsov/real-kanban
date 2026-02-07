use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use super::projects::AppState;

#[derive(Debug, Serialize)]
pub struct Column {
    id: i64,
    project_id: i64,
    name: String,
    sort_order: i64,
    created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateColumnRequest {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateColumnRequest {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct ReorderItem {
    id: i64,
    sort_order: i64,
}

pub async fn list_columns(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<Column>>, StatusCode> {
    let columns = sqlx::query_as!(
        Column,
        r#"
        SELECT id, project_id, name, sort_order, created_at
        FROM columns
        WHERE project_id = ?
        ORDER BY sort_order ASC
        "#,
        project_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(columns))
}

pub async fn create_column(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    Json(payload): Json<CreateColumnRequest>,
) -> Result<Json<Column>, StatusCode> {
    let max_sort_order: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT MAX(sort_order)
        FROM columns
        WHERE project_id = ?
        "#,
    )
    .bind(project_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_sort_order = max_sort_order.unwrap_or(0) + 1000;

    let result = sqlx::query(
        r#"
        INSERT INTO columns (project_id, name, sort_order)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(project_id)
    .bind(&payload.name)
    .bind(new_sort_order)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result_id = result.last_insert_rowid();

    let column = sqlx::query_as!(
        Column,
        r#"
        SELECT id, project_id, name, sort_order, created_at
        FROM columns
        WHERE id = ?
        "#,
        result_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(column))
}

pub async fn update_column(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateColumnRequest>,
) -> Result<Json<Column>, StatusCode> {
    sqlx::query(
        r#"
        UPDATE columns
        SET name = ?
        WHERE id = ?
        "#,
    )
    .bind(&payload.name)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let column = sqlx::query_as!(
        Column,
        r#"
        SELECT id, project_id, name, sort_order, created_at
        FROM columns
        WHERE id = ?
        "#,
        id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(column))
}

pub async fn delete_column(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        r#"
        DELETE FROM columns
        WHERE id = ?
        "#,
    )
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn reorder_columns(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    Json(payload): Json<Vec<ReorderItem>>,
) -> Result<StatusCode, StatusCode> {
    let mut tx = state.db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for item in payload {
        sqlx::query(
            r#"
            UPDATE columns
            SET sort_order = ?
            WHERE id = ? AND project_id = ?
            "#,
        )
        .bind(item.sort_order)
        .bind(item.id)
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
