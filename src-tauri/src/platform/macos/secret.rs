//! Account credentials in the macOS login Keychain.
//!
//! The small file in the app data directory contains only a reference. The
//! bearer token itself is stored by Security.framework in the user's Keychain.

use std::sync::atomic::{AtomicU64, Ordering};

const SERVICE: &str = "com.a2tools.dps-meter";
const REFERENCE: &[u8] = b"macos-keychain:";

fn new_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    format!("{nanos:x}-{:x}-{:x}", std::process::id(), COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn parse_reference(sealed: &[u8]) -> Option<&str> {
    let id = std::str::from_utf8(sealed.strip_prefix(REFERENCE)?).ok()?;
    (!id.is_empty()).then_some(id)
}

/// The Security.framework Keychain API is available on every supported macOS
/// installation. Actual access is checked by `protect`/`unprotect`, where any
/// user approval or Keychain error can be reported accurately.
pub fn available() -> bool {
    true
}

pub fn protect(plaintext: &[u8], _entropy: &[u8]) -> Result<Vec<u8>, String> {
    use security_framework::passwords::set_generic_password;

    let id = new_id();
    set_generic_password(SERVICE, &id, plaintext)
        .map_err(|e| format!("macOS Keychain could not store the credential ({}): {e}", e.code()))?;
    Ok([REFERENCE, id.as_bytes()].concat())
}

pub fn unprotect(sealed: &[u8], _entropy: &[u8]) -> Result<Vec<u8>, crate::platform::UnsealError> {
    use security_framework::passwords::get_generic_password;

    let id = parse_reference(sealed).ok_or(crate::platform::UnsealError::Invalid)?;
    get_generic_password(SERVICE, id).map_err(|e| {
        tracing::warn!("Could not read the credential from the macOS Keychain ({}): {e}", e.code());
        crate::platform::UnsealError::Unavailable
    })
}

pub fn forget(sealed: &[u8]) {
    use security_framework::passwords::delete_generic_password;

    let Some(id) = parse_reference(sealed) else { return };
    if let Err(e) = delete_generic_password(SERVICE, id) {
        // errSecItemNotFound is harmless: the item may already have been
        // removed by the user from Keychain Access.
        tracing::debug!("Could not remove macOS Keychain item {id} ({}): {e}", e.code());
    }
}
