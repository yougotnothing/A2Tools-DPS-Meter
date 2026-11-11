/// Report whether this user can actually open a BPF device. This is the
/// capability the capture pipeline needs; it does not require uid 0 when the
/// device permissions grant access directly.
pub fn is_admin() -> bool {
    let Ok(entries) = std::fs::read_dir("/dev") else { return false };
    entries.flatten().any(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("bpf") || !name[3..].bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
        std::fs::OpenOptions::new().read(true).write(true).open(entry.path()).is_ok()
    })
}
pub fn relaunch_elevated(_args: &[String]) -> Result<bool, String> {
    Err("macOS does not support relaunching this app with administrator privileges".into())
}
pub fn wait_for_exit(_pid: u32, _timeout: std::time::Duration) {}
pub fn wants_admin(_identifier: &str) -> bool { false }
