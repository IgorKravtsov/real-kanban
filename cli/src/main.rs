mod api;
mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};

use api::{ApiClient, CreateLinkedPathParams, CreateTaskParams};
use config::{load_global_config, save_global_config, GlobalConfig};

#[derive(Parser)]
#[command(name = "rk")]
#[command(about = "Real Kanban CLI - Create tasks from anywhere")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize global config with API URL and key")]
    Init {
        #[arg(help = "Backend API URL (e.g., http://localhost:3001)")]
        url: Option<String>,
        #[arg(help = "API key for authentication")]
        api_key: Option<String>,
        #[arg(long = "url", help = "Set only the API URL")]
        set_url: Option<String>,
        #[arg(long = "key", help = "Set only the API key")]
        set_key: Option<String>,
    },

    #[command(about = "Check if rk is initialized and can connect to backend")]
    Check,

    #[command(about = "Link current directory to a project")]
    Link {
        #[arg(help = "Project ID to link to")]
        project_id: i64,
        #[arg(short, long, help = "Default column ID for new tasks")]
        column: Option<i64>,
    },

    #[command(about = "Unlink current directory from its project")]
    Unlink,

    #[command(about = "List available projects")]
    Projects,

    #[command(about = "List columns for the linked project")]
    Columns,

    #[command(about = "List tasks for the linked project")]
    Tasks,

    #[command(about = "Show current directory's linked project")]
    Status,

    #[command(about = "Add a new task to the linked project")]
    Add {
        #[arg(help = "Task title")]
        title: String,
        #[arg(short, long, help = "Column name or ID (e.g., 'In Progress' or '3')")]
        column: Option<String>,
        #[arg(short, long, help = "Task description")]
        description: Option<String>,
        #[arg(short, long, help = "Source tag (default: 'manual')")]
        tag: Option<String>,
    },

    #[command(about = "Remove a task by title")]
    Remove {
        #[arg(help = "Task title to remove")]
        title: String,
    },

    #[command(about = "Move a task to a different column")]
    Move {
        #[arg(help = "Task title to move")]
        title: String,
        #[arg(short, long, help = "Target column name or ID")]
        column: String,
    },

    #[command(about = "Mark a task as done (move to last column)")]
    Done {
        #[arg(help = "Task title to mark as done")]
        title: String,
    },

    #[command(about = "Append text to a task's description")]
    Describe {
        #[arg(help = "Task title")]
        title: String,
        #[arg(help = "Text to append to description")]
        text: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            url,
            api_key,
            set_url,
            set_key,
        } => cmd_init(url, api_key, set_url, set_key),
        Commands::Check => cmd_check(),
        Commands::Link { project_id, column } => cmd_link(project_id, column),
        Commands::Unlink => cmd_unlink(),
        Commands::Projects => cmd_projects(),
        Commands::Columns => cmd_columns(),
        Commands::Tasks => cmd_tasks(),
        Commands::Status => cmd_status(),
        Commands::Add {
            title,
            column,
            description,
            tag,
        } => cmd_add(title, column, description, tag),
        Commands::Remove { title } => cmd_remove(title),
        Commands::Move { title, column } => cmd_move(title, column),
        Commands::Done { title } => cmd_done(title),
        Commands::Describe { title, text } => cmd_describe(title, text),
    }
}

fn cmd_init(
    url: Option<String>,
    api_key: Option<String>,
    set_url: Option<String>,
    set_key: Option<String>,
) -> Result<()> {
    if let Some(new_url) = set_url {
        let mut config = load_global_config()?;
        config.api_url = Some(new_url.clone());
        save_global_config(&config)?;
        println!("API URL updated: {}", new_url);
        return Ok(());
    }

    if let Some(new_key) = set_key {
        let mut config = load_global_config()?;
        config.api_key = Some(new_key);
        save_global_config(&config)?;
        println!("API key updated");
        return Ok(());
    }

    match (url, api_key) {
        (Some(u), Some(k)) => {
            let config = GlobalConfig {
                api_url: Some(u.clone()),
                api_key: Some(k),
            };
            save_global_config(&config)?;
            println!("Configuration saved. API URL: {}", u);
            Ok(())
        }
        _ => {
            anyhow::bail!(
                "Usage: rk init <URL> <API_KEY> or rk init --url <URL> or rk init --key <API_KEY>"
            );
        }
    }
}

