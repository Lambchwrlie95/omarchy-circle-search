use std::collections::HashMap;
use std::fs;

pub struct Theme {
    pub background: (f64, f64, f64),
    pub foreground: (f64, f64, f64),
    pub accent: (f64, f64, f64),
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: (0.13, 0.13, 0.13),
            foreground: (0.76, 0.76, 0.69),
            accent: (0.47, 0.51, 0.29),
        }
    }
}

fn hex_to_rgb(hex: &str) -> Option<(f64, f64, f64)> {
    let h = hex.trim().trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some((r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0))
}

pub fn load() -> Theme {
    let path = dirs_next::home_dir()
        .map(|h| h.join(".config/omarchy/current/theme/colors.toml"))
        .filter(|p| p.exists());

    let Some(path) = path else {
        return Theme::default();
    };
    let Ok(content) = fs::read_to_string(path) else {
        return Theme::default();
    };

    let map: HashMap<&str, &str> = content
        .lines()
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim(), v.trim().trim_matches('"')))
        .collect();

    let get = |key: &str, default: (f64, f64, f64)| {
        map.get(key).and_then(|v| hex_to_rgb(v)).unwrap_or(default)
    };

    Theme {
        background: get("background", (0.13, 0.13, 0.13)),
        foreground: get("foreground", (0.76, 0.76, 0.69)),
        accent: get("accent", (0.47, 0.51, 0.29)),
    }
}
