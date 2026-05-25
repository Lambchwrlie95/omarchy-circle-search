use crate::capture::{Region, spawn_fullscreen_capture};
use crate::config::Config;
use crate::theme::Theme;
use anyhow::{Context, Result};
use gtk4::glib::MainLoop;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::f64::consts::TAU;
use std::rc::Rc;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub enum Engine {
    #[default]
    Lens,
    Yandex,
    Bing,
    AiChat,
    TinEye,
    SauceNao,
}

// ── Menu layout constants ────────────────────────────────────────────
const TOP_OFF: f64 = 27.0;

const MENU_ICON_CX: f64 = 18.0;
const MENU_ICON_CY: f64 = 18.0 + TOP_OFF;
const MENU_ICON_R: f64 = 12.0;
const PILL_W: f64 = 54.0;
const PILL_H: f64 = 22.0;
const PILL_GAP: f64 = 6.0;

const ALL_ENGINES: &[(&str, Engine)] = &[
    ("lens", Engine::Lens),
    ("yandex", Engine::Yandex),
    ("bing", Engine::Bing),
    ("ai_chat", Engine::AiChat),
    ("tineye", Engine::TinEye),
    ("saucenao", Engine::SauceNao),
];

fn resolve_engines(enabled: &[String]) -> Vec<Engine> {
    let list: Vec<Engine> = enabled
        .iter()
        .filter_map(|name| {
            ALL_ENGINES
                .iter()
                .find(|(n, _)| *n == name.as_str())
                .map(|(_, e)| *e)
        })
        .collect();
    if list.is_empty() {
        ALL_ENGINES.iter().map(|(_, e)| *e).collect()
    } else {
        list
    }
}

fn selected_engine(engines: &[Engine]) -> Engine {
    engines.first().copied().unwrap_or_default()
}

fn shortcut_engine(key: gtk4::gdk::Key, engines: &[Engine]) -> Option<Engine> {
    let engine = match key {
        gtk4::gdk::Key::l => Engine::Lens,
        gtk4::gdk::Key::y => Engine::Yandex,
        gtk4::gdk::Key::b => Engine::Bing,
        gtk4::gdk::Key::c => Engine::AiChat,
        gtk4::gdk::Key::t => Engine::TinEye,
        gtk4::gdk::Key::s => Engine::SauceNao,
        _ => return None,
    };

    engines.contains(&engine).then_some(engine)
}

fn pill_rect(i: usize) -> (f64, f64, f64, f64) {
    let x = MENU_ICON_CX + MENU_ICON_R + 10.0 + i as f64 * (PILL_W + PILL_GAP);
    let y = MENU_ICON_CY - PILL_H / 2.0;
    (x, y, PILL_W, PILL_H)
}

fn pill_hit(mx: f64, my: f64, i: usize) -> bool {
    let (px, py, pw, ph) = pill_rect(i);
    mx >= px && mx <= px + pw && my >= py && my <= py + ph
}

fn hit_engine(mx: f64, my: f64, engines: &[Engine]) -> Option<Engine> {
    engines
        .iter()
        .enumerate()
        .find(|(i, _)| pill_hit(mx, my, *i))
        .map(|(_, e)| *e)
}

fn in_menu_area(mx: f64, my: f64, engines: &[Engine]) -> bool {
    let icon_hit = (mx - MENU_ICON_CX).hypot(my - MENU_ICON_CY) <= MENU_ICON_R + 4.0;
    icon_hit || (0..engines.len()).any(|i| pill_hit(mx, my, i))
}

struct Colors {
    bg: (f64, f64, f64),
    fg: (f64, f64, f64),
    ac: (f64, f64, f64),
}

