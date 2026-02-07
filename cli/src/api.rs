use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::load_global_config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    pub id: i64,
    pub name: String,
    pub sort_order: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub column_id: i64,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkedPath {
    pub id: i64,
    pub project_id: i64,
    pub path: String,
    pub hostname: Option<String>,
    pub default_column_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkedPathLookup {
    pub linked_path: LinkedPath,
    pub project_name: String,
}

#[derive(Debug, Serialize)]
struct CreateTaskPayload {
    column_id: i64,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_tag: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateLinkedPathPayload {
    path: String,
    hostname: Option<String>,
    default_column_id: Option<i64>,
}

#[derive(Debug, Serialize)]
struct DeleteByPathPayload {
    path: String,
}

pub struct ApiClient {
    client: reqwest::blocking::Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    pub fn new() -> Result<Self> {
        let config = load_global_config()?;
        let base_url = config
            .api_url
            .context("API URL not configured. Run: rk init <url> <api-key>")?;
        let api_key = config
            .api_key
            .context("API key not configured. Run: rk init <url> <api-key>")?;

        Ok(Self {
            client: reqwest::blocking::Client::new(),
            base_url,
            api_key,
        })
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let url = format!("{}/api/projects", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            bail!("API error: {}", response.status());
        }

        let projects: Vec<Project> = response.json()?;
        Ok(projects)
    }

    pub fn get_project_columns(&self, project_id: i64) -> Result<Vec<Column>> {
        let url = format!("{}/api/projects/{}/columns", self.base_url, project_id);
        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            bail!("API error: {}", response.status());
        }

        let columns: Vec<Column> = response.json()?;
        Ok(columns)
    }

    pub fn create_task(&self, params: CreateTaskParams) -> Result<Task> {
        let column_id = match params.column_id {
            Some(id) => id,
            None => {
                let columns = self.get_project_columns(params.project_id)?;
                columns.first().context("Project has no columns")?.id
            }
        };

        let url = format!("{}/api/projects/{}/tasks", self.base_url, params.project_id);
        let payload = CreateTaskPayload {
            column_id,
            title: params.title,
            description: params.description,
            source_tag: params.source_tag,
        };

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&payload)
            .send()
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            bail!("API error: {} - {}", status, body);
        }

        let task: Task = response.json()?;
        Ok(task)
    }

    pub fn create_linked_path(&self, params: CreateLinkedPathParams) -> Result<LinkedPath> {
        let url = format!(
            "{}/api/projects/{}/linked-paths",
            self.base_url, params.project_id
        );
        let payload = CreateLinkedPathPayload {
            path: params.path,
            hostname: params.hostname,
            default_column_id: params.default_column_id,
        };

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&payload)
            .send()
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            bail!("API error: {} - {}", status, body);
        }

        let linked_path: LinkedPath = response.json()?;
        Ok(linked_path)
    }

    pub fn delete_linked_path_by_path(&self, path: &str) -> Result<()> {
        let url = format!("{}/api/linked-paths/by-path", self.base_url);
        let payload = DeleteByPathPayload {
            path: path.to_string(),
        };

        let response = self
            .client
            .delete(&url)
            .header("X-API-Key", &self.api_key)
            .json(&payload)
            .send()
            .context("Failed to connect to API")?;

        if !response.status().is_success() && response.status() != reqwest::StatusCode::NO_CONTENT {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            bail!("API error: {} - {}", status, body);
        }

        Ok(())
    }

    pub fn lookup_linked_path(&self, path: &str) -> Result<Option<LinkedPathLookup>> {
        let hostname = hostname::get().ok().and_then(|h| h.into_string().ok());
        let mut url = format!(
            "{}/api/linked-paths/lookup?path={}",
            self.base_url,
            urlencoding::encode(path)
        );
        if let Some(h) = &hostname {
            url.push_str(&format!("&hostname={}", urlencoding::encode(h)));
        }

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .context("Failed to connect to API")?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            bail!("API error: {} - {}", status, body);
        }

        let lookup: LinkedPathLookup = response.json()?;
        Ok(Some(lookup))
    }

    pub fn list_tasks(&self, project_id: i64) -> Result<Vec<Task>> {
        let url = format!("{}/api/projects/{}/tasks", self.base_url, project_id);
        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            bail!("API error: {} - {}", status, body);
        }

        let tasks: Vec<Task> = response.json()?;
        Ok(tasks)
    }

    pub fn delete_task(&self, task_id: i64) -> Result<()> {
        let url = format!("{}/api/tasks/{}", self.base_url, task_id);
        let response = self
            .client
            .delete(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .context("Failed to connect to API")?;

        if !response.status().is_success() && response.status() != reqwest::StatusCode::NO_CONTENT {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            bail!("API error: {} - {}", status, body);
        }

        Ok(())
    }

    pub fn move_task(&self, task_id: i64, column_id: i64) -> Result<Task> {
        let url = format!("{}/api/tasks/{}", self.base_url, task_id);
        let response = self
            .client
            .put(&url)
            .header("X-API-Key", &self.api_key)
            .json(&serde_json::json!({ "column_id": column_id }))
            .send()
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            bail!("API error: {} - {}", status, body);
        }

        let task: Task = response.json()?;
        Ok(task)
    }

    pub fn update_task_description(&self, task_id: i64, description: &str) -> Result<Task> {
        let url = format!("{}/api/tasks/{}", self.base_url, task_id);
        let response = self
            .client
            .put(&url)
            .header("X-API-Key", &self.api_key)
            .json(&serde_json::json!({ "description": description }))
            .send()
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            bail!("API error: {} - {}", status, body);
        }

        let task: Task = response.json()?;
        Ok(task)
    }

    pub fn get_task(&self, task_id: i64) -> Result<Task> {
        let url = format!("{}/api/tasks/{}", self.base_url, task_id);
        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            bail!("API error: {} - {}", status, body);
        }

        let task: Task = response.json()?;
        Ok(task)
    }
}

pub struct CreateLinkedPathParams {
    pub project_id: i64,
    pub path: String,
    pub hostname: Option<String>,
    pub default_column_id: Option<i64>,
}

pub struct CreateTaskParams {
    pub project_id: i64,
    pub column_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub source_tag: Option<String>,
}
