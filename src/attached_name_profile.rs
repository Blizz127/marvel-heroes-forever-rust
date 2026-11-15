//! Bounded membership facts for the native attached-MovieClip name profile.
//!
//! See `docs/ATTACHED_NAME_CLASSIFICATION.md`. Native inventories are private
//! caller inputs, never source tables. This module neither hashes artifacts nor
//! decides whether a live object is missing a property. Its digest callback must
//! hash the exact supplied bytes using the caller's trusted SHA-256 implementation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const SUPPORTED_BUILD: &str = "1.52.0.1700";
pub const MAX_PROFILE_BYTES: usize = 64 * 1024;
pub const MAX_NAME_CODE_UNITS: usize = 1024;
const STANDARD_COUNT: usize = 117;
const MOVIECLIP_COUNT: usize = 44;
const OBJECT_COUNT: usize = 9;
const HEADER_LINES: usize = 4;

/// Trusted expectations, obtained independently of the input profile.
/// The caller must also verify the actual source executable's identity.
pub struct ExpectedProfile<'a> {
    pub build: &'a str,
    pub source_sha256: [u8; 32],
    pub inventory_sha256: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedName {
    Empty,
    TooLong,
    NonAsciiOrNul,
    UnderscoreRoute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachedNameClass {
    Ordinary,
    /// Membership only: no native getter or setter is implemented by this fact.
    Standard(u8),
    UnsupportedName(UnsupportedName),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReservedReadName {
    Constructor,
    HiddenConstructor,
    Prototype,
    Resolver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaselineOwners(u8);

impl BaselineOwners {
    pub const MOVIECLIP: Self = Self(1);
    pub const OBJECT: Self = Self(2);

    pub fn contains(self, owner: Self) -> bool {
        self.0 & owner.0 == owner.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaselineMembership {
    Reserved(ReservedReadName),
    Stored(BaselineOwners),
    /// Absent from the verified original builtin inventories only.
    /// This is never a finding that the current receiver lacks the property.
    Absent,
    UnsupportedName(UnsupportedName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    TooLarge,
    UnsupportedBuild,
    DigestFailed(String),
    DigestMismatch,
    Malformed { line: usize },
    SourceMismatch,
    CountMismatch,
    Duplicate { line: usize },
    InvalidOrder { line: usize },
    BaselineShapeMismatch,
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => write!(f, "native name profile exceeds its byte limit"),
            Self::UnsupportedBuild => write!(f, "unsupported native name profile build"),
            Self::DigestFailed(error) => write!(f, "native inventory digest failed: {error}"),
            Self::DigestMismatch => write!(f, "native inventory digest mismatch"),
            Self::Malformed { line } => write!(f, "malformed native name profile at line {line}"),
            Self::SourceMismatch => write!(f, "native inventory source identity mismatch"),
            Self::CountMismatch => write!(f, "incomplete or unexpected native inventory counts"),
            Self::Duplicate { line } => {
                write!(f, "duplicate native inventory entry at line {line}")
            }
            Self::InvalidOrder { line } => {
                write!(f, "noncanonical native inventory order at line {line}")
            }
            Self::BaselineShapeMismatch => write!(f, "unexpected native builtin inventory overlap"),
        }
    }
}

impl std::error::Error for ProfileError {}

/// Immutable, verified original-inventory membership. Use behind an Arc if shared.
pub struct NativeNameProfile {
    source_sha256: [u8; 32],
    inventory_sha256: [u8; 32],
    standard: BTreeMap<String, u8>,
    baseline: BTreeMap<String, BaselineOwners>,
}

// Avoid accidentally dumping extracted native inventories through ordinary Debug.
impl fmt::Debug for NativeNameProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeNameProfile")
            .field("build", &SUPPORTED_BUILD)
            .field("standard_names", &self.standard.len())
            .field("baseline_names", &self.baseline.len())
            .finish_non_exhaustive()
    }
}

impl NativeNameProfile {
    /// Authenticate the exact buffer, then validate the complete private profile.
    ///
    /// The digest implementation and expected identities are caller-owned trust
    /// inputs. Header claims are not a substitute for measuring the source file.
    /// Allocation is bounded before calling the digest callback; no filesystem,
    /// executable, host object, or VM state is touched here.
    pub fn parse_verified(
        bytes: &[u8],
        expected: &ExpectedProfile<'_>,
        sha256: impl FnOnce(&[u8]) -> Result<[u8; 32], String>,
    ) -> Result<Self, ProfileError> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err(ProfileError::TooLarge);
        }
        if expected.build != SUPPORTED_BUILD {
            return Err(ProfileError::UnsupportedBuild);
        }
        let actual = sha256(bytes).map_err(ProfileError::DigestFailed)?;
        if actual != expected.inventory_sha256 {
            return Err(ProfileError::DigestMismatch);
        }
        let text = std::str::from_utf8(bytes).map_err(|_| malformed(1))?;
        if !text.is_ascii() || text.contains('\r') || text.contains('\0') {
            return Err(malformed(1));
        }
        let body = text.strip_suffix('\n').ok_or_else(|| malformed(1))?;
        let lines: Vec<_> = body.split('\n').collect();
        if lines.len() != HEADER_LINES + STANDARD_COUNT + MOVIECLIP_COUNT + OBJECT_COUNT {
            return Err(ProfileError::CountMismatch);
        }
        if lines[0] != "MHF-ATTACHED-NAMES\t1" {
            return Err(malformed(1));
        }
        if lines[1] != format!("build\t{}", expected.build) {
            return Err(ProfileError::SourceMismatch);
        }
        let source = lines[2]
            .strip_prefix("source-sha256\t")
            .and_then(decode_sha256)
            .ok_or_else(|| malformed(3))?;
        if source != expected.source_sha256 {
            return Err(ProfileError::SourceMismatch);
        }
        if lines[3] != "counts\t117\t44\t9" {
            return Err(ProfileError::CountMismatch);
        }

        let mut standard = BTreeMap::new();
        for (id, row) in lines[HEADER_LINES..HEADER_LINES + STANDARD_COUNT]
            .iter()
            .enumerate()
        {
            let line = HEADER_LINES + id + 1;
            let fields: Vec<_> = row.split('\t').collect();
            if fields.len() != 4 || fields[0] != "standard" {
                return Err(malformed(line));
            }
            if fields[1] != id.to_string() {
                return Err(ProfileError::InvalidOrder { line });
            }
            let alias = id <= 21 || (110..=115).contains(&id);
            if fields[2] != if alias { "1" } else { "0" } {
                return Err(malformed(line));
            }
            let name = inventory_name(fields[3], line)?;
            if alias && (!name.starts_with('_') || name.bytes().any(|b| b.is_ascii_uppercase())) {
                return Err(malformed(line));
            }
            if standard.insert(name.to_owned(), id as u8).is_some() {
                return Err(ProfileError::Duplicate { line });
            }
        }

        let mut baseline: BTreeMap<String, BaselineOwners> = BTreeMap::new();
        let mut start = HEADER_LINES + STANDARD_COUNT;
        for (role, count, owner) in [
            ("movieclip", MOVIECLIP_COUNT, BaselineOwners::MOVIECLIP),
            ("object", OBJECT_COUNT, BaselineOwners::OBJECT),
        ] {
            let mut previous: Option<&str> = None;
            for (offset, row) in lines[start..start + count].iter().enumerate() {
                let line = start + offset + 1;
                let (actual_role, name) = row.split_once('\t').ok_or_else(|| malformed(line))?;
                if actual_role != role {
                    return Err(malformed(line));
                }
                let name = inventory_name(name, line)?;
                if previous == Some(name) {
                    return Err(ProfileError::Duplicate { line });
                }
                if previous.is_some_and(|old| old > name) {
                    return Err(ProfileError::InvalidOrder { line });
                }
                previous = Some(name);
                baseline
                    .entry(name.to_owned())
                    .and_modify(|bits| bits.0 |= owner.0)
                    .or_insert(owner);
            }
            start += count;
        }
        let overlaps: Vec<_> = baseline.iter().filter(|(_, roles)| roles.0 == 3).collect();
        let mut exclusions: BTreeSet<&str> = baseline.keys().map(String::as_str).collect();
        exclusions.extend(["constructor", "__constructor__", "__proto__", "__resolve"]);
        if baseline.len() != 52
            || overlaps.len() != 1
            || overlaps[0].0 != "constructor"
            || exclusions.len() != 54
        {
            return Err(ProfileError::BaselineShapeMismatch);
        }

        Ok(Self {
            source_sha256: source,
            inventory_sha256: actual,
            standard,
            baseline,
        })
    }

    pub fn source_sha256(&self) -> &[u8; 32] {
        &self.source_sha256
    }

    pub fn inventory_sha256(&self) -> &[u8; 32] {
        &self.inventory_sha256
    }

    /// Exact map membership first; all other underscore routes remain gated.
    /// Ordinary admits only a name route on the proved concrete attached class.
    pub fn classify_attached_name(&self, name: &[u16]) -> AttachedNameClass {
        let name = match ascii_name(name) {
            Ok(name) => name,
            Err(reason) => return AttachedNameClass::UnsupportedName(reason),
        };
        if let Some(&id) = self.standard.get(&name) {
            AttachedNameClass::Standard(id)
        } else if name.starts_with('_') {
            AttachedNameClass::UnsupportedName(UnsupportedName::UnderscoreRoute)
        } else {
            AttachedNameClass::Ordinary
        }
    }

    /// Original builtin inventory membership, not lookup on a live object.
    pub fn native_baseline_membership(&self, name: &[u16]) -> BaselineMembership {
        let name = match ascii_name(name) {
            Ok(name) => name,
            Err(reason) => return BaselineMembership::UnsupportedName(reason),
        };
        let reserved = match name.as_str() {
            "constructor" => Some(ReservedReadName::Constructor),
            "__constructor__" => Some(ReservedReadName::HiddenConstructor),
            "__proto__" => Some(ReservedReadName::Prototype),
            "__resolve" => Some(ReservedReadName::Resolver),
            _ => None,
        };
        if let Some(reserved) = reserved {
            BaselineMembership::Reserved(reserved)
        } else if let Some(&owners) = self.baseline.get(&name) {
            BaselineMembership::Stored(owners)
        } else {
            BaselineMembership::Absent
        }
    }
}