pub fn show_selector(theme: &Theme, config: &Config) -> Result<(Region, Engine)> {
    // Spawn grim early so it captures while GTK initialises in parallel.
    let (grim_child, screenshot_path) = spawn_fullscreen_capture()?;

    gtk4::init().context("failed to initialize GTK")?;

    let bg_surface: Rc<RefCell<Option<cairo::ImageSurface>>> = Rc::new(RefCell::new(None));
    let result: Rc<RefCell<Option<(Region, Engine)>>> = Rc::new(RefCell::new(None));

    let window = gtk4::Window::new();
    window.set_title(Some("Circle Search"));
    window.set_decorated(false);

    use gtk4_layer_shell::{KeyboardMode, LayerShell};
    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Overlay);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_exclusive_zone(-1);
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    let area = gtk4::DrawingArea::new();
    area.set_can_focus(true);
    let area_focus = area.clone();
    area.connect_realize(move |_| {
        area_focus.grab_focus();
    });

    let (bg_r, bg_g, bg_b) = theme.background;
    let (fg_r, fg_g, fg_b) = theme.foreground;
    let (ac_r, ac_g, ac_b) = theme.accent;

    let engines: Vec<Engine> = resolve_engines(&config.enabled_engines);
    let state = Rc::new(RefCell::new(DrawState {
        engine: selected_engine(&engines),
        ..Default::default()
    }));
    let ai_label: String = config
        .ai_chat_name
        .chars()
        .take(4)
        .collect::<String>()
        .to_lowercase();

    let state_draw = state.clone();
    let bg_draw = bg_surface.clone();
    let ai_label_draw = ai_label.clone();
    let engines_draw = engines.clone();
    area.set_draw_func(move |_area, cr, width, _height| {
        let st = state_draw.borrow();
        let bg = bg_draw.borrow();

        // Background: screenshot when loaded, theme colour while loading
        if let Some(surface) = bg.as_ref() {
            cr.set_source_surface(surface, 0.0, 0.0).ok();
        } else {
            cr.set_source_rgba(bg_r, bg_g, bg_b, 1.0);
        }
        cr.paint().ok();

        let w = width as f64;

        // ── Instruction label ──────────────────────────────────────────
        if st.points.is_empty() {
            cr.select_font_face(
                "monospace",
                cairo::FontSlant::Normal,
                cairo::FontWeight::Normal,
            );
            cr.set_font_size(12.0);
            let text = "choose engine  ·  draw to search  ·  esc to cancel";
            if let Ok(ext) = cr.text_extents(text) {
                let pad = 10.0;
                let tx = (w - ext.width()) / 2.0;
                let box_top = 28.0;
                let ty = box_top + pad + ext.height();
                cr.set_source_rgba(bg_r, bg_g, bg_b, 1.0);
                cr.rectangle(
                    tx - pad,
                    box_top,
                    ext.width() + pad * 2.0,
                    ext.height() + pad * 2.0,
                );
                cr.fill().ok();
                cr.set_source_rgba(fg_r, fg_g, fg_b, 1.0);
                cr.move_to(tx, ty);
                cr.show_text(text).ok();
            }
        }

        if !st.points.is_empty() {
            // ── Dim overlay ────────────────────────────────────────────
            cr.set_source_rgba(bg_r, bg_g, bg_b, 0.45);
            cr.paint().ok();

            let ox = st.shake_offset;

            let path = if st.smoothed.is_empty() {
                &st.points
            } else {
                &st.smoothed
            };

            if path.len() > 1 {
                // Glow
                cr.set_line_width(8.0);
                cr.set_source_rgba(ac_r, ac_g, ac_b, 0.25);
                draw_path(cr, path, ox, st.closed);
                cr.stroke().ok();

                // Main stroke
                cr.set_line_width(2.5);
                cr.set_source_rgba(ac_r, ac_g, ac_b, 1.0);
                draw_path(cr, path, ox, st.closed);
                cr.stroke().ok();

                // Dashed closing preview
                if !st.closed && st.points.len() > 2 {
                    let (sx, sy) = st.points[0];
                    let (ex, ey) = *st.points.last().unwrap();
                    cr.set_dash(&[6.0, 4.0], 0.0);
                    cr.set_line_width(1.5);
                    cr.set_source_rgba(ac_r, ac_g, ac_b, 0.4);
                    cr.move_to(ex + ox, ey);
                    cr.line_to(sx + ox, sy);
                    cr.stroke().ok();
                    cr.set_dash(&[], 0.0);
                }
            }

            // Start dot
            let (sx, sy) = st.points[0];
            cr.set_source_rgba(ac_r, ac_g, ac_b, 0.4);
            cr.arc(sx + ox, sy, 8.0, 0.0, TAU);
            cr.fill().ok();
            cr.set_source_rgba(ac_r, ac_g, ac_b, 1.0);
            cr.arc(sx + ox, sy, 4.0, 0.0, TAU);
            cr.fill().ok();
        }

        // ── Engine-select menu (always on top) ─────────────────────────
        draw_engine_menu(
            cr,
            &st,
            &Colors {
                bg: (bg_r, bg_g, bg_b),
                fg: (fg_r, fg_g, fg_b),
                ac: (ac_r, ac_g, ac_b),
            },
            &ai_label_draw,
            &engines_draw,
        );
    });

    // ── Poll for grim completion, load PNG, trigger redraw ─────────────
    {
        let bg_ref = bg_surface.clone();
        let area_ref = area.clone();
        let path_ref = screenshot_path.clone();
        let grim_cell: Rc<RefCell<Option<std::process::Child>>> =
            Rc::new(RefCell::new(Some(grim_child)));
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(20), move || {
            let done = grim_cell
                .borrow_mut()
                .as_mut()
                .and_then(|c| c.try_wait().ok())
                .flatten()
                .is_some();
            if done {
                if let Ok(mut file) = std::fs::File::open(&path_ref)
                    && let Ok(surface) = cairo::ImageSurface::create_from_png(&mut file)
                {
                    *bg_ref.borrow_mut() = Some(surface);
                    area_ref.queue_draw();
                }
                gtk4::glib::ControlFlow::Break
            } else {
                gtk4::glib::ControlFlow::Continue
            }
        });
    }

    // ── Mouse motion for menu hover ────────────────────────────────────
    let state_hov1 = state.clone();
    let area_hov1 = area.clone();
    let state_hov2 = state.clone();
    let area_hov2 = area.clone();
    let engines_hov = engines.clone();
    let motion = gtk4::EventControllerMotion::new();
    motion.connect_motion(move |_, x, y| {
        if let Ok(mut st) = state_hov1.try_borrow_mut() {
            st.hovered_engine = hit_engine(x, y, &engines_hov);
            area_hov1.queue_draw();
        }
    });
    motion.connect_leave(move |_| {
        if let Ok(mut st) = state_hov2.try_borrow_mut()
            && st.hovered_engine.is_some()
        {
            st.hovered_engine = None;
            area_hov2.queue_draw();
        }
    });
    area.add_controller(motion);

    // ── Click on menu pills to select engine ───────────────────────────
    let state_click = state.clone();
    let area_click = area.clone();
    let engines_click = engines.clone();
    let click = gtk4::GestureClick::new();
    click.set_button(1);
    click.connect_pressed(move |_, _n, x, y| {
        if let Some(engine) = hit_engine(x, y, &engines_click) {
            state_click.borrow_mut().engine = engine;
            area_click.queue_draw();
        }
    });
    area.add_controller(click);

    let state_press = state.clone();
    let area_press = area.clone();
    let engines_begin = engines.clone();
    let engines_end = engines.clone();
    let gesture = gtk4::GestureDrag::new();
    gesture.set_button(1);

    gesture.connect_drag_begin(move |g, x, y| {
        if in_menu_area(x, y, &engines_begin) {
            if let Some(seq) = g.last_updated_sequence() {
                g.set_sequence_state(&seq, gtk4::EventSequenceState::Denied);
            }
            return;
        }
        let mut st = state_press.borrow_mut();
        st.points.clear();
        st.smoothed.clear();
        st.closed = false;
        st.shake_offset = 0.0;
        st.points.push((x, y));
        st.smoothed = st.points.clone();
        area_press.queue_draw();
    });

    let state_upd = state.clone();
    let area_upd = area.clone();
    gesture.connect_drag_update(move |g, ox, oy| {
        let (sx, sy) = g.start_point().unwrap_or((0.0, 0.0));
        let mut st = state_upd.borrow_mut();
        let cx = sx + ox;
        let cy = sy + oy;
        if st.points.is_empty() {
            st.points.push((sx, sy));
        }

        let should_push = match st.points.last() {
            Some(&(lx, ly)) if st.points.len() > 1 => (cx - lx).hypot(cy - ly) > 2.0,
            Some(_) => true,
            None => true,
        };

        if should_push {
            st.points.push((cx, cy));
            st.smoothed = catmull_rom(&st.points, false);
        }
        area_upd.queue_draw();
    });

    let state_end = state.clone();
    let area_end = area.clone();
    let result_end = result.clone();
    let window_end = window.clone();
    gesture.connect_drag_end(move |g, ox, oy| {
        let (sx, sy) = g.start_point().unwrap_or((0.0, 0.0));

        if in_menu_area(sx, sy, &engines_end) {
            let mut st = state_end.borrow_mut();
            st.points.clear();
            st.smoothed.clear();
            st.closed = false;
            return;
        }

        {
            let mut st = state_end.borrow_mut();
            st.points.push((sx + ox, sy + oy));
            st.closed = true;
            if st.points.len() > 1 {
                st.smoothed = catmull_rom(&st.points, true);
            }
        }

        let st = state_end.borrow();
        if st.points.len() < 2 {
            return;
        }

        let xs: Vec<f64> = st.points.iter().map(|p| p.0).collect();
        let ys: Vec<f64> = st.points.iter().map(|p| p.1).collect();
        let pad = 8.0;
        let x = (xs.iter().cloned().fold(f64::INFINITY, f64::min) - pad).max(0.0) as i32;
        let y = (ys.iter().cloned().fold(f64::INFINITY, f64::min) - pad).max(0.0) as i32;
        let x2 = (xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max) + pad) as u32;
        let y2 = (ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max) + pad) as u32;
        let w = x2.saturating_sub(x as u32);
        let h = y2.saturating_sub(y as u32);

        if w < 10 || h < 10 {
            drop(st);
            let state_shake = state_end.clone();
            let area_shake = area_end.clone();
            let mut tick = 0i32;
            gtk4::glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                tick += 1;
                let offset = (tick as f64 * 1.8).sin() * 8.0 * (1.0 - tick as f64 / 20.0);
                state_shake.borrow_mut().shake_offset = offset;
                area_shake.queue_draw();
                if tick >= 20 {
                    let mut st = state_shake.borrow_mut();
                    st.shake_offset = 0.0;
                    st.points.clear();
                    st.smoothed.clear();
                    area_shake.queue_draw();
                    gtk4::glib::ControlFlow::Break
                } else {
                    gtk4::glib::ControlFlow::Continue
                }
            });
            return;
        }

        *result_end.borrow_mut() = Some((
            Region {
                x,
                y,
                width: w,
                height: h,
            },
            st.engine,
        ));
        area_end.queue_draw();
        window_end.close();
    });

    area.add_controller(gesture);

    // ── Key events ────────────────────────────────────────────────────
    let window_key = window.clone();
    let state_key = state.clone();
    let area_key = area.clone();
    let engines_key = engines.clone();
    let key_ctrl = gtk4::EventControllerKey::new();
    key_ctrl.set_propagation_phase(gtk4::PropagationPhase::Capture);
    key_ctrl.connect_key_pressed(move |_, key, _, _| match key {
        gtk4::gdk::Key::Escape => {
            window_key.close();
            gtk4::glib::Propagation::Stop
        }
        _ => {
            let Some(engine) = shortcut_engine(key, &engines_key) else {
                return gtk4::glib::Propagation::Proceed;
            };
            state_key.borrow_mut().engine = engine;
            area_key.queue_draw();
            gtk4::glib::Propagation::Stop
        }
    });
    window.add_controller(key_ctrl);

    window.set_child(Some(&area));
    window.present();
    window.grab_focus();

    let main_loop = MainLoop::new(None, false);
    let ml = main_loop.clone();
    window.connect_close_request(move |_| {
        ml.quit();
        gtk4::glib::Propagation::Proceed
    });
    main_loop.run();

    let _ = std::fs::remove_file(&screenshot_path);

    result
        .borrow_mut()
        .take()
        .ok_or_else(|| anyhow::anyhow!("selection cancelled"))
}

