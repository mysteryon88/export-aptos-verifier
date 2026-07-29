pub mod arkworks;

use crate::error::{Error, Result};
use crate::model::MAX_PUBLIC_INPUTS;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

pub(crate) const MAX_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
pub(crate) const MAX_ARKWORKS_PROOF_BYTES: usize = 1024;

pub(crate) fn read_bounded_text(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path).map_err(|source| Error::Io {
        source,
        context: format!("failed to inspect file {}", path.display()),
    })?;
    if metadata.len() > MAX_ARTIFACT_BYTES {
        return Err(Error::InputTooLarge {
            path: path.to_path_buf(),
            size: metadata.len(),
            max: MAX_ARTIFACT_BYTES,
        });
    }

    let file = File::open(path).map_err(|source| Error::Io {
        source,
        context: format!("failed to open file {}", path.display()),
    })?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_ARTIFACT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| Error::Io {
            source,
            context: format!("failed to read file {}", path.display()),
        })?;
    if bytes.len() as u64 > MAX_ARTIFACT_BYTES {
        return Err(Error::InputTooLarge {
            path: path.to_path_buf(),
            size: bytes.len() as u64,
            max: MAX_ARTIFACT_BYTES,
        });
    }

    String::from_utf8(bytes).map_err(|err| {
        Error::Serialization(format!("{} is not valid UTF-8: {err}", path.display()))
    })
}

pub(crate) fn ensure_public_input_count(len: usize, field: &str) -> Result<()> {
    if len > MAX_PUBLIC_INPUTS {
        return Err(Error::PublicInputCountMismatch(format!(
            "{field} has {len} values; maximum is {MAX_PUBLIC_INPUTS}"
        )));
    }
    Ok(())
}

pub(crate) fn decode_hex(raw: &str, field: &str, max_bytes: usize) -> Result<Vec<u8>> {
    let hex = raw.trim().trim_start_matches("0x").trim_start_matches("0X");
    if hex.is_empty() {
        return Err(Error::HexParse(format!("{field} must not be empty")));
    }
    if hex.len() > max_bytes.saturating_mul(2) {
        return Err(Error::HexParse(format!(
            "{field} exceeds {max_bytes}-byte limit"
        )));
    }
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::HexParse(format!("{field} must be a hex string")));
    }
    if !hex.len().is_multiple_of(2) {
        return Err(Error::HexParse(format!(
            "{field} has odd hex length {}",
            hex.len()
        )));
    }
    hex::decode(hex).map_err(|e| Error::HexParse(format!("{field}: {e}")))
}

pub(crate) fn validate_arkworks_vk_encoding(
    bytes: &[u8],
    fixed_prefix_bytes: usize,
    compressed_g1_bytes: usize,
    field: &str,
) -> Result<()> {
    let count_end = fixed_prefix_bytes
        .checked_add(8)
        .ok_or_else(|| Error::Serialization(format!("{field} length offset overflow")))?;
    let count_bytes: [u8; 8] = bytes
        .get(fixed_prefix_bytes..count_end)
        .ok_or_else(|| Error::Serialization(format!("{field} is truncated before IC length")))?
        .try_into()
        .map_err(|_| Error::Serialization(format!("{field} has invalid IC length encoding")))?;
    let ic_len: usize = u64::from_le_bytes(count_bytes)
        .try_into()
        .map_err(|_| Error::IcLengthMismatch(format!("{field} IC length does not fit usize")))?;
    if ic_len == 0 || ic_len > MAX_PUBLIC_INPUTS + 1 {
        return Err(Error::IcLengthMismatch(format!(
            "{field} has {ic_len} IC points; maximum is {}",
            MAX_PUBLIC_INPUTS + 1
        )));
    }
    let required = ic_len
        .checked_mul(compressed_g1_bytes)
        .and_then(|points| count_end.checked_add(points))
        .ok_or_else(|| Error::IcLengthMismatch(format!("{field} IC byte length overflow")))?;
    if required > bytes.len() {
        return Err(Error::Serialization(format!(
            "{field} declares {ic_len} IC points but is truncated"
        )));
    }
    Ok(())
}