fn cmd_check() -> Result<()> {
    let config = load_global_config()?;

    if config.api_url.is_none() || config.api_key.is_none() {
        println!("not configured");
        std::process::exit(1);
    }

    match ApiClient::new() {
        Ok(client) => match client.list_projects() {
            Ok(_) => {
                println!("ok");
                Ok(())
            }
            Err(_) => {
                println!("cannot connect");
                std::process::exit(1);
            }
        },
        Err(_) => {
            println!("not configured");
            std::process::exit(1);
        }
    }
}

fn cmd_describe(title: String, text: String) -> Result<()> {
    let client = ApiClient::new()?;
    let current_dir = std::env::current_dir()?;
    let path = current_dir.to_string_lossy().to_string();

    let lookup = client.lookup_linked_path(&path)?.ok_or_else(|| {
        anyhow::anyhow!("Current directory is not linked. Run: rk link <project-id>")
    })?;

    let tasks = client.list_tasks(lookup.linked_path.project_id)?;
    let task = find_task_by_title(&tasks, &title)?;

    let full_task = client.get_task(task.id)?;
    let new_description = match &full_task.description {
        Some(existing) if !existing.is_empty() => format!("{}\n\n{}", existing, text),
        _ => text,
    };

    client.update_task_description(task.id, &new_description)?;
    println!("Updated description for task '{}'", task.title);
    Ok(())
}

fn find_task_by_title<'a>(tasks: &'a [api::Task], title: &str) -> Result<&'a api::Task> {
    let matching: Vec<_> = tasks
        .iter()
        .filter(|t| t.title.trim().to_lowercase() == title.trim().to_lowercase())
        .collect();

    match matching.len() {
        0 => anyhow::bail!("No task found with title '{}'", title),
        1 => Ok(matching[0]),
        _ => {
            println!("Multiple tasks found with title '{}':", title);
            for task in &matching {
                println!(
                    "  [{}] {} (column: {})",
                    task.id, task.title, task.column_id
                );
            }
            anyhow::bail!("Ambiguous task title - please use a more specific title");
        }
    }
}

fn find_column_by_name_or_id<'a>(
    columns: &'a [api::Column],
    column_arg: &str,
) -> Result<&'a api::Column> {
    if let Ok(id) = column_arg.parse::<i64>() {
        columns
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| anyhow::anyhow!("Column with ID {} not found", id))
    } else {
        columns
            .iter()
            .find(|c| c.name.to_lowercase() == column_arg.to_lowercase())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Column '{}' not found. Run 'rk columns' to see available columns.",
                    column_arg
                )
            })
    }
}

fn cmd_move(title: String, column: String) -> Result<()> {
    let client = ApiClient::new()?;
    let current_dir = std::env::current_dir()?;
    let path = current_dir.to_string_lossy().to_string();

    let lookup = client.lookup_linked_path(&path)?.ok_or_else(|| {
        anyhow::anyhow!("Current directory is not linked. Run: rk link <project-id>")
    })?;

    let tasks = client.list_tasks(lookup.linked_path.project_id)?;
    let task = find_task_by_title(&tasks, &title)?;

    let columns = client.get_project_columns(lookup.linked_path.project_id)?;
    let target_column = find_column_by_name_or_id(&columns, &column)?;

    client.move_task(task.id, target_column.id)?;
    println!("Moved task '{}' to '{}'", task.title, target_column.name);
    Ok(())
}

