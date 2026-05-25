# omarchy-circle-search

`omarchy-circle-search` is a small Wayland tool for Omarchy and Hyprland. You trigger it, draw around anything on screen, and it sends that crop to a reverse-image search engine or your AI chat of choice.

It is meant to feel like part of Omarchy rather than a separate app. It reads the current Omarchy theme, keeps its config under `~/.config/omarchy/`, and uses `omarchy-notification-send` for feedback.

This is for the moment when something on screen catches your eye and you want to ask: what is this, where is this from, or can I drop this straight into chat?

## What you get

- Freehand selection instead of a rigid rectangle
- Google Lens, Yandex, Bing, TinEye, SauceNao, and AI chat
- Clickable top-left engine picker plus keyboard shortcuts
- Omarchy theme colors out of the box
- Fast startup by grabbing the background screenshot before GTK finishes coming up

## Installation

### AUR

```bash
yay -S omarchy-circle-search-git
```

### From source

```bash
git clone https://github.com/Lambchwrlie95/omarchy-circle-search
cd omarchy-circle-search
cargo install --path . --root ~/.local
```

## Quick start

Add a Hyprland binding:

```ini
bindd = SUPER SHIFT, PRINT, Circle to Search, exec, omarchy-circle-search
```

Run it, then:

1. Pick an engine from the menu in the top-left corner, or press its shortcut.
2. Hold left mouse and draw around what you want.
3. Release to capture it and send it where you chose.

Press `Esc` any time to cancel.

Very small accidental selections are ignored, and the overlay gives a short shake instead of sending a bad crop.

## Engines

| Key | Action |
|-----|--------|
| `l` | Google Lens |
| `y` | Yandex Images |
| `b` | Bing Visual Search |
| `t` | TinEye |
| `s` | SauceNao |
| `c` | AI chat |
| `Esc` | Cancel |

## CLI

```bash
omarchy-circle-search --copy
omarchy-circle-search --notify
```

- `--copy` captures the selected region and copies it to the clipboard instead of searching
- `--notify` sends a completion notification after `--copy`

## Configuration

On first run the app creates:

`~/.config/omarchy/circle-search.toml`

```toml
[engines]
# Remove or reorder entries to control which pills appear and in what order.
# Available: lens, yandex, bing, ai_chat, tineye, saucenao
enabled = ["lens", "yandex", "bing", "ai_chat", "tineye", "saucenao"]

[ai_chat]
url = "https://chatgpt.com"
name = "ChatGPT"
# paste_delay = 3

[upload]
# imgur_client_id = "your_client_id_here"
```

- `engines.enabled` controls which engine pills appear, and in what order
- `ai_chat.url` is the site opened in AI mode
- `ai_chat.name` is the label source for the AI pill
- `ai_chat.paste_delay` is how long to wait before trying auto-paste
- `upload.imgur_client_id` overrides the bundled Imgur client ID if needed

Unknown sections and keys are ignored, so the file can grow later without breaking older setups.

## Dependencies

### Runtime

| Tool | Package | Why it is needed |
|------|---------|------------------|
| `grim` | `grim` | Capture fullscreen and cropped screenshots |
| `wl-copy` | `wl-clipboard` | Copy PNG data to the Wayland clipboard |
| `python3` | `python` | Run the async upload / AI-chat helper scripts |
| `curl` | `curl` | Upload captures to Imgur or `0x0.st` |
| `xdg-open` | `xdg-utils` | Open search results or chat URLs in your browser |
| `omarchy-notification-send` | `omarchy` | Show status and error notifications |
| `wtype` | `wtype` | Optional: auto-paste and submit in AI chat mode |

### Build

- Rust 1.85 or newer
- GTK4 development libraries
- `gtk4-layer-shell` development libraries

## Privacy and behavior

Visual search only works if the engine can fetch your image from a URL, so this app uploads the selected region to Imgur first and falls back to `https://0x0.st` if that fails. AI chat mode does not do that upload step; it copies the PNG to your clipboard and opens the configured chat URL instead.

## Project layout

- `src/main.rs` handles CLI flags, dependency checks, and top-level flow
- `src/overlay.rs` draws the GTK4 layer-shell overlay and handles input
- `src/capture.rs` wraps `grim` for the background screenshot and final cropped capture
- `src/search.rs` uploads the image or opens the AI chat flow
- `src/theme.rs` reads `~/.config/omarchy/current/theme/colors.toml`
- `src/config.rs` reads and writes `~/.config/omarchy/circle-search.toml`

## Development

```bash
cargo fmt
cargo check
cargo test
cargo clippy -- -D warnings
```

There are no tests in the repo yet, but the pure logic here is straightforward to cover. Geometry formatting, config parsing, theme parsing, and URL construction are the obvious places to start.

## License

MIT
