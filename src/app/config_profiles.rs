use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::models::mqtt::MqttLoginData;

#[derive(Clone, Debug)]
pub(crate) struct ProfileEntry {
    pub(crate) display_name: String,
    pub(crate) file_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginTemplateFile {
    #[serde(default)]
    profile_name: Option<String>,
    #[serde(default)]
    name: String,
    #[serde(default)]
    broker: String,
    #[serde(default)]
    port: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    client_id: String,
    #[serde(default)]
    keep_alive_secs: u16,
    #[serde(default)]
    testament_and_last_will: String,
    #[serde(default)]
    testament_topic: String,
    #[serde(default)]
    testament_qos: u8,
    #[serde(default)]
    testament_retain: bool,
}

impl LoginTemplateFile {
    fn from_login(profile_name: Option<String>, login: &MqttLoginData) -> Self {
        Self {
            profile_name,
            name: login.name.clone(),
            broker: login.broker.clone(),
            port: login.port.clone(),
            username: login.username.clone(),
            client_id: login.client_id.clone(),
            keep_alive_secs: login.effective_keep_alive_secs(),
            testament_and_last_will: login.testament_and_last_will.clone(),
            testament_topic: login.testament_topic.clone(),
            testament_qos: login.testament_qos,
            testament_retain: login.testament_retain,
        }
    }

    fn into_login(self) -> MqttLoginData {
        MqttLoginData {
            name: self.name,
            broker: self.broker,
            port: self.port,
            username: self.username,
            password: String::new(),
            client_id: self.client_id,
            keep_alive_secs: self.keep_alive_secs.max(1),
            testament_and_last_will: self.testament_and_last_will,
            testament_topic: self.testament_topic,
            testament_qos: self.testament_qos,
            testament_retain: self.testament_retain,
        }
    }
}

pub(crate) fn list_profiles() -> Result<Vec<ProfileEntry>, String> {
    let dir = profiles_dir()?;
    let mut entries = Vec::new();

    let dir_entries = fs::read_dir(&dir)
        .map_err(|err| format!("Failed to read profile directory {}: {err}", dir.display()))?;

    for dir_entry in dir_entries {
        let dir_entry = match dir_entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let path = dir_entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }

        let fallback = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("profile")
            .to_string();

        let display_name = match fs::read_to_string(&path)
            .ok()
            .and_then(|text| toml::from_str::<LoginTemplateFile>(&text).ok())
            .and_then(|template| template.profile_name)
        {
            Some(name) if !name.trim().is_empty() => name,
            _ => fallback,
        };

        entries.push(ProfileEntry {
            display_name,
            file_path: path,
        });
    }

    entries.sort_by_key(|entry| entry.display_name.to_lowercase());
    Ok(entries)
}

pub(crate) fn save_profile(profile_name: &str, login: &MqttLoginData) -> Result<(), String> {
    let trimmed = profile_name.trim();
    if trimmed.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let file_name = format!("{}.toml", safe_file_name(trimmed));
    let path = profiles_dir()?.join(file_name);

    let template = LoginTemplateFile::from_login(Some(trimmed.to_string()), login);
    let serialized = toml::to_string_pretty(&template)
        .map_err(|err| format!("Failed to serialize profile '{trimmed}': {err}"))?;

    fs::write(&path, serialized)
        .map_err(|err| format!("Failed to write profile {}: {err}", path.display()))
}

pub(crate) fn load_profile_file(path: &Path) -> Result<MqttLoginData, String> {
    let contents =
        fs::read_to_string(path).map_err(|err| format!("Failed to read {}: {err}", path.display()))?;
    let template: LoginTemplateFile = toml::from_str(&contents)
        .map_err(|err| format!("Failed to parse TOML {}: {err}", path.display()))?;
    Ok(template.into_login())
}

pub(crate) fn load_template_file(path: &Path) -> Result<MqttLoginData, String> {
    load_profile_file(path)
}

fn profiles_dir() -> Result<PathBuf, String> {
    let project_dirs = ProjectDirs::from("io", "jotrorox", "mqui")
        .ok_or_else(|| "Could not resolve operating system config directory".to_string())?;

    let dir = project_dirs.config_dir().join("profiles");
    fs::create_dir_all(&dir)
        .map_err(|err| format!("Failed to create profile directory {}: {err}", dir.display()))?;
    Ok(dir)
}

fn safe_file_name(value: &str) -> String {
    let mut output = String::new();

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            output.push(ch);
        } else {
            output.push('_');
        }
    }

    let trimmed = output.trim_matches('_');
    if trimmed.is_empty() {
        "profile".to_string()
    } else {
        trimmed.to_string()
    }
}