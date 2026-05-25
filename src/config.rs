use std::fs;
use std::path::PathBuf;

pub struct Config {
    pub ai_chat_url: String,
    pub ai_chat_name: String,
    pub imgur_client_id: String,
    pub paste_delay_secs: u64,
    pub enabled_engines: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ai_chat_url: "https://chatgpt.com".to_string(),
            ai_chat_name: "ChatGPT".to_string(),
            imgur_client_id: "546c25a59c58ad7".to_string(),
            paste_delay_secs: 3,
            enabled_engines: default_engines(),
        }
    }
}

pub fn default_engines() -> Vec<String> {
    ["lens", "yandex", "bing", "ai_chat", "tineye", "saucenao"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    if !path.exists() {
        let _ = write_default(&path);
        return Config::default();
    }
    let Ok(content) = fs::read_to_string(&path) else {
        return Config::default();
    };
    parse(&content)
}

fn config_path() -> Option<PathBuf> {
    dirs_next::home_dir().map(|h| h.join(".config/omarchy/circle-search.toml"))
}

fn parse(content: &str) -> Config {
    let mut cfg = Config::default();
    let mut section = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].to_string();
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let (k, v) = (k.trim(), v.trim().trim_matches('"'));
        match (section.as_str(), k) {
            ("ai_chat", "url") => cfg.ai_chat_url = v.to_string(),
            ("ai_chat", "name") => cfg.ai_chat_name = v.to_string(),
            ("ai_chat", "paste_delay") => cfg.paste_delay_secs = v.parse().unwrap_or(3),
            ("upload", "imgur_client_id") => cfg.imgur_client_id = v.to_string(),
            ("engines", "enabled") => {
                let parsed: Vec<String> = v
                    .trim_matches(|c: char| c == '[' || c == ']')
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !parsed.is_empty() {
                    cfg.enabled_engines = parsed;
                }
            }
            _ => {}
        }
    }
    cfg
}

fn write_default(path: &PathBuf) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, DEFAULT_CONFIG)
}

const DEFAULT_CONFIG: &str = "\
# Circle Search — configuration

[engines]
# Remove or reorder engines. Available: lens, yandex, bing, ai_chat, tineye, saucenao
enabled = [\"lens\", \"yandex\", \"bing\", \"ai_chat\", \"tineye\", \"saucenao\"]

[ai_chat]
url = \"https://chatgpt.com\"
name = \"ChatGPT\"
# paste_delay = 3   # seconds to wait for browser to load before auto-pasting

[upload]
# imgur_client_id = \"your_client_id_here\"   # override if the default hits rate limits
";
