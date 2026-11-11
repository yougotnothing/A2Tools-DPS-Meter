//! macOS global shortcuts, using the native event loop through `global-hotkey`.
//!
//! `GlobalHotKeyManager` must be created while the application's main event
//! loop is running on the main thread. `HotkeyManager::new` is constructed
//! during Tauri setup, before the app starts dispatching commands.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use tracing::{info, warn};

pub const DEFAULT_ON: bool = true;

pub struct HotkeyManager {
    manager: Option<GlobalHotKeyManager>,
    listener: Mutex<Option<(Arc<AtomicBool>, JoinHandle<()>, Vec<HotKey>)>>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        // Tauri constructs this state during setup on its main thread. On
        // macOS the native shortcut backend must be initialized there.
        let manager = match GlobalHotKeyManager::new() {
            Ok(manager) => Some(manager),
            Err(error) => {
                warn!("Could not initialize macOS global hotkeys: {error}");
                None
            }
        };
        Self { manager, listener: Mutex::new(None) }
    }

    pub fn start(
        &self,
        bindings: Vec<crate::platform::hotkeys::Binding>,
        on_status: impl Fn(&'static str, &'static str, Option<String>) + Send + 'static,
    ) {
        let mut listener = self.listener.lock().unwrap_or_else(|p| p.into_inner());
        if listener.as_ref().is_some_and(|(running, _, _)| running.load(Ordering::SeqCst)) {
            return;
        }
        let Some(manager) = self.manager.as_ref() else {
            for binding in &bindings {
                if binding.vk != 0 { on_status(binding.name, "taken", None); }
            }
            return;
        };

        let mut by_id = HashMap::new();
        let mut registered = Vec::new();
        for binding in bindings {
            if binding.vk == 0 { continue; }
            let Some(hotkey) = make_hotkey(binding.mods, binding.vk) else {
                warn!("Unsupported macOS hotkey for {}: mods={:#x} vk={:#x}", binding.name, binding.mods, binding.vk);
                on_status(binding.name, "taken", None);
                continue;
            };
            match manager.register(hotkey) {
                Ok(()) => {
                    info!("macOS global hotkey registered: {} ({})", binding.name, hotkey.into_string());
                    by_id.insert(hotkey.id(), (binding.name, binding.on_fire));
                    registered.push(hotkey);
                    on_status(binding.name, "ok", None);
                }
                Err(error) => {
                    warn!("Could not register macOS hotkey {}: {error}", binding.name);
                    on_status(binding.name, "taken", None);
                }
            }
        }

        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();
        let thread = std::thread::spawn(move || {
            let receiver = GlobalHotKeyEvent::receiver();
            while thread_running.load(Ordering::SeqCst) {
                match receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(event) if event.state == HotKeyState::Pressed => {
                        if let Some((_, callback)) = by_id.get(&event.id) { callback(); }
                    }
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        });
        *listener = Some((running, thread, registered));
    }

    pub fn open_desktop_settings(&self) -> Result<(), String> {
        Err("Global shortcuts are configured in the meter's Settings → Keybinds on macOS".into())
    }

    pub fn stop(&self) {
        let entry = self.listener.lock().unwrap_or_else(|p| p.into_inner()).take();
        if let Some((running, thread, registered)) = entry {
            running.store(false, Ordering::SeqCst);
            let _ = thread.join();
            if let Some(manager) = self.manager.as_ref() {
                for hotkey in registered {
                    if let Err(error) = manager.unregister(hotkey) {
                        warn!("Could not unregister macOS hotkey {}: {error}", hotkey.into_string());
                    }
                }
            }
        }
    }
}

fn make_hotkey(mods: u32, vk: u32) -> Option<HotKey> {
    let key = match vk {
        0x30 => Code::Digit0, 0x31 => Code::Digit1, 0x32 => Code::Digit2,
        0x33 => Code::Digit3, 0x34 => Code::Digit4, 0x35 => Code::Digit5,
        0x36 => Code::Digit6, 0x37 => Code::Digit7, 0x38 => Code::Digit8,
        0x39 => Code::Digit9,
        0x41 => Code::KeyA, 0x42 => Code::KeyB, 0x43 => Code::KeyC,
        0x44 => Code::KeyD, 0x45 => Code::KeyE, 0x46 => Code::KeyF,
        0x47 => Code::KeyG, 0x48 => Code::KeyH, 0x49 => Code::KeyI,
        0x4A => Code::KeyJ, 0x4B => Code::KeyK, 0x4C => Code::KeyL,
        0x4D => Code::KeyM, 0x4E => Code::KeyN, 0x4F => Code::KeyO,
        0x50 => Code::KeyP, 0x51 => Code::KeyQ, 0x52 => Code::KeyR,
        0x53 => Code::KeyS, 0x54 => Code::KeyT, 0x55 => Code::KeyU,
        0x56 => Code::KeyV, 0x57 => Code::KeyW, 0x58 => Code::KeyX,
        0x59 => Code::KeyY, 0x5A => Code::KeyZ,
        0x08 => Code::Backspace, 0x09 => Code::Tab, 0x0D => Code::Enter,
        0x1B => Code::Escape, 0x20 => Code::Space,
        0x21 => Code::PageUp, 0x22 => Code::PageDown,
        0x23 => Code::End, 0x24 => Code::Home,
        0x25 => Code::ArrowLeft, 0x26 => Code::ArrowUp,
        0x27 => Code::ArrowRight, 0x28 => Code::ArrowDown,
        0x2D => Code::Insert, 0x2E => Code::Delete,
        0x70 => Code::F1, 0x71 => Code::F2, 0x72 => Code::F3,
        0x73 => Code::F4, 0x74 => Code::F5, 0x75 => Code::F6,
        0x76 => Code::F7, 0x77 => Code::F8, 0x78 => Code::F9,
        0x79 => Code::F10, 0x7A => Code::F11, 0x7B => Code::F12,
        _ => return None,
    };
    let mut modifiers = Modifiers::empty();
    if mods & 0x0001 != 0 { modifiers |= Modifiers::ALT; }
    if mods & 0x0002 != 0 { modifiers |= Modifiers::CONTROL; }
    if mods & 0x0004 != 0 { modifiers |= Modifiers::SHIFT; }
    if mods & 0x0008 != 0 { modifiers |= Modifiers::SUPER; }
    Some(HotKey::new(Some(modifiers), key))
}

//! The hotkeys helper is only used by Linux.
pub fn run_helper() -> i32 {
    eprintln!("error the hotkeys helper is only used on Linux");
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_mac_defaults_to_command_option() {
        let (mods, key) = crate::platform::hotkeys::parse_hotkey_label("Cmd+Option+R").unwrap();
        let hotkey = make_hotkey(mods, key).unwrap();
        assert!(hotkey.mods.contains(Modifiers::SUPER));
        assert!(hotkey.mods.contains(Modifiers::ALT));
        assert_eq!(hotkey.key, Code::KeyR);
    }
}
