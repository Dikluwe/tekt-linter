//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/sarif-formatter.md
//! @prompt-hash c3c77a4b
//! @layer L2
//! @updated 2026-08-24

//! Lossless presentation of native paths at output boundaries.

use std::path::Path;

#[cfg(unix)]
fn native_bytes(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    path.as_os_str().as_bytes().to_vec()
}

#[cfg(unix)]
pub fn human_path(path: &Path) -> String {
    let bytes = native_bytes(path);
    let mut output = String::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        match std::str::from_utf8(&bytes[cursor..]) {
            Ok(valid) => {
                output.push_str(valid);
                break;
            }
            Err(error) => {
                let valid = error.valid_up_to();
                output.push_str(
                    std::str::from_utf8(&bytes[cursor..cursor + valid])
                        .expect("valid_up_to always identifies valid UTF-8"),
                );
                cursor += valid;
                let invalid = error.error_len().unwrap_or(bytes.len() - cursor);
                for byte in &bytes[cursor..cursor + invalid] {
                    output.push_str(&format!("\\x{byte:02X}"));
                }
                cursor += invalid;
            }
        }
    }
    output
}

#[cfg(unix)]
pub fn machine_path_uri(path: &Path) -> String {
    let mut output = String::new();
    for byte in native_bytes(path) {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.' | b'~' | b':') {
            output.push(byte as char);
        } else {
            output.push_str(&format!("%{byte:02X}"));
        }
    }
    output
}

#[cfg(windows)]
pub fn human_path(path: &Path) -> String {
    use std::os::windows::ffi::OsStrExt;
    std::char::decode_utf16(path.as_os_str().encode_wide()).fold(
        String::new(),
        |mut output, value| {
            match value {
                Ok(character) => output.push(character),
                Err(error) => output.push_str(&format!("\\u{:04X}", error.unpaired_surrogate())),
            }
            output
        },
    )
}

#[cfg(windows)]
pub fn machine_path_uri(path: &Path) -> String {
    use std::os::windows::ffi::OsStrExt;
    let mut output = String::new();
    for value in std::char::decode_utf16(path.as_os_str().encode_wide()) {
        match value {
            Ok(character) => {
                let mut encoded = [0; 4];
                for byte in character.encode_utf8(&mut encoded).bytes() {
                    if byte == b'\\' {
                        output.push('/');
                        continue;
                    }
                    if byte.is_ascii_alphanumeric()
                        || matches!(byte, b'/' | b'-' | b'_' | b'.' | b'~' | b':')
                    {
                        output.push(byte as char);
                    } else {
                        output.push_str(&format!("%{byte:02X}"));
                    }
                }
            }
            Err(error) => output.push_str(&format!("%u{:04X}", error.unpaired_surrogate())),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_paths_remain_readable_and_percent_is_not_ambiguous() {
        assert_eq!(human_path(Path::new("src/ação.rs")), "src/ação.rs");
        assert_eq!(machine_path_uri(Path::new("src/100%.rs")), "src/100%25.rs");
    }

    #[cfg(unix)]
    #[test]
    fn invalid_unix_bytes_are_lossless_in_human_and_machine_output() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let path = Path::new(OsStr::from_bytes(b"src/\xff.rs"));
        assert_eq!(human_path(path), "src/\\xFF.rs");
        assert_eq!(machine_path_uri(path), "src/%FF.rs");
    }

    #[cfg(windows)]
    #[test]
    fn unpaired_windows_surrogate_has_lossless_native_escape() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        let native = OsString::from_wide(&[b'x' as u16, 0xD800, b'y' as u16]);
        let path = Path::new(&native);
        assert_eq!(human_path(path), "x\\uD800y");
        assert_eq!(machine_path_uri(path), "x%uD800y");
    }
}
