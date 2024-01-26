use anyhow::{bail, Result};
use serde_yaml::Value;
use std::fs::{metadata, File};

pub struct Taskfile {
    pub tasks: Vec<Task>,
}

#[derive(Clone)]
pub struct Task {
    pub name: String,
}

pub fn load() -> Result<Taskfile> {
    let cfg_file = open_file()?;

    let cfg: Value = serde_yaml::from_reader(cfg_file)?;

    let mut task_list: Vec<Task> = Vec::new();

    if let Some(tasks) = cfg.get("tasks").and_then(Value::as_mapping) {
        for (key, _) in tasks {
            let task_name = key.as_str().unwrap().to_string();
            task_list.push(Task { name: task_name });
        }
    } else {
        bail!("failed to extract tasks")
    }

    let taskfile = Taskfile { tasks: task_list };

    Ok(taskfile)
}

fn open_file() -> Result<File> {
    let file_names = vec!["Taskfile.yml", "Taskfile.yaml"];

    let mut found = None;

    for &file_name in &file_names {
        if metadata(file_name).is_ok() {
            found = Some(file_name);
            break;
        }
    }

    match found {
        Some(file_name) => Ok(File::open(file_name).unwrap()),
        None => bail!("Taskfile.(yml/yaml) not found"),
    }
}
