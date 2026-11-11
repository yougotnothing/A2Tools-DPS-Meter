//! Native macOS window management using AppKit.

use objc2_app_kit::{
    NSNormalWindowLevel, NSScreenSaverWindowLevel, NSWindow, NSWindowCollectionBehavior,
};
use std::ptr::NonNull;

/// Configure native NSWindow on the main thread.
///
/// AppKit window operations must happen on the main thread.
fn with_ns_window(
    window: &tauri::WebviewWindow,

    callback: impl FnOnce(&NSWindow) + Send + 'static,
) -> bool {
    let target = window.clone();

    window
        .run_on_main_thread(move || unsafe {
            let Ok(raw_window) = target.ns_window() else {
                return;
            };

            let Some(raw_window) = NonNull::new(raw_window.cast::<NSWindow>()) else {
                return;
            };

            callback(raw_window.as_ref());
        })
        .is_ok()
}

/// Configure the native window for overlay usage.
fn configure_overlay(ns_window: &NSWindow) {
    ns_window.setLevel(NSScreenSaverWindowLevel);

    let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
        | NSWindowCollectionBehavior::FullScreenAuxiliary;

    ns_window.setCollectionBehavior(behavior);

    ns_window.setHidesOnDeactivate(false);

    tracing::info!("macOS overlay configured: level={}", ns_window.level());
}

/// Restore the native window to its normal level.
fn reset_window_level(ns_window: &NSWindow) {
    ns_window.setLevel(NSNormalWindowLevel);
}

/// Returns the cursor position.
///
/// Tauri does not provide a reliable global cursor position
/// for this implementation.
pub fn cursor_position() -> Option<(i32, i32)> {
    None
}

/// No native foreground tracking.
pub fn user_focus_unchanged(_before: Option<isize>) -> bool {
    true
}

/// Activate the window following an explicit user request.
pub fn activate_from_user(window: &tauri::WebviewWindow, _before: Option<isize>) {
    let _ = window.set_focus();
}

/// Start native window dragging.
pub fn start_drag(window: &tauri::WebviewWindow) {
    let _ = window.start_dragging();
}

/// Show the DPS Meter above other windows without activating it.
pub fn show_on_top_without_focus(window: &tauri::WebviewWindow) {
    let _ = window.show();

    with_ns_window(window, |ns_window| {
        configure_overlay(ns_window);
        ns_window.orderFrontRegardless();
    });
}

/// Raise the overlay above foreign topmost windows.
pub fn rise_over_foreign_topmost(window: &tauri::WebviewWindow) -> bool {
    with_ns_window(window, |ns_window| {
        configure_overlay(ns_window);
        ns_window.orderFrontRegardless();
    })
}

/// Enable or disable native always-on-top behavior.
///
/// Does not show hidden windows.
pub fn set_on_top_without_showing(window: &tauri::WebviewWindow, on_top: bool) {
    with_ns_window(window, move |ns_window| {
        if on_top {
            configure_overlay(ns_window);
        } else {
            reset_window_level(ns_window);
        }
    });
}

/// Bring the window forward without requesting application focus.
pub fn bring_forward(window: &tauri::WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();

    with_ns_window(window, |ns_window| {
        configure_overlay(ns_window);
        ns_window.orderFrontRegardless();
    });
}

/// Hide the window without changing application focus.
pub fn hide_without_focus(window: &tauri::WebviewWindow) {
    let _ = window.hide();
}

/// Do not hide the window during construction.
pub const HIDE_AFTER_BUILD: bool = false;

/// Remove topmost status before minimizing.
pub fn minimize_off_top(window: &tauri::WebviewWindow) {
    set_on_top_without_showing(window, false);
    let _ = window.minimize();
}

/// Set window size.
pub fn set_size(window: &tauri::WebviewWindow, size: tauri::Size) {
    let _ = window.set_size(size);
}

/// Set window size and position.
pub fn set_size_and_position(
    window: &tauri::WebviewWindow,
    size: tauri::Size,
    pos: tauri::PhysicalPosition<i32>,
) {
    let _ = window.set_size(size);
    let _ = window.set_position(tauri::Position::Physical(pos));
}

/// Linux-only functionality.
pub fn resizable_edges(_window: &tauri::WebviewWindow, _min: tauri::LogicalSize<f64>) {}

pub fn release_size(_window: &tauri::WebviewWindow, _min: tauri::LogicalSize<f64>) {}

pub fn primary_button_down() -> Option<bool> {
    None
}

pub fn compositor_resize_supported(_window: &tauri::WebviewWindow) -> bool {
    false
}

pub fn resize_pointer_down(_window: &tauri::WebviewWindow) -> Option<bool> {
    None
}

pub async fn prepare_resize(
    _window: &tauri::WebviewWindow,
    _min: tauri::LogicalSize<f64>,
) -> Result<(), String> {
    Err("Native compositor resize is unavailable".into())
}

/// Click-through input region is handled by app.rs.
pub fn click_through_except(
    _window: &tauri::WebviewWindow,
    _ignore: bool,
    _keep: Option<(f64, f64, f64, f64, f64)>,
) -> bool {
    false
}

/// Hide a window while preserving its position.
pub fn hide_in_place(window: &tauri::WebviewWindow) {
    hide_without_focus(window);
}

pub fn places_own_windows() -> bool {
    true
}

pub fn known_position(_window: &tauri::WebviewWindow) -> Option<(i32, i32)> {
    None
}

pub fn set_main_anchor(_anchor: crate::platform::meter_anchor::Anchor) {}

pub async fn anchor_next_resize(_anchored: bool) {}

pub fn meter_anchor_available() -> bool {
    true
}

pub fn keep_places(
    _save: impl Fn(&str, (i32, i32, i32, i32)) + Send + Sync + 'static,
    _load: impl Fn(&str) -> Option<(i32, i32, i32, i32)> + Send + Sync + 'static,
) {
}

pub fn is_hidden_in_place(_window: &tauri::WebviewWindow) -> bool {
    false
}

pub fn reveal_in_place(_window: &tauri::WebviewWindow) -> bool {
    false
}

pub fn front_window() -> Option<isize> {
    None
}

pub fn front_is_ours_or_game() -> bool {
    true
}

pub fn never_activate(_window: &tauri::WebviewWindow, _before: Option<isize>) {}

pub fn give_front_back(_window: &tauri::WebviewWindow, _before: Option<isize>) {}

pub fn tray_supported() -> bool {
    false
}

pub fn click_through_by_region() -> bool {
    false
}

pub fn widget_click_through(
    _window: &tauri::WebviewWindow,
    _ignore: bool,
    _keep: Option<(&[(f64, f64, f64, f64)], f64)>,
) -> bool {
    false
}

pub fn share_process<'a, R: tauri::Runtime, M: tauri::Manager<R>>(
    builder: tauri::WebviewWindowBuilder<'a, R, M>,
    _group: crate::platform::ProcessGroup,
) -> tauri::WebviewWindowBuilder<'a, R, M> {
    builder
}

pub fn joined(_window: &tauri::WebviewWindow, _group: crate::platform::ProcessGroup) {}