fn cmd_done(title: String) -> Result<()> {
    let client = ApiClient::new()?;
    let current_dir = std::env::current_dir()?;
    let path = current_dir.to_string_lossy().to_string();

    let lookup = client.lookup_linked_path(&path)?.ok_or_else(|| {
        anyhow::anyhow!("Current directory is not linked. Run: rk link <project-id>")
    })?;

    let tasks = client.list_tasks(lookup.linked_path.project_id)?;
    let task = find_task_by_title(&tasks, &title)?;

    let columns = client.get_project_columns(lookup.linked_path.project_id)?;
    let done_column = columns
        .last()
        .ok_or_else(|| anyhow::anyhow!("Project has no columns"))?;

    client.move_task(task.id, done_column.id)?;
    println!(
        "Marked task '{}' as done (moved to '{}')",
        task.title, done_column.name
    );
    Ok(())
}

fn cmd_link(project_id: i64, column: Option<i64>) -> Result<()> {
    let client = ApiClient::new()?;
    let projects = client.list_projects()?;

    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| anyhow::anyhow!("Project with ID {} not found", project_id))?;

    let current_dir = std::env::current_dir()?;
    let path = current_dir.to_string_lossy().to_string();
    let hostname = hostname::get().ok().and_then(|h| h.into_string().ok());

    client.create_linked_path(CreateLinkedPathParams {
        project_id,
        path: path.clone(),
        hostname,
        default_column_id: column,
    })?;

    println!(
        "Linked '{}' to project '{}' (ID: {})",
        path, project.name, project_id
    );
    Ok(())
}

fn cmd_unlink() -> Result<()> {
    let client = ApiClient::new()?;
    let current_dir = std::env::current_dir()?;
    let path = current_dir.to_string_lossy().to_string();

    client.delete_linked_path_by_path(&path)?;
    println!("Unlinked '{}'", path);
    Ok(())
}

fn cmd_projects() -> Result<()> {
    let client = ApiClient::new()?;
    let projects = client.list_projects()?;

    if projects.is_empty() {
        println!("No projects found.");
        return Ok(());
    }

    println!("Available projects:");
    for project in projects {
        println!("  [{}] {}", project.id, project.name);
    }
    Ok(())
}

fn cmd_columns() -> Result<()> {
    let client = ApiClient::new()?;
    let current_dir = std::env::current_dir()?;
    let path = current_dir.to_string_lossy().to_string();

    let lookup = client.lookup_linked_path(&path)?.ok_or_else(|| {
        anyhow::anyhow!("Current directory is not linked. Run: rk link <project-id>")
    })?;

    let columns = client.get_project_columns(lookup.linked_path.project_id)?;

    if columns.is_empty() {
        println!("No columns found in project '{}'.", lookup.project_name);
        return Ok(());
    }

    println!("Columns in '{}':", lookup.project_name);
    for col in columns {
        let default_marker = if Some(col.id) == lookup.linked_path.default_column_id {
            " (default)"
        } else {
            ""
        };
        println!("  [{}] {}{}", col.id, col.name, default_marker);
    }
    Ok(())
}

fn cmd_tasks() -> Result<()> {
    let client = ApiClient::new()?;
    let current_dir = std::env::current_dir()?;
    let path = current_dir.to_string_lossy().to_string();

    let lookup = client.lookup_linked_path(&path)?.ok_or_else(|| {
        anyhow::anyhow!("Current directory is not linked. Run: rk link <project-id>")
    })?;

    let columns = client.get_project_columns(lookup.linked_path.project_id)?;
    let tasks = client.list_tasks(lookup.linked_path.project_id)?;

    if tasks.is_empty() {
        println!("No tasks in project '{}'.", lookup.project_name);
        return Ok(());
    }

    println!("Tasks in '{}':", lookup.project_name);
    for col in &columns {
        let col_tasks: Vec<_> = tasks.iter().filter(|t| t.column_id == col.id).collect();
        if !col_tasks.is_empty() {
            println!("\n  {}:", col.name);
            for task in col_tasks {
                println!("    [{}] {}", task.id, task.title);
            }
        }
    }
    Ok(())
}

