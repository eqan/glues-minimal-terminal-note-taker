use std::{collections::BTreeMap, fs, path::PathBuf};

pub const LAST_FILE_PATH: &str = "last_file_path";
pub const LAST_GIT_PATH: &str = "last_git_path";
pub const LAST_GIT_REMOTE: &str = "last_git_remote";
pub const LAST_GIT_BRANCH: &str = "last_git_branch";
pub const LAST_THEME: &str = "last_theme";

const DEFAULTS: &[(&str, &str)] = &[
    (LAST_FILE_PATH, ""),
    (LAST_GIT_PATH, ""),
    (LAST_GIT_REMOTE, "origin"),
    (LAST_GIT_BRANCH, "main"),
    (LAST_THEME, "dark"),
];

pub(crate) fn config_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));
    home.join(".glues")
}

fn config_file() -> PathBuf {
    config_dir().join("config")
}

fn read_values() -> BTreeMap<String, String> {
    fs::read_to_string(config_file())
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn write_values(values: &BTreeMap<String, String>) {
    let directory = config_dir();
    fs::create_dir_all(&directory).expect("failed to create Glues config directory");
    let content = values
        .iter()
        .map(|(key, value)| format!("{key}={value}\n"))
        .collect::<String>();
    fs::write(config_file(), content).expect("failed to write Glues config");
}

pub async fn init() {
    let mut values = read_values();
    for (key, value) in DEFAULTS {
        values
            .entry((*key).to_owned())
            .or_insert_with(|| (*value).to_owned());
    }
    write_values(&values);
}

pub async fn update(key: &str, value: &str) {
    let mut values = read_values();
    values.insert(key.to_owned(), value.to_owned());
    write_values(&values);
}

pub async fn get(key: &str) -> Option<String> {
    read_values().remove(key)
}
