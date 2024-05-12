use std::env;

const ENV_PREFIX: &str = "TASKUI_";

pub struct Config {
    pub list_internal: bool,
}

impl Config {
    pub fn load() -> Config {
        Config {
            list_internal: env::var(ENV_PREFIX.to_string() + "LIST_INTERNAL")
                .unwrap_or("false".to_string())
                .parse()
                .unwrap(),
        }
    }
}
