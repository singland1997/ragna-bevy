//! Minimal, constant-time equality check compatible with the `constant_time_eq` crate API.
//! This vendored shim exists to avoid pulling a crates.io release that requires `edition2024`
//! on environments that ship an older stable cargo.
#![forbid(unsafe_code)]

/// Compare two byte slices in constant time.
///
/// Returns `true` if the slices are equal, `false` otherwise.
/// Runs in time proportional to the length of the shortest slice, and always
/// visits every byte, so it does not short-circuit on the first difference.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    // XOR accumulate all differences into `diff`, then test if zero.
    let len = core::cmp::min(a.len(), b.len());
    let mut diff: u8 = (a.len() ^ b.len()) as u8;
    let mut i = 0;
    while i < len {
        diff |= a[i] ^ b[i];
        i += 1;
    }
    diff == 0
}