fn catmull_rom(points: &[(f64, f64)], closed: bool) -> Vec<(f64, f64)> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let n = points.len();
    let mut result = Vec::with_capacity(n * 8);

    let get = |i: i64| -> (f64, f64) {
        if closed {
            points[i.rem_euclid(n as i64) as usize]
        } else {
            points[i.clamp(0, n as i64 - 1) as usize]
        }
    };

    let steps = 8usize;
    let end = if closed { n } else { n - 1 };
    for i in 0..end {
        let p0 = get(i as i64 - 1);
        let p1 = get(i as i64);
        let p2 = get(i as i64 + 1);
        let p3 = get(i as i64 + 2);
        for s in 0..steps {
            let t = s as f64 / steps as f64;
            let t2 = t * t;
            let t3 = t2 * t;
            let x = 0.5
                * ((2.0 * p1.0)
                    + (-p0.0 + p2.0) * t
                    + (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2
                    + (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3);
            let y = 0.5
                * ((2.0 * p1.1)
                    + (-p0.1 + p2.1) * t
                    + (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2
                    + (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3);
            result.push((x, y));
        }
    }
    result
}

fn draw_path(cr: &cairo::Context, points: &[(f64, f64)], ox: f64, closed: bool) {
    if points.is_empty() {
        return;
    }
    cr.move_to(points[0].0 + ox, points[0].1);
    for &(x, y) in &points[1..] {
        cr.line_to(x + ox, y);
    }
    if closed {
        cr.close_path();
    }
}

fn draw_engine_menu(
    cr: &cairo::Context,
    st: &DrawState,
    c: &Colors,
    ai_label: &str,
    engines: &[Engine],
) {
    let (bg_r, bg_g, bg_b) = c.bg;
    let (fg_r, fg_g, fg_b) = c.fg;
    let (ac_r, ac_g, ac_b) = c.ac;

    // ── Icon circle ────────────────────────────────────────────────────
    cr.set_source_rgba(bg_r, bg_g, bg_b, 1.0);
    cr.arc(MENU_ICON_CX, MENU_ICON_CY, MENU_ICON_R, 0.0, TAU);
    cr.fill().ok();
    cr.set_source_rgba(fg_r, fg_g, fg_b, 0.6);
    cr.set_line_width(1.0);
    cr.arc(MENU_ICON_CX, MENU_ICON_CY, MENU_ICON_R, 0.0, TAU);
    cr.stroke().ok();

    // ── Magnifying glass icon ──────────────────────────────────────────
    cr.set_source_rgba(fg_r, fg_g, fg_b, 1.0);
    cr.set_line_width(1.5);
    cr.arc(MENU_ICON_CX - 1.5, MENU_ICON_CY - 1.5, 4.5, 0.0, TAU);
    cr.stroke().ok();
    cr.move_to(MENU_ICON_CX + 2.5, MENU_ICON_CY + 2.5);
    cr.line_to(MENU_ICON_CX + 7.0, MENU_ICON_CY + 7.0);
    cr.stroke().ok();

    // ── Engine pills ──────────────────────────────────────────────────
    cr.select_font_face(
        "monospace",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Normal,
    );
    cr.set_font_size(11.0);
    for (i, eng) in engines.iter().enumerate() {
        let (px, py, pw, ph) = pill_rect(i);
        let is_active = *eng == st.engine;
        let is_hovered = st.hovered_engine == Some(*eng);

        if is_active {
            cr.set_source_rgba(ac_r, ac_g, ac_b, 1.0);
        } else if is_hovered {
            cr.set_source_rgba(fg_r, fg_g, fg_b, 0.25);
        } else {
            cr.set_source_rgba(bg_r, bg_g, bg_b, 1.0);
        }

        cr.rectangle(px, py, pw, ph);

        cr.fill_preserve().ok();
        let (border_r, border_g, border_b, border_a) = if is_active {
            (ac_r, ac_g, ac_b, 1.0)
        } else if is_hovered {
            (fg_r, fg_g, fg_b, 0.6)
        } else {
            (fg_r, fg_g, fg_b, 0.2)
        };
        cr.set_source_rgba(border_r, border_g, border_b, border_a);
        cr.set_line_width(1.0);
        cr.stroke().ok();

        let name = match eng {
            Engine::Lens => "lens",
            Engine::Yandex => "yandex",
            Engine::Bing => "bing",
            Engine::AiChat => ai_label,
            Engine::TinEye => "tineye",
            Engine::SauceNao => "sauce",
        };

        if let Ok(ext) = cr.text_extents(name) {
            let (tc, tg, tb) = if is_active {
                (bg_r, bg_g, bg_b)
            } else {
                (fg_r, fg_g, fg_b)
            };
            cr.set_source_rgba(tc, tg, tb, 1.0);
            cr.move_to(
                px + (pw - ext.width()) / 2.0 - ext.x_bearing(),
                py + (ph + ext.height()) / 2.0 - ext.y_bearing() - ext.height(),
            );
            cr.show_text(name).ok();
        }
    }
}

#[derive(Default)]
struct DrawState {
    points: Vec<(f64, f64)>,
    smoothed: Vec<(f64, f64)>,
    closed: bool,
    shake_offset: f64,
    engine: Engine,
    hovered_engine: Option<Engine>,
}

#[cfg(test)]
mod tests {
    use super::{Engine, resolve_engines, selected_engine, shortcut_engine};

    #[test]
    fn resolves_enabled_engines_in_config_order() {
        let enabled = vec![
            "saucenao".to_string(),
            "bing".to_string(),
            "lens".to_string(),
        ];
        assert_eq!(
            resolve_engines(&enabled),
            vec![Engine::SauceNao, Engine::Bing, Engine::Lens]
        );
    }

    #[test]
    fn defaults_to_first_enabled_engine() {
        let engines = vec![Engine::TinEye, Engine::AiChat];
        assert_eq!(selected_engine(&engines), Engine::TinEye);
    }

    #[test]
    fn ignores_shortcuts_for_hidden_engines() {
        let engines = vec![Engine::AiChat, Engine::TinEye];
        assert_eq!(
            shortcut_engine(gtk4::gdk::Key::c, &engines),
            Some(Engine::AiChat)
        );
        assert_eq!(shortcut_engine(gtk4::gdk::Key::l, &engines), None);
    }
}
