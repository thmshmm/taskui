use anyhow::{bail, Result};
use serde_yaml::Value;
use std::fs::File;

pub struct Taskfile {
    pub tasks: Vec<String>,
}

pub fn load() -> Result<Taskfile> {
    let cfg_file = File::open("Taskfile.yml")?;

    let cfg: Value = serde_yaml::from_reader(cfg_file)?;

    let mut task_list: Vec<String> = Vec::new();

    if let Some(tasks) = cfg.get("tasks").and_then(Value::as_mapping) {
        for (key, _) in tasks {
            let task_name = key.as_str().unwrap().to_string();
            task_list.push(task_name);
        }
    } else {
        bail!("failed to extract tasks")
    }

    let taskfile = Taskfile { tasks: task_list };

    Ok(taskfile)
}