fn malformed(line: usize) -> ProfileError {
    ProfileError::Malformed { line }
}

fn inventory_name(name: &str, line: usize) -> Result<&str, ProfileError> {
    if name.is_empty()
        || name.len() > MAX_NAME_CODE_UNITS
        || !name.bytes().all(|b| (0x21..=0x7e).contains(&b))
    {
        return Err(malformed(line));
    }
    Ok(name)
}

fn ascii_name(name: &[u16]) -> Result<String, UnsupportedName> {
    if name.is_empty() {
        return Err(UnsupportedName::Empty);
    }
    if name.len() > MAX_NAME_CODE_UNITS {
        return Err(UnsupportedName::TooLong);
    }
    if name.iter().any(|&c| c == 0 || c > 0x7f) {
        return Err(UnsupportedName::NonAsciiOrNul);
    }
    Ok(name.iter().map(|&c| char::from(c as u8)).collect())
}

fn decode_sha256(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64
        || !text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return None;
    }
    let mut out = [0; 32];
    for (slot, pair) in out.iter_mut().zip(text.as_bytes().chunks_exact(2)) {
        let nibble = |c: u8| if c <= b'9' { c - b'0' } else { c - b'a' + 10 };
        *slot = (nibble(pair[0]) << 4) | nibble(pair[1]);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DIGEST: [u8; 32] = [0x51; 32];

    // Synthetic names only. A caller-owned mock digest tests parser boundaries;
    // the separate owner-private smoke uses SHA-256 over the actual input bytes.
    fn fixture() -> Vec<u8> {
        let mut text = format!(
            "MHF-ATTACHED-NAMES\t1\nbuild\t{SUPPORTED_BUILD}\nsource-sha256\t{}\ncounts\t117\t44\t9\n",
            "ab".repeat(32)
        );
        for id in 0..STANDARD_COUNT {
            let alias = id <= 21 || (110..=115).contains(&id);
            text.push_str(&format!(
                "standard\t{id}\t{}\t{}native{id:03}\n",
                u8::from(alias),
                if alias { "_" } else { "" }
            ));
        }
        let mut movieclip = vec!["constructor".to_owned(), "__proto__".to_owned()];
        movieclip.extend((0..42).map(|i| format!("mc{i:03}")));
        movieclip.sort();
        let mut object = vec!["constructor".to_owned()];
        object.extend((0..8).map(|i| format!("obj{i:03}")));
        object.sort();
        for (role, names) in [("movieclip", movieclip), ("object", object)] {
            for name in names {
                text.push_str(&format!("{role}\t{name}\n"));
            }
        }
        text.into_bytes()
    }

    fn expected() -> ExpectedProfile<'static> {
        ExpectedProfile {
            build: SUPPORTED_BUILD,
            source_sha256: [0xab; 32],
            inventory_sha256: TEST_DIGEST,
        }
    }

    fn parse(bytes: &[u8]) -> Result<NativeNameProfile, ProfileError> {
        NativeNameProfile::parse_verified(bytes, &expected(), |_| Ok(TEST_DIGEST))
    }

    fn units(name: &str) -> Vec<u16> {
        name.encode_utf16().collect()
    }

    #[test]
    fn separates_route_membership_from_native_baseline_presence() {
        let profile = parse(&fixture()).unwrap();
        assert_eq!(
            profile.classify_attached_name(&units("native022")),
            AttachedNameClass::Standard(22)
        );
        assert_eq!(
            profile.classify_attached_name(&units("NATIVE022")),
            AttachedNameClass::Ordinary
        );
        assert_eq!(
            profile.classify_attached_name(&units("mc000")),
            AttachedNameClass::Ordinary
        );
        assert_eq!(
            profile.native_baseline_membership(&units("mc000")),
            BaselineMembership::Stored(BaselineOwners::MOVIECLIP)
        );
        assert_eq!(
            profile.native_baseline_membership(&units("notInstalled")),
            BaselineMembership::Absent
        );
        assert_eq!(
            profile.native_baseline_membership(&units("__resolve")),
            BaselineMembership::Reserved(ReservedReadName::Resolver)
        );
        assert_eq!(
            profile.classify_attached_name(&units("_NATIVE000")),
            AttachedNameClass::UnsupportedName(UnsupportedName::UnderscoreRoute)
        );
    }

    #[test]
    fn digest_is_bound_to_the_exact_buffer_and_failure_is_closed() {
        let bytes = fixture();
        let profile = NativeNameProfile::parse_verified(&bytes, &expected(), |actual| {
            assert_eq!(actual.as_ptr(), bytes.as_ptr());
            assert_eq!(actual, bytes);
            Ok(TEST_DIGEST)
        })
        .unwrap();
        assert_eq!(profile.inventory_sha256(), &TEST_DIGEST);
        assert_eq!(
            NativeNameProfile::parse_verified(&bytes, &expected(), |_| Ok([0; 32])).unwrap_err(),
            ProfileError::DigestMismatch
        );
        assert!(matches!(
            NativeNameProfile::parse_verified(&bytes, &expected(), |_| Err("unavailable".into())),
            Err(ProfileError::DigestFailed(_))
        ));
    }

    #[test]
    fn rejects_truncation_extras_and_source_substitution() {
        let bytes = fixture();
        assert_eq!(parse(&bytes[..bytes.len() - 1]).unwrap_err(), malformed(1));
        let mut extra = bytes.clone();
        extra.push(b'\n');
        assert_eq!(parse(&extra).unwrap_err(), ProfileError::CountMismatch);
        let text = String::from_utf8(bytes)
            .unwrap()
            .replace(&"ab".repeat(32), &"cd".repeat(32));
        assert_eq!(
            parse(text.as_bytes()).unwrap_err(),
            ProfileError::SourceMismatch
        );
    }

    #[test]
    fn rejects_duplicate_names_and_incomplete_ids() {
        let text = String::from_utf8(fixture()).unwrap();
        assert!(matches!(
            parse(text.replace("native023", "native022").as_bytes()),
            Err(ProfileError::Duplicate { .. })
        ));
        assert!(matches!(
            parse(text.replace("standard\t23\t", "standard\t22\t").as_bytes()),
            Err(ProfileError::InvalidOrder { .. })
        ));
        assert!(matches!(
            parse(
                text.replace("movieclip\tmc001", "movieclip\tmc000")
                    .as_bytes()
            ),
            Err(ProfileError::Duplicate { .. })
        ));
    }

    #[test]
    fn rejects_schema_order_and_prototype_shape_changes() {
        let text = String::from_utf8(fixture()).unwrap();
        assert!(
            parse(
                text.replace("MHF-ATTACHED-NAMES\t1", "MHF-ATTACHED-NAMES\t2")
                    .as_bytes()
            )
            .is_err()
        );
        assert!(parse(text.replace("standard\t0\t1", "standard\t0\t0").as_bytes()).is_err());
        assert!(
            parse(
                text.replace("movieclip\t__proto__", "movieclip\t__unknown")
                    .as_bytes()
            )
            .is_err()
        );
        assert!(
            parse(
                text.replace("object\tconstructor", "object\tanotherConstructor")
                    .as_bytes()
            )
            .is_err()
        );
    }

    #[test]
    fn bounds_names_and_input_before_digest_work() {
        let bytes = vec![b'x'; MAX_PROFILE_BYTES + 1];
        assert_eq!(
            NativeNameProfile::parse_verified(&bytes, &expected(), |_| panic!(
                "must not hash oversized input"
            ))
            .unwrap_err(),
            ProfileError::TooLarge
        );
        let profile = parse(&fixture()).unwrap();
        for name in [
            vec![],
            vec![0],
            vec![0x100],
            vec![b'x' as u16; MAX_NAME_CODE_UNITS + 1],
        ] {
            assert!(matches!(
                profile.classify_attached_name(&name),
                AttachedNameClass::UnsupportedName(_)
            ));
            assert!(matches!(
                profile.native_baseline_membership(&name),
                BaselineMembership::UnsupportedName(_)
            ));
        }
        assert!(
            parse(
                &fixture()
                    .into_iter()
                    .map(|b| if b == b'\t' { b'\r' } else { b })
                    .collect::<Vec<_>>()
            )
            .is_err()
        );
    }
}
