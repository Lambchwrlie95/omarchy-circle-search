use crate::config::Config;
use crate::overlay::Engine;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn open_search(image_path: &Path, engine: Engine, config: &Config) -> Result<()> {
    let path_str = image_path.to_string_lossy().to_string();

    match engine {
        Engine::AiChat => open_ai_chat(&path_str, config),
        _ => open_visual_search(&path_str, engine, &config.imgur_client_id),
    }
}

fn open_visual_search(path_str: &str, engine: Engine, imgur_client_id: &str) -> Result<()> {
    let (engine_name, url_builder) = match engine {
        Engine::Lens => ("Google Lens", "lens"),
        Engine::Yandex => ("Yandex Images", "yandex"),
        Engine::Bing => ("Bing Visual", "bing"),
        Engine::TinEye => ("TinEye", "tineye"),
        Engine::SauceNao => ("SauceNao", "saucenao"),
        Engine::AiChat => unreachable!(),
    };

    let script = r#"
import subprocess, json, urllib.parse, os

path = os.environ["CS_PATH"]
engine = os.environ["CS_ENGINE"]
name = os.environ["CS_ENGINE_NAME"]
imgur_client_id = os.environ.get("CS_IMGUR_CLIENT_ID", "")

fallback_urls = {
    "lens":     "https://lens.google.com/",
    "yandex":   "https://yandex.com/images/",
    "bing":     "https://www.bing.com/visualsearch",
    "tineye":   "https://tineye.com/",
    "saucenao": "https://saucenao.com/",
}
fallback = fallback_urls.get(engine, "https://lens.google.com/")

def build_url(image_url):
    encoded = urllib.parse.quote(image_url, safe="")
    if engine == "lens":
        return f"https://lens.google.com/uploadbyurl?url={encoded}"
    elif engine == "yandex":
        return f"https://yandex.com/images/search?rpt=imageview&url={encoded}"
    elif engine == "bing":
        return f"https://www.bing.com/images/search?view=detailv2&iss=sbi&q=imgurl:{encoded}"
    elif engine == "tineye":
        return f"https://tineye.com/search?url={encoded}"
    elif engine == "saucenao":
        return f"https://saucenao.com/search.php?url={encoded}"

subprocess.run(["omarchy-notification-send", "", "Circle Search", f"Uploading to {name}...", "-t", "4000"])

image_url = None

try:
    r = subprocess.run([
        "curl", "-s", "-X", "POST",
        "-H", f"Authorization: Client-ID {imgur_client_id}",
        "-F", f"image=@{path}",
        "https://api.imgur.com/3/image"
    ], capture_output=True, text=True, timeout=60)
    resp = json.loads(r.stdout) if r.stdout else {}
    if resp.get("success") and resp.get("data", {}).get("link"):
        image_url = resp["data"]["link"]
except Exception:
    pass

if not image_url:
    try:
        r2 = subprocess.run(
            ["curl", "-s", "-F", f"file=@{path}", "https://0x0.st"],
            capture_output=True, text=True, timeout=60
        )
        candidate = r2.stdout.strip()
        if candidate.startswith("http"):
            image_url = candidate
    except Exception:
        pass

try:
    if image_url:
        url = build_url(image_url)
        subprocess.run(["omarchy-notification-send", "", "Circle Search", f"Opening {name}...", "-t", "2000"])
        subprocess.Popen(["xdg-open", url])
    else:
        subprocess.run(["omarchy-notification-send", "", "Circle Search", "Upload failed — check your connection"])
        subprocess.Popen(["xdg-open", fallback])
finally:
    try:
        os.remove(path)
    except:
        pass
"#;

    let log_path = std::env::temp_dir().join("circle-search.log");
    let log = std::fs::File::create(&log_path).ok();
    let (out, err) = log
        .map(|f| {
            let f2 = f.try_clone().ok();
            (f.into(), f2.map(Into::into).unwrap_or_else(Stdio::null))
        })
        .unwrap_or_else(|| (Stdio::null(), Stdio::null()));
    Command::new("python3")
        .args(["-c", script])
        .env("CS_PATH", path_str)
        .env("CS_ENGINE", url_builder)
        .env("CS_ENGINE_NAME", engine_name)
        .env("CS_IMGUR_CLIENT_ID", imgur_client_id)
        .stdout(out)
        .stderr(err)
        .spawn()
        .context("failed to spawn upload process")?;

    Ok(())
}

fn open_ai_chat(path_str: &str, config: &Config) -> Result<()> {
    let script = r#"
import subprocess, time, os

path = os.environ["CS_PATH"]
ai_url = os.environ["CS_AI_URL"]
ai_name = os.environ["CS_AI_NAME"]
paste_delay = int(os.environ.get("CS_PASTE_DELAY", "3"))

try:
    with open(path, "rb") as f:
        subprocess.run(["wl-copy", "--type", "image/png"], stdin=f, check=True)
    subprocess.Popen(["xdg-open", ai_url])

    time.sleep(paste_delay)

    try:
        subprocess.run(["wtype", "-M", "ctrl", "-k", "v", "-m", "ctrl"], timeout=5)
        time.sleep(0.5)
        subprocess.run(["wtype", "-k", "Return"], timeout=5)
        subprocess.run(["omarchy-notification-send", "", "Circle Search", f"Pasted into {ai_name}", "-t", "3000"])
    except Exception:
        subprocess.run(["omarchy-notification-send", "", "Circle Search",
            f"Auto-paste failed. Image copied — open {ai_name} and paste (Ctrl+V).", "-t", "6000"])
except Exception as e:
    subprocess.run(["omarchy-notification-send", "", "Circle Search", str(e)])
finally:
    try:
        os.remove(path)
    except:
        pass
"#;

    Command::new("python3")
        .args(["-c", script])
        .env("CS_PATH", path_str)
        .env("CS_AI_URL", &config.ai_chat_url)
        .env("CS_AI_NAME", &config.ai_chat_name)
        .env("CS_PASTE_DELAY", config.paste_delay_secs.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to spawn ai-chat process")?;

    Ok(())
}
