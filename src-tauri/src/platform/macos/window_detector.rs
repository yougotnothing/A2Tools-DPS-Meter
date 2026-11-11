//! Wine/CrossOver runs the Windows game as a macOS process. Window titles
//! belong to Wine and are not reliably available through public APIs, so
//! identify the game by its executable/process command line.
use std::process::Command;
fn candidates() -> Vec<String> {
    let out = Command::new("/bin/ps").args(["-axo", "pid=,comm=,args="]).output();
    let Ok(out) = out else { return Vec::new() };
    String::from_utf8_lossy(&out.stdout).lines()
        .filter(|line| {
            let l = line.to_ascii_lowercase();
            l.contains("aion2-win64-shipping") || l.contains("aion2.exe")
        })
        .map(str::to_string).collect()
}
pub fn find_aion2_window_title() -> Option<String> {
    (!candidates().is_empty()).then(|| "AION2".to_string())
}
pub fn find_aion2_window() -> bool { find_aion2_window_title().is_some() }
pub fn is_aion2_foreground() -> bool { true }
pub fn watch_screenshot_tool(_on_change: impl Fn(bool) + Send + Sync + 'static) {}
pub fn watch_focus(_on_change: impl Fn() + Send + Sync + 'static) -> Result<(), String> { Ok(()) }
pub fn describe_candidates() -> Vec<String> { candidates().into_iter().take(8).collect() }