fn cmd_status() -> Result<()> {
    let config = load_global_config()?;

    println!("Global config:");
    match &config.api_url {
        Some(url) => println!("  API URL: {}", url),
        None => println!("  API URL: (not configured)"),
    }
    println!(
        "  API Key: {}",
        if config.api_key.is_some() {
            "(set)"
        } else {
            "(not set)"
        }
    );

    println!();

    let client = ApiClient::new()?;
    let current_dir = std::env::current_dir()?;
    let path = current_dir.to_string_lossy().to_string();

    match client.lookup_linked_path(&path)? {
        Some(lookup) => {
            println!("Current directory is linked to:");
            println!(
                "  Project: {} (ID: {})",
                lookup.project_name, lookup.linked_path.project_id
            );
            if let Some(col) = lookup.linked_path.default_column_id {
                println!("  Default column: {}", col);
            }
        }
        None => {
            println!("Current directory is not linked to any project.");
            println!("Run: rk link <project-id>");
        }
    }
    Ok(())
}

fn cmd_add(
    title: String,
    column: Option<String>,
    description: Option<String>,
    tag: Option<String>,
) -> Result<()> {
    let client = ApiClient::new()?;
    let current_dir = std::env::current_dir()?;
    let path = current_dir.to_string_lossy().to_string();

    let lookup = client.lookup_linked_path(&path)?.ok_or_else(|| {
        anyhow::anyhow!("Current directory is not linked. Run: rk link <project-id>")
    })?;

    let column_id = match column {
        Some(col_arg) => {
            if let Ok(id) = col_arg.parse::<i64>() {
                Some(id)
            } else {
                let columns = client.get_project_columns(lookup.linked_path.project_id)?;
                let found = columns
                    .iter()
                    .find(|c| c.name.to_lowercase() == col_arg.to_lowercase())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Column '{}' not found. Run 'rk columns' to see available columns.",
                            col_arg
                        )
                    })?;
                Some(found.id)
            }
        }
        None => lookup.linked_path.default_column_id,
    };

    let task = client.create_task(CreateTaskParams {
        project_id: lookup.linked_path.project_id,
        column_id,
        title: title.clone(),
        description,
        source_tag: tag,
    })?;

    println!(
        "Created task '{}' (ID: {}) in project '{}'",
        task.title, task.id, lookup.project_name
    );
    Ok(())
}

fn cmd_remove(title: String) -> Result<()> {
    let client = ApiClient::new()?;
    let current_dir = std::env::current_dir()?;
    let path = current_dir.to_string_lossy().to_string();

    let lookup = client.lookup_linked_path(&path)?.ok_or_else(|| {
        anyhow::anyhow!("Current directory is not linked. Run: rk link <project-id>")
    })?;

    let tasks = client.list_tasks(lookup.linked_path.project_id)?;
    let matching: Vec<_> = tasks
        .iter()
        .filter(|t| t.title.to_lowercase() == title.to_lowercase())
        .collect();

    match matching.len() {
        0 => {
            anyhow::bail!("No task found with title '{}'", title);
        }
        1 => {
            let task = matching[0];
            client.delete_task(task.id)?;
            println!("Deleted task '{}' (ID: {})", task.title, task.id);
        }
        _ => {
            println!("Multiple tasks found with title '{}':", title);
            for task in &matching {
                println!(
                    "  [{}] {} (column: {})",
                    task.id, task.title, task.column_id
                );
            }
            println!("\nUse 'rk remove-id <ID>' to delete a specific task.");
            anyhow::bail!("Ambiguous task title");
        }
    }

    Ok(())
}
