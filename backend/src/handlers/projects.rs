use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

#[derive(Debug, Serialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct Column {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Task {
    pub id: i64,
    pub project_id: i64,
    pub column_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub sort_order: i64,
    pub source_tag: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProjectWithDetails {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub columns: Vec<ColumnWithTasks>,
}

#[derive(Debug, Serialize)]
pub struct ColumnWithTasks {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub sort_order: i64,
    pub created_at: String,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), (StatusCode, Json<ErrorResponse>)> {
    let mut tx = state.db.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to start transaction: {}", e),
            }),
        )
    })?;

    let project = sqlx::query_as!(
        Project,
        r#"INSERT INTO projects (name, sort_order) VALUES (?, (SELECT COALESCE(MAX(sort_order), 0) + 1000 FROM projects)) RETURNING id, name, sort_order, created_at"#,
        payload.name
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create project: {}", e),
            }),
        )
    })?;

    let default_columns = [
        ("Backlog", 1000),
        ("To Do", 2000),
        ("In Progress", 3000),
        ("Testing", 4000),
        ("Done", 5000),
    ];

    for (name, sort_order) in default_columns {
        sqlx::query!(
            r#"INSERT INTO columns (project_id, name, sort_order) VALUES (?, ?, ?)"#,
            project.id,
            name,
            sort_order
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to create default columns: {}", e),
                }),
            )
        })?;
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to commit transaction: {}", e),
            }),
        )
    })?;

    Ok((StatusCode::CREATED, Json(project)))
}

pub async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Vec<Project>>, (StatusCode, Json<ErrorResponse>)> {
    let projects = sqlx::query_as!(
        Project,
        r#"SELECT id, name, sort_order, created_at FROM projects ORDER BY sort_order ASC"#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch projects: {}", e),
            }),
        )
    })?;

    Ok(Json(projects))
}

pub async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ProjectWithDetails>, (StatusCode, Json<ErrorResponse>)> {
    let project = sqlx::query_as!(
        Project,
        r#"SELECT id, name, sort_order, created_at FROM projects WHERE id = ?"#,
        id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch project: {}", e),
            }),
        )
    })?;

    let project = project.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
            }),
        )
    })?;

    let columns = sqlx::query_as!(
        Column,
        r#"SELECT id, project_id, name, sort_order, created_at FROM columns WHERE project_id = ? ORDER BY sort_order"#,
        id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch columns: {}", e),
            }),
        )
    })?;

    let tasks = sqlx::query_as!(
        Task,
        r#"SELECT id, project_id, column_id, title, description, priority as "priority?", sort_order, source_tag as "source_tag?", created_at FROM tasks WHERE project_id = ? ORDER BY sort_order"#,
        id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch tasks: {}", e),
            }),
        )
    })?;

    let columns_with_tasks: Vec<ColumnWithTasks> = columns
        .into_iter()
        .map(|col| {
            let column_tasks: Vec<Task> = tasks
                .iter()
                .filter(|t| t.column_id == col.id)
                .cloned()
                .collect();

            ColumnWithTasks {
                id: col.id,
                project_id: col.project_id,
                name: col.name,
                sort_order: col.sort_order,
                created_at: col.created_at,
                tasks: column_tasks,
            }
        })
        .collect();

    Ok(Json(ProjectWithDetails {
        id: project.id,
        name: project.name,
        created_at: project.created_at,
        columns: columns_with_tasks,
    }))
}

pub async fn update_project(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, (StatusCode, Json<ErrorResponse>)> {
    let project = sqlx::query_as!(
        Project,
        r#"UPDATE projects SET name = ? WHERE id = ? RETURNING id, name, sort_order, created_at"#,
        payload.name,
        id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to update project: {}", e),
            }),
        )
    })?;

    let project = project.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
            }),
        )
    })?;

    Ok(Json(project))
}

pub async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query!(r#"DELETE FROM projects WHERE id = ?"#, id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to delete project: {}", e),
                }),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct ReorderProjectItem {
    pub id: i64,
    pub sort_order: i64,
}

pub async fn reorder_projects(
    State(state): State<AppState>,
    Json(payload): Json<Vec<ReorderProjectItem>>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut tx = state.db.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to start transaction: {}", e),
            }),
        )
    })?;

    for item in payload {
        sqlx::query!(
            r#"UPDATE projects SET sort_order = ? WHERE id = ?"#,
            item.sort_order,
            item.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to update project order: {}", e),
                }),
            )
        })?;
    }

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to commit transaction: {}", e),
            }),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}
