//! Conditional constant-key index selection from the owned client's native code.
//!
//! This selects two keys and an alpha; it does not interpolate values or advance
//! an animation clock. The caller supplies every input, including the integer
//! codec flag and knowledge of the shared native index cache. No class default
//! or scene-loop policy is inferred. See `docs/ANIMATION_TIMING.md` for evidence.

use std::io;

/// Counts through this inclusive bound convert exactly to binary32 integers.
/// This is a supported arithmetic limit, not a recovered retail asset limit.
pub const MAX_COUNT: i32 = 1 << 24;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Inputs {
    pub key_count: i32,
    pub frame_count: i32,
    pub duration: f32,
    pub time: f32,
    /// Zero/nonzero controls the branch, but the cache compares all 32 bits.
    /// Its relationship to scene or animation looping flags is unresolved.
    pub codec_flag: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Selection {
    pub first: i32,
    pub next: i32,
    pub alpha: f32,
}

/// The seven observed native cache fields, without an added frame-count key.
/// Public fields permit an explicitly supplied snapshot; a hit is checked before
/// its result is returned. A malformed unused result is replaced on a miss.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CacheRecord {
    pub key_count: i32,
    pub normalized_time: f32,
    pub duration: f32,
    pub codec_flag: i32,
    pub selection: Selection,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CacheState {
    /// The native initial/current cache has not been established. Multi-key
    /// sampling cannot choose a hit or miss and returns Unsupported.
    Unknown,
    /// Caller assertion that the next multi-key call must miss. This models a
    /// conditional fresh computation, not the retail cache's initial state.
    KnownMiss,
    Known(CacheRecord),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheUse {
    /// The native single-key path does not inspect or change the cache.
    Bypassed,
    Hit,
    Miss,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub selection: Selection,
    pub normalized_time: f32,
    pub cache_use: CacheUse,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

fn point(index: i32) -> Selection {
    Selection {
        first: index,
        next: index,
        alpha: 0.0,
    }
}

fn check_selection(selection: Selection, count: i32) -> io::Result<()> {
    if selection.first < 0
        || selection.first >= count
        || selection.next < 0
        || selection.next >= count
        || !selection.alpha.is_finite()
        || !(0.0..1.0).contains(&selection.alpha)
    {
        return Err(unsupported(
            "cached selection is outside the supported bounds",
        ));
    }
    Ok(())
}

/// Native floor/conversion steps reduce to truncation on this nonnegative,
/// finite, representable domain. Subtraction precedes index clamping.
fn split_position(position: f32, last: i32) -> Selection {
    let floor = position as i32;
    let alpha = position - floor as f32;
    let first = floor.min(last);
    Selection {
        first,
        next: (first + 1).min(last),
        alpha,
    }
}

fn fresh(input: Inputs, ratio: f32) -> Selection {
    if ratio <= 0.0 {
        return point(0);
    }
    let last = input.key_count - 1;
    if input.codec_flag == 0 {
        if ratio >= 1.0 {
            return point(last);
        }
        let position = last as f32 * ratio;
        return split_position(position, last);
    }
    if ratio >= 1.0 {
        return point(0);
    }

    let frame_position = input.frame_count as f32 * ratio;
    let frame_floor = frame_position as i32;
    let alpha = frame_position - frame_floor as f32;
    let first = frame_floor.min(input.frame_count - 1);
    let next = first + 1;
    if next == input.frame_count {
        return Selection {
            first: last,
            next: 0,
            alpha,
        };
    }
    if input.frame_count == input.key_count {
        return Selection { first, next, alpha };
    }

    // Keep division then multiplication separate, as in DIVSS then MULSS.
    // frame_count==1 already took the final-to-first branch above.
    let rescaled = frame_position / (input.frame_count - 1) as f32;
    let position = rescaled * last as f32;
    split_position(position, last)
}

/// Select constant-profile channel keys with explicit native cache knowledge.
///
/// Supported inputs have finite time, finite positive duration, counts in
/// `1..=MAX_COUNT` and finite time/duration. Operations use separate binary32
/// division, multiplication and subtraction. Retail floating-point environment
/// and live caller policy have not been observed.
///
/// A cache hit deliberately ignores frame_count, channel and sequence identity.
/// Float keys compare numerically, so +0 and -0 hit. Nonfinite known cache keys
/// are Unsupported: native unordered comparison behavior is outside this API.
/// Errors leave the cache unchanged. Successful misses replace the cache after
/// computing and checking the completed seven-field record. The exclusive Rust
/// reference does not emulate native write ordering or concurrent observation.
pub fn sample_constant(input: Inputs, cache: &mut CacheState) -> io::Result<Sample> {
    if !(1..=MAX_COUNT).contains(&input.key_count) || !(1..=MAX_COUNT).contains(&input.frame_count)
    {
        return Err(invalid(
            "animation key/frame count is outside the supported bounds",
        ));
    }
    if !input.time.is_finite() || !input.duration.is_finite() || input.duration <= 0.0 {
        return Err(invalid(
            "animation time/duration must be finite with positive duration",
        ));
    }
    let ratio = input.time / input.duration;
    if !ratio.is_finite() {
        return Err(unsupported("normalized animation time is not finite"));
    }
    if input.key_count == 1 {
        return Ok(Sample {
            selection: point(0),
            normalized_time: ratio,
            cache_use: CacheUse::Bypassed,
        });
    }

    match *cache {
        CacheState::Unknown => {
            return Err(unsupported("native index cache state is unknown"));
        }
        CacheState::KnownMiss => {}
        CacheState::Known(record) => {
            if !record.duration.is_finite() || !record.normalized_time.is_finite() {
                return Err(unsupported("nonfinite native cache keys are not supported"));
            }
            if record.key_count == input.key_count
                && record.codec_flag == input.codec_flag
                && record.duration == input.duration
                && record.normalized_time == ratio
            {
                check_selection(record.selection, input.key_count)?;
                return Ok(Sample {
                    selection: record.selection,
                    normalized_time: ratio,
                    cache_use: CacheUse::Hit,
                });
            }
        }
    }

    let selection = fresh(input, ratio);
    check_selection(selection, input.key_count)?;
    *cache = CacheState::Known(CacheRecord {
        key_count: input.key_count,
        normalized_time: ratio,
        duration: input.duration,
        codec_flag: input.codec_flag,
        selection,
    });
    Ok(Sample {
        selection,
        normalized_time: ratio,
        cache_use: CacheUse::Miss,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(k: i32, n: i32, time: f32, flag: i32) -> Inputs {
        Inputs {
            key_count: k,
            frame_count: n,
            duration: 4.0,
            time,
            codec_flag: flag,
        }
    }

    fn miss(input: Inputs) -> Selection {
        let mut cache = CacheState::KnownMiss;
        let sample = sample_constant(input, &mut cache).unwrap();
        assert_eq!(sample.cache_use, CacheUse::Miss);
        sample.selection
    }

    #[test]
    fn endpoints_and_constant_channel_bypass() {
        for time in [-8.0, -0.0, 0.0] {
            assert_eq!(miss(input(120, 120, time, 0)), point(0));
            assert_eq!(miss(input(120, 120, time, 1)), point(0));
        }
        for time in [4.0, 8.0] {
            assert_eq!(miss(input(120, 120, time, 0)), point(119));
            assert_eq!(miss(input(120, 120, time, 1)), point(0));
        }
        let mut unknown = CacheState::Unknown;
        let one = sample_constant(input(1, 120, 2.0, 1), &mut unknown).unwrap();
        assert_eq!(one.selection, point(0));
        assert_eq!(one.cache_use, CacheUse::Bypassed);
        assert_eq!(unknown, CacheState::Unknown);
        assert_eq!(
            sample_constant(input(2, 120, 2.0, 1), &mut unknown)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }

    #[test]
    fn endpoint_interval_and_frame_remapping_are_distinct() {
        assert_eq!(
            miss(input(4, 4, 2.0, 0)),
            Selection {
                first: 1,
                next: 2,
                alpha: 0.5
            }
        );
        assert_eq!(
            miss(input(4, 4, 2.0, 1)),
            Selection {
                first: 2,
                next: 3,
                alpha: 0.0
            }
        );
        assert_eq!(
            miss(input(4, 4, 3.5, 1)),
            Selection {
                first: 3,
                next: 0,
                alpha: 0.5
            }
        );
        assert_eq!(
            miss(input(3, 5, 1.0, 1)),
            Selection {
                first: 0,
                next: 1,
                alpha: 0.625
            }
        );
        assert_eq!(
            miss(input(3, 5, 3.5, 1)),
            Selection {
                first: 2,
                next: 0,
                alpha: 0.375
            }
        );
        assert_eq!(
            miss(input(3, 1, 2.0, 1)),
            Selection {
                first: 2,
                next: 0,
                alpha: 0.5
            }
        );
    }

    #[test]
    fn shared_cache_deliberately_omits_frame_count() {
        let mut cache = CacheState::KnownMiss;
        let original = sample_constant(input(4, 4, 2.0, 1), &mut cache).unwrap();
        let aliased = sample_constant(input(4, 8, 2.0, 1), &mut cache).unwrap();
        assert_eq!(aliased.cache_use, CacheUse::Hit);
        assert_eq!(aliased.selection, original.selection);
        assert_ne!(aliased.selection, miss(input(4, 8, 2.0, 1)));
        // The nonzero branch is the same, but the whole integer is a cache key.
        let changed_flag = sample_constant(input(4, 8, 2.0, 2), &mut cache).unwrap();
        assert_eq!(changed_flag.cache_use, CacheUse::Miss);
        assert_eq!(changed_flag.selection, miss(input(4, 8, 2.0, 1)));
    }

    #[test]
    fn signed_zero_hits_and_duration_remains_a_key() {
        let mut cache = CacheState::KnownMiss;
        sample_constant(input(4, 4, -0.0, 0), &mut cache).unwrap();
        let before = cache;
        let sample = sample_constant(input(4, 4, 0.0, 0), &mut cache).unwrap();
        assert_eq!(sample.cache_use, CacheUse::Hit);
        match (before, cache) {
            (CacheState::Known(a), CacheState::Known(b)) => {
                assert_eq!(a.normalized_time.to_bits(), 0x8000_0000);
                assert_eq!(a.normalized_time.to_bits(), b.normalized_time.to_bits());
            }
            _ => panic!("expected retained known cache"),
        }
        let changed_duration = Inputs {
            duration: 8.0,
            ..input(4, 4, 0.0, 0)
        };
        assert_eq!(
            sample_constant(changed_duration, &mut cache)
                .unwrap()
                .cache_use,
            CacheUse::Miss
        );
    }

    #[test]
    fn malformed_inputs_and_hit_state_do_not_mutate_cache() {
        for bad in [
            input(0, 1, 0.0, 0),
            input(2, 0, 0.0, 0),
            input(MAX_COUNT + 1, 1, 0.0, 0),
            input(2, 1, f32::NAN, 0),
            Inputs {
                duration: 0.0,
                ..input(2, 1, 1.0, 0)
            },
            Inputs {
                duration: f32::MIN_POSITIVE,
                ..input(2, 1, f32::MAX, 0)
            },
        ] {
            let mut cache = CacheState::KnownMiss;
            assert!(sample_constant(bad, &mut cache).is_err());
            assert_eq!(cache, CacheState::KnownMiss);
        }
        let broken = CacheRecord {
            key_count: 4,
            normalized_time: 0.5,
            duration: 4.0,
            codec_flag: 0,
            selection: Selection {
                first: 4,
                next: 0,
                alpha: 0.5,
            },
        };
        let mut cache = CacheState::Known(broken);
        assert!(sample_constant(input(4, 4, 2.0, 0), &mut cache).is_err());
        assert_eq!(cache, CacheState::Known(broken));
        // A miss replaces unused result fields; it does not sample them.
        assert_eq!(
            sample_constant(input(4, 4, 1.0, 0), &mut cache)
                .unwrap()
                .cache_use,
            CacheUse::Miss
        );
        let nonfinite = CacheRecord {
            normalized_time: f32::NAN,
            ..broken
        };
        cache = CacheState::Known(nonfinite);
        assert!(sample_constant(input(4, 4, 2.0, 0), &mut cache).is_err());
        // Single-key selection never inspects even an unsupported cache key.
        assert_eq!(
            sample_constant(input(1, 4, 2.0, 0), &mut cache)
                .unwrap()
                .cache_use,
            CacheUse::Bypassed
        );
        match cache {
            CacheState::Known(record) => {
                assert_eq!(
                    record.normalized_time.to_bits(),
                    nonfinite.normalized_time.to_bits()
                );
                assert_eq!(record.selection, nonfinite.selection);
            }
            _ => panic!("expected unchanged unsupported cache"),
        }
    }
}
