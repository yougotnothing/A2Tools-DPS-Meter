//! macOS implementation. Packet capture uses the system libpcap/BPF stack.
pub mod admin;
#[path = "../unsupported/atomic_file.rs"] pub mod atomic_file;
#[path = "../unsupported/clock.rs"] pub mod clock;
#[path = "../linux/discord_ipc.rs"] pub mod discord_ipc;
#[path = "../unsupported/dialog.rs"] pub mod dialog;
#[path = "../unsupported/fps.rs"] pub mod fps;
#[path = "../unsupported/hotkeys.rs"] pub mod hotkeys;
pub mod pcap;
#[path = "../unsupported/process.rs"] pub mod process;
#[path = "../unsupported/screen.rs"] pub mod screen;
#[path = "../unsupported/secret.rs"] pub mod secret;
#[path = "../unsupported/shell.rs"] pub mod shell;
#[path = "../unsupported/updater.rs"] pub mod updater;
#[path = "../unsupported/window.rs"] pub mod window;
pub mod window_detector;
#[path = "../unsupported/window_rules.rs"] pub mod window_rules;
#[path = "../unsupported/window_startup.rs"] pub mod window_startup;
