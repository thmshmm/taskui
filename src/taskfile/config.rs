use anyhow::{anyhow, bail, Result};
use serde_yaml::Value;
use std::{
    fs::{metadata, File},
    path::PathBuf,
};

#[derive(Clone, Debug)]
pub struct Task {
    pub name: String,
    pub body: String,
    pub internal: bool,
}

#[derive(Clone, Debug)]
struct Include {
    name: String,
    path: String,
    optional: bool,
    internal: bool,
}

pub fn load() -> Result<Vec<Task>> {
    let taskfile = File::open(find_supported_file()?).unwrap();
    let taskfile_yml: Value = serde_yaml::from_reader(taskfile)?;

    let mut tasks = get_tasks(&taskfile_yml)?;
    let includes = get_includes(&taskfile_yml)?;
    let current_path = std::env::current_dir().unwrap();
    let included_tasks = handle_includes(
        includes.unwrap_or(Vec::new()),
        current_path.to_str().unwrap(),
    )?;

    tasks.extend(included_tasks);

    Ok(tasks)
}

fn find_supported_file() -> Result<&'static str> {
    // https://taskfile.dev/usage/#supported-file-names
    let file_names = [
        "Taskfile.yml",
        "taskfile.yml",
        "Taskfile.yaml",
        "taskfile.yaml",
        "Taskfile.dist.yml",
        "taskfile.dist.yml",
        "Taskfile.dist.yaml",
        "taskfile.dist.yaml",
    ];

    let found = file_names
        .iter()
        .find(|&file_name| metadata(file_name).is_ok())
        .copied();

    match found {
        Some(file_name) => Ok(file_name),
        None => Err(anyhow!("no supported file found")),
    }
}

fn handle_includes(includes: Vec<Include>, current_path: &str) -> Result<Vec<Task>> {
    let mut tasks: Vec<Task> = Vec::new();

    for include in includes {
        let include_path = PathBuf::from(current_path).join(include.path.clone());

        match File::open(&include_path) {
            Ok(taskfile) => {
                let taskfile_yml: Value = serde_yaml::from_reader(taskfile)?;
                let include_tasks = get_tasks(&taskfile_yml)?;
                let include_tasks: Vec<Task> =
                    prefix_tasks(include_tasks.clone(), include.name.clone());
                let include_tasks: Vec<Task> = flag_internal_tasks(&include, include_tasks);
                tasks.extend(include_tasks);

                if let Some(sub_includes) = get_includes(&taskfile_yml)? {
                    // remove filename from path
                    let include_path = include_path.parent().unwrap().to_str().unwrap();
                    let sub_include_tasks = handle_includes(sub_includes, include_path)?;
                    let sub_include_tasks: Vec<Task> =
                        prefix_tasks(sub_include_tasks.clone(), include.name.clone());
                    let sub_include_tasks = flag_internal_tasks(&include, sub_include_tasks);
                    tasks.extend(sub_include_tasks);
                }
            }
            Err(_) => {
                if include.optional {
                    continue;
                }
                bail!("include not found: {}", include.path);
            }
        }
    }

    tasks.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(tasks)
}

fn flag_internal_tasks(include: &Include, tasks: Vec<Task>) -> Vec<Task> {
    tasks
        .into_iter()
        .map(|task| Task {
            name: task.name,
            body: task.body,
            internal: task.internal || include.internal,
        })
        .collect()
}

fn prefix_tasks(tasks: Vec<Task>, include_name: String) -> Vec<Task> {
    tasks
        .into_iter()
        .map(|task| Task {
            name: format!("{}:{}", include_name, task.name),
            body: task.body,
            internal: task.internal,
        })
        .collect()
}

fn get_tasks(taskfile_yml: &Value) -> Result<Vec<Task>> {
    let mut tasks: Vec<Task> = Vec::new();

    if let Some(task_mapping) = taskfile_yml.get("tasks").and_then(Value::as_mapping) {
        for (key, body) in task_mapping {
            let task_name = key.as_str().unwrap().to_string();
            let internal = extract_bool(body, "internal", false)?;

            tasks.push(Task {
                name: task_name,
                body: serde_yaml::to_string(body).unwrap_or_else(|_| "no content".to_string()),
                internal,
            });
        }
    } else {
        bail!("failed to extract tasks")
    }

    tasks.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(tasks)
}

fn get_includes(taskfile_yml: &Value) -> Result<Option<Vec<Include>>> {
    let mut includes: Vec<Include> = Vec::new();

    if let Some(include_mapping) = taskfile_yml.get("includes").and_then(Value::as_mapping) {
        for (key, value) in include_mapping {
            let name = extract_include_name(key);
            let path = extract_include_path(value)?;
            let optional = extract_bool(value, "optional", false)?;
            let internal = extract_bool(value, "internal", false)?;

            includes.push(Include {
                name,
                path,
                optional,
                internal,
            });
        }
    } else {
        return Ok(None);
    }

    Ok(Some(includes))
}

fn extract_include_name(include_key: &Value) -> String {
    include_key.as_str().unwrap().to_string()
}

fn extract_include_path(include_yml: &Value) -> Result<String> {
    let path = match include_yml {
        Value::String(path) if path.ends_with(".yml") || path.ends_with(".yaml") => {
            path.to_string()
        }
        Value::String(path) if path.ends_with('/') => format!("{}Taskfile.yml", path),
        Value::String(path) => format!("{}/Taskfile.yml", path),
        Value::Mapping(v) => {
            if let Some(taskfile) = v.get(&Value::String("taskfile".to_string())) {
                if let Value::String(s) = taskfile {
                    if s.ends_with(".yml") || s.ends_with(".yaml") {
                        s.to_string()
                    } else {
                        bail!("value of taskfile key must end with .yml or .yaml")
                    }
                } else {
                    bail!("value of taskfile key must be of type string")
                }
            } else {
                bail!("taskfile key not found in include mapping")
            }
        }
        _ => bail!("invalid include found"),
    };

    Ok(path)
}

fn extract_bool(yml: &Value, field: &str, default: bool) -> Result<bool> {
    match yml {
        Value::Mapping(v) => {
            if let Some(value) = v
                .get(&Value::String(field.to_string()))
                .and_then(Value::as_bool)
            {
                Ok(value)
            } else {
                Ok(default)
            }
        }
        _ => Ok(false),
    }
}
