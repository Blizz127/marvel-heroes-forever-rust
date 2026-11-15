//! Explicit resource backend for the saved localization callback profile.
//! Movie/Environment selection and ExternalInterface installation are separate.

use super::localized_label::{Configuration, ConfiguredLabels, ResolvedLabel};
use crate::{
    avm1::vm::{HostResult, Value},
    localization::LocaleCatalog,
};
use std::io;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_requests: u64,
    pub max_label_units: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_requests: 100_000,
            max_label_units: 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    /// Admitted attempts, including requests later rejected by this backend.
    pub requests: u64,
    /// Resource resolution only; subsequent VM allocation/setters may still fail.
    pub resolved: u64,
}

/// One explicit configured resource source and locale catalog. This type does
/// not select an Unreal receiver or assert that a movie installed its handler.
pub struct LocalizationCallback {
    labels: ConfiguredLabels,
    locale: LocaleCatalog,
    limits: Limits,
    stats: Statistics,
    last_resolution: Option<ResolvedLabel>,
}

impl LocalizationCallback {
    pub fn new(
        labels: ConfiguredLabels,
        locale: LocaleCatalog,
        limits: Limits,
    ) -> io::Result<Self> {
        if limits.max_requests == 0
            || limits.max_label_units == 0
            || limits.max_label_units > 65_536
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid localization callback budgets",
            ));
        }
        Ok(Self {
            labels,
            locale,
            limits,
            stats: Statistics::default(),
            last_resolution: None,
        })
    }

    pub fn statistics(&self) -> Statistics {
        self.stats
    }
    pub fn configuration(&self) -> &Configuration {
        self.labels.configuration()
    }
    /// Last successful resource resolution, not last completed VM invocation.
    /// Rejected requests preserve it; results do not accumulate in this backend.
    pub fn last_resolution(&self) -> Option<&ResolvedLabel> {
        self.last_resolution.as_ref()
    }

    /// Accept only the saved wrapper's exact callback name and one ASCII string
    /// label. The caller must first establish the current movie/handler/receiver
    /// context. No general AS coercion or native failure fallback is implied.
    pub fn resolve_call(&mut self, args: &[Value]) -> io::Result<HostResult> {
        if self.stats.requests >= self.limits.max_requests {
            return Err(io::Error::other(
                "localization callback request budget exceeded",
            ));
        }
        self.stats.requests += 1;
        let label = Self::validated_label(args, self.limits.max_label_units)?;
        let resolved = self.labels.resolve(&label)?;
        let data = resolved.prototype().localized_data(&self.locale)?;
        self.last_resolution = Some(resolved);
        self.stats.resolved += 1;
        Ok(data.into_host_result())
    }

    pub(crate) fn validated_label(args: &[Value], max_label_units: usize) -> io::Result<String> {
        let [Value::String(name), Value::String(label)] = args else {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "localization callback requires its name and one string label",
            ));
        };
        if !name
            .iter()
            .copied()
            .eq("GetLocalizedObjectDataByLabel".encode_utf16())
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "external callback is not implemented by the localization backend",
            ));
        }
        // Check before converting/allocating. Native incoming code page zero is
        // only established for the restricted ASCII profile, not general UTF-8.
        if label.len() > max_label_units {
            return Err(io::Error::other(
                "localization callback label budget exceeded",
            ));
        }
        if label.is_empty() || label.iter().any(|&unit| unit == 0 || unit > 127) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "localization callback label must be nonempty NUL-free ASCII",
            ));
        }
        String::from_utf16(label).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}
