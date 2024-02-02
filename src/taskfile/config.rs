use anyhow::{bail, Result};
use serde_yaml::Value;
use std::{
    fs::{metadata, File},
    path::PathBuf,
};

#[derive(Clone)]
pub struct Task {
    pub name: String,
}

#[derive(Clone)]
struct Include {
    name: String,
    path: String,
    optional: bool,
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

    tasks.extend(included_tasks.into_iter());

    Ok(tasks)
}

fn find_supported_file() -> Result<&'static str> {
    // https://taskfile.dev/usage/#supported-file-names
    let file_names = vec![
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
        .map(|&file_name| file_name);

    match found {
        Some(file_name) => Ok(file_name),
        None => bail!("no supported file found"),
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
                tasks.extend(include_tasks);

                if let Some(sub_includes) = get_includes(&taskfile_yml)? {
                    // remove filename from path
                    let include_path = include_path.parent().unwrap().to_str().unwrap();
                    let sub_include_tasks = handle_includes(sub_includes, include_path)?;
                    let sub_include_tasks: Vec<Task> =
                        prefix_tasks(sub_include_tasks.clone(), include.name.clone());
                    tasks.extend(sub_include_tasks);
                }
            }
            Err(_) => {
                if include.optional {
                    continue;
                } else {
                    bail!("include not found: {}", include.path);
                }
            }
        }
    }

    tasks.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(tasks)
}

fn prefix_tasks(tasks: Vec<Task>, include_name: String) -> Vec<Task> {
    tasks
        .into_iter()
        .map(|task| Task {
            name: format!("{}:{}", include_name, task.name),
        })
        .collect()
}

fn get_tasks(taskfile_yml: &Value) -> Result<Vec<Task>> {
    let mut tasks: Vec<Task> = Vec::new();

    if let Some(task_mapping) = taskfile_yml.get("tasks").and_then(Value::as_mapping) {
        for (key, _) in task_mapping {
            let task_name = key.as_str().unwrap().to_string();
            tasks.push(Task { name: task_name });
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
            let optional = extract_include_optional(value)?;

            includes.push(Include {
                name,
                path,
                optional,
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
        Value::String(path) if path.ends_with("/") => format!("{}Taskfile.yml", path),
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

fn extract_include_optional(include_yml: &Value) -> Result<bool> {
    match include_yml {
        Value::Mapping(v) => {
            if let Some(optional) = v
                .get(&Value::String("optional".to_string()))
                .and_then(Value::as_bool)
            {
                Ok(optional)
            } else {
                Ok(false)
            }
        }
        _ => Ok(false),
    }
}
