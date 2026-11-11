//! macOS system libpcap backed by the kernel BPF devices.

/// Prefer Apple's system library; the unqualified name allows a custom
/// installation to be found through the normal dynamic-library search path.
pub const LIBRARIES: &[&str] = &["/usr/lib/libpcap.A.dylib", "libpcap.dylib"];

pub const MISSING_HELP: &str = "macOS packet capture uses /dev/bpf*. Grant the app permission to open a BPF device (or adjust its device permissions); no Npcap installation is needed."; 

/// The capture library is supplied by macOS, not bundled by the meter.
pub const OFFERS_INSTALL: bool = false;

/// BPF reads return at the configured libpcap timeout while idle.
pub const READ_TIMES_OUT_WHEN_IDLE: bool = true;

/// Treat filter changes conservatively: BPF may discard packets queued under
/// the previous filter, so the capturer sets filters at its normal safe point.
pub const FILTER_CHANGE_KEEPS_QUEUE: bool = false;

/// Nothing to set up before loading libpcap.
pub fn prepare() {}

pub fn run_installer(_installer: &std::path::Path) -> Result<(), String> {
    Err("the meter does not install libpcap; macOS supplies the system library".into())
}

/// Load Apple's system libpcap or a custom installation.
///
/// # Safety
/// Loading a library runs its initialisation, as `libloading::Library::new`.
pub unsafe fn load(name: &str) -> Result<libloading::Library, libloading::Error> {
    unsafe { libloading::Library::new(name) }
}

pub fn library_available() -> bool {
    // SAFETY: loading libpcap runs no initialisation we depend on not running.
    LIBRARIES.iter().any(|name| unsafe { load(name).is_ok() })
}

/// Skip libpcap's pseudo-devices. "any" sees every packet a second time on top
/// of the real interface it arrived on; the rest carry no IP traffic.
///
/// Container networking too: Docker and Podman give every container a `veth`
/// pair and every network a `docker0` / `br-<id>` / `podman` bridge. The game,
/// under Proton, talks through the real interface, so these only add capture
/// threads (one player had 22 of them).
pub fn skip_device(name: &str) -> bool {
    name == "any" || name.starts_with("pktap") || name.starts_with("utun")
        || name.starts_with("awdl") || name.starts_with("llw")
}
