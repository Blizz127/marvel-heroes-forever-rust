//! Original bounded button definitions and companions; no event dispatch or AVM1 execution.
//! All spans address the complete source movie, including retained action terminators.

use super::{ColorTransform, Filter, Matrix, Reader, Span, invalid};
use std::io;

pub const MAX_BUTTON_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_BUTTON_RECORDS: usize = 65_536;
pub const MAX_BUTTON_ACTIONS: usize = 65_536;
pub const MAX_BUTTON_FILTERS: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Up,
    Over,
    Down,
    HitTest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonRecord {
    pub source: Span,
    /// Original flags: low four bits select states; 0x10/0x20 select effects.
    pub flags: u8,
    pub character_id: u16,
    pub depth: u16,
    pub matrix: Matrix,
    /// Absent in DefineButton; its companion may supply an RGB transform.
    pub color: Option<ColorTransform>,
    pub filters: Option<Vec<Filter>>,
    /// Exact serialized value; reserved blend values are not interpreted.
    pub blend_mode: Option<u8>,
}

impl ButtonRecord {
    pub fn in_state(&self, state: ButtonState) -> bool {
        let mask = match state {
            ButtonState::Up => 1,
            ButtonState::Over => 2,
            ButtonState::Down => 4,
            ButtonState::HitTest => 8,
        };
        self.flags & mask != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonTransition {
    IdleToOverUp,
    OverUpToIdle,
    OverUpToOverDown,
    OverDownToOverUp,
    OverDownToOutDown,
    OutDownToOverDown,
    OutDownToIdle,
    IdleToOverDown,
    OverDownToIdle,
}

/// Raw little-endian condition word. Key bits are not transition bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonConditions(pub u16);

impl ButtonConditions {
    pub fn includes(self, transition: ButtonTransition) -> bool {
        let mask = match transition {
            ButtonTransition::IdleToOverUp => 0x001,
            ButtonTransition::OverUpToIdle => 0x002,
            ButtonTransition::OverUpToOverDown => 0x004,
            ButtonTransition::OverDownToOverUp => 0x008,
            ButtonTransition::OverDownToOutDown => 0x010,
            ButtonTransition::OutDownToOverDown => 0x020,
            ButtonTransition::OutDownToIdle => 0x040,
            ButtonTransition::IdleToOverDown => 0x080,
            ButtonTransition::OverDownToIdle => 0x100,
        };
        self.0 & mask != 0
    }

    pub fn key_code(self) -> Option<u8> {
        let code = (self.0 >> 9) as u8;
        (code != 0).then_some(code)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonAction {
    /// Includes the condition-record header for DefineButton2.
    pub source: Span,
    /// None for DefineButton's release action; Some(0) marks the final v2 record.
    pub declared_size: Option<u16>,
    pub conditions: Option<ButtonConditions>,
    /// Complete bytecode block including its final zero. Inner operands remain
    /// opaque here; pass these bytes and offset to avm1::Program::parse_at.
    pub actions: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Button {
    pub source: Span,
    pub tag_code: u16,
    pub id: u16,
    /// DefineButton has no serialized tracking flags or action offset.
    pub flags: Option<u8>,
    pub action_offset: Option<u16>,
    pub records: Vec<ButtonRecord>,
    pub actions: Vec<ButtonAction>,
}

impl Button {
    pub fn parse(
        tag_code: u16,
        body: &[u8],
        body_offset: usize,
        swf_version: u8,
    ) -> io::Result<Self> {
        let minimum = match tag_code {
            7 => 1,
            34 => 3,
            _ => return Err(unsupported("unsupported button definition tag")),
        };
        let mut r = reader(body, body_offset, swf_version, minimum)?;
        let id = nonzero_id(&mut r)?;
        let (flags, action_offset) = if tag_code == 34 {
            let flags = r.u8()?;
            if flags & !1 != 0 {
                return Err(invalid("nonzero reserved DefineButton2 flags"));
            }
            (Some(flags), Some(r.u16()?))
        } else {
            (None, None)
        };

        // Bound the display-list reader before the declared first action. An
        // invalid record cannot consume action bytes to manufacture its end.
        let character_end = match action_offset {
            Some(offset) if offset != 0 => 3usize
                .checked_add(usize::from(offset))
                .filter(|&end| end >= r.position && end < body.len())
                .ok_or_else(|| invalid("button action offset is outside its body"))?,
            _ => body.len(),
        };
        let mut characters = Reader::new(&body[r.position..character_end], r.absolute(), 0);
        let records = read_records(&mut characters, tag_code == 34, swf_version)?;
        if tag_code == 34 {
            characters.finish()?;
        }
        r.skip(characters.position)?;

        let mut actions = Vec::new();
        if tag_code == 7 {
            let span = remaining_actions(&mut r)?;
            actions.push(ButtonAction {
                source: span,
                declared_size: None,
                conditions: None,
                actions: span,
            });
        } else if action_offset != Some(0) {
            loop {
                if actions.len() >= MAX_BUTTON_ACTIONS {
                    return Err(invalid("button action-record limit exceeded"));
                }
                let start = r.absolute();
                let size = r.u16()?;
                let conditions = ButtonConditions(r.u16()?);
                if swf_version < 4 && conditions.key_code().is_some() {
                    return Err(unsupported("key-conditioned buttons require SWF 4"));
                }
                let action_bytes = if size == 0 {
                    r.remaining()
                } else {
                    usize::from(size).checked_sub(4).ok_or_else(|| {
                        invalid("button condition size is smaller than its header")
                    })?
                };
                let offset = r.absolute();
                require_action_end(r.take(action_bytes)?)?;
                actions.push(ButtonAction {
                    source: Span {
                        offset: start,
                        len: r.absolute() - start,
                    },
                    declared_size: Some(size),
                    conditions: Some(conditions),
                    actions: Span {
                        offset,
                        len: action_bytes,
                    },
                });
                if size == 0 {
                    break;
                }
                if r.remaining() == 0 {
                    return Err(invalid(
                        "button condition list has no final zero-size record",
                    ));
                }
            }
        }
        r.finish()?;
        Ok(Self {
            source: Span {
                offset: body_offset,
                len: body.len(),
            },
            tag_code,
            id,
            flags,
            action_offset,
            records,
            actions,
        })
    }

    pub fn track_as_menu(&self) -> bool {
        self.flags.is_some_and(|flags| flags & 1 != 0)
    }
}

fn read_records(
    r: &mut Reader<'_>,
    modern: bool,
    swf_version: u8,
) -> io::Result<Vec<ButtonRecord>> {
    let mut records = Vec::new();
    let mut total_filters = 0usize;
    loop {
        let offset = r.absolute();
        let flags = r.u8()?;
        if flags == 0 {
            break;
        }
        if records.len() >= MAX_BUTTON_RECORDS {
            return Err(invalid("button display-record limit exceeded"));
        }
        if flags & 0xc0 != 0 {
            return Err(invalid("nonzero reserved button-record flags"));
        }
        if flags & 0x30 != 0 && (!modern || swf_version < 8) {
            return Err(unsupported(
                "button effects require DefineButton2 and SWF 8",
            ));
        }
        let character_id = nonzero_id(r)?;
        let depth = r.u16()?;
        let matrix = r.matrix()?;
        let color = if modern {
            Some(r.color_transform(true)?)
        } else {
            None
        };
        let filters = if flags & 0x10 != 0 {
            let filters = r.filters()?;
            total_filters += filters.len();
            if total_filters > MAX_BUTTON_FILTERS {
                return Err(invalid("button filter-record limit exceeded"));
            }
            Some(filters)
        } else {
            None
        };
        let blend_mode = if flags & 0x20 != 0 {
            Some(r.u8()?)
        } else {
            None
        };
        records.push(ButtonRecord {
            source: Span {
                offset,
                len: r.absolute() - offset,
            },
            flags,
            character_id,
            depth,
            matrix,
            color,
            filters,
            blend_mode,
        });
    }
    Ok(records)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonColorTransform {
    pub source: Span,
    pub button_id: u16,
    pub color: ColorTransform,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoundEnvelope {
    pub source: Span,
    pub position_44khz: u32,
    pub left_level: u16,
    pub right_level: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoundInfo {
    pub source: Span,
    pub flags: u8,
    pub in_point: Option<u32>,
    pub out_point: Option<u32>,
    pub loop_count: Option<u16>,
    pub envelope: Option<Vec<SoundEnvelope>>,
}

impl SoundInfo {
    pub fn stop(&self) -> bool {
        self.flags & 0x20 != 0
    }
    pub fn no_multiple(&self) -> bool {
        self.flags & 0x10 != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonSound {
    pub source: Span,
    /// Zero is explicitly serialized when this transition has no sound.
    pub sound_id: u16,
    pub info: Option<SoundInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonSounds {
    pub source: Span,
    pub button_id: u16,
    /// In order: OverUpToIdle, IdleToOverUp, OverUpToOverDown, OverDownToOverUp.
    pub sounds: [ButtonSound; 4],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ButtonCompanion {
    ColorTransform(ButtonColorTransform),
    Sounds(ButtonSounds),
}

impl ButtonCompanion {
    pub fn parse(
        tag_code: u16,
        body: &[u8],
        body_offset: usize,
        swf_version: u8,
    ) -> io::Result<Self> {
        if !matches!(tag_code, 17 | 23) {
            return Err(unsupported("unsupported button companion tag"));
        }
        let mut r = reader(body, body_offset, swf_version, 2)?;
        let button_id = nonzero_id(&mut r)?;
        let source = Span {
            offset: body_offset,
            len: body.len(),
        };
        let result = if tag_code == 23 {
            Self::ColorTransform(ButtonColorTransform {
                source,
                button_id,
                color: r.color_transform(false)?,
            })
        } else {
            Self::Sounds(ButtonSounds {
                source,
                button_id,
                sounds: [
                    sound(&mut r)?,
                    sound(&mut r)?,
                    sound(&mut r)?,
                    sound(&mut r)?,
                ],
            })
        };
        r.finish()?;
        Ok(result)
    }
}

fn sound(r: &mut Reader<'_>) -> io::Result<ButtonSound> {
    let start = r.absolute();
    let sound_id = r.u16()?;
    let info = if sound_id == 0 {
        None
    } else {
        let info_start = r.absolute();
        let flags = r.u8()?;
        if flags & 0xc0 != 0 {
            return Err(invalid("nonzero reserved sound-info flags"));
        }
        let in_point = if flags & 1 != 0 { Some(r.u32()?) } else { None };
        let out_point = if flags & 2 != 0 { Some(r.u32()?) } else { None };
        let loop_count = if flags & 4 != 0 { Some(r.u16()?) } else { None };
        let envelope = if flags & 8 != 0 {
            let count = usize::from(r.u8()?);
            if count > r.remaining() / 8 {
                return Err(invalid("button sound envelope count exceeds its bytes"));
            }
            let mut records = Vec::with_capacity(count);
            for _ in 0..count {
                let offset = r.absolute();
                records.push(SoundEnvelope {
                    source: Span { offset, len: 8 },
                    position_44khz: r.u32()?,
                    left_level: r.u16()?,
                    right_level: r.u16()?,
                });
            }
            Some(records)
        } else {
            None
        };
        Some(SoundInfo {
            source: Span {
                offset: info_start,
                len: r.absolute() - info_start,
            },
            flags,
            in_point,
            out_point,
            loop_count,
            envelope,
        })
    };
    Ok(ButtonSound {
        source: Span {
            offset: start,
            len: r.absolute() - start,
        },
        sound_id,
        info,
    })
}

fn reader(body: &[u8], offset: usize, version: u8, minimum_version: u8) -> io::Result<Reader<'_>> {
    if body.len() > MAX_BUTTON_BYTES {
        return Err(invalid("button tag exceeds byte limit"));
    }
    offset
        .checked_add(body.len())
        .ok_or_else(|| invalid("button source span overflow"))?;
    if !(minimum_version..=19).contains(&version) {
        return Err(unsupported("unsupported SWF version for button tag"));
    }
    Ok(Reader::new(body, offset, 0))
}

fn nonzero_id(r: &mut Reader<'_>) -> io::Result<u16> {
    let id = r.u16()?;
    if id == 0 {
        Err(invalid("zero button or referenced character ID"))
    } else {
        Ok(id)
    }
}

fn remaining_actions(r: &mut Reader<'_>) -> io::Result<Span> {
    let offset = r.absolute();
    let len = r.remaining();
    require_action_end(r.take(len)?)?;
    Ok(Span { offset, len })
}

fn require_action_end(bytes: &[u8]) -> io::Result<()> {
    if bytes.last() == Some(&0) {
        Ok(())
    } else {
        Err(invalid("button action block lacks its final zero"))
    }
}

fn unsupported(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(out: &mut Vec<u8>, value: u16) {
        out.extend_from_slice(&value.to_le_bytes());
    }
    fn bits(fields: &[(u32, u8)]) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut position = 0usize;
        for &(value, width) in fields {
            for shift in (0..width).rev() {
                if position % 8 == 0 {
                    bytes.push(0);
                }
                bytes[position / 8] |= (((value >> shift) & 1) as u8) << (7 - position % 8);
                position += 1;
            }
        }
        bytes
    }
    fn modern() -> Vec<u8> {
        let mut bytes = vec![23, 0, 1, 0, 0];
        bytes.push(0x3f); // All states, one filter and explicit blend mode.
        word(&mut bytes, 7);
        word(&mut bytes, 2);
        bytes.extend(bits(&[(0, 1), (0, 1), (4, 5), (13, 4), (5, 4)]));
        bytes.extend(bits(&[
            (1, 1),
            (1, 1),
            (10, 4),
            (256, 10),
            (256, 10),
            (128, 10),
            (256, 10),
            (1021, 10),
            (4, 10),
            (0, 10),
            (0, 10),
        ]));
        bytes.extend_from_slice(&[1, 1]); // One blur filter.
        bytes.extend_from_slice(&[0; 9]);
        bytes.push(3);
        bytes.push(0); // Character list terminator.
        let action_offset = u16::try_from(bytes.len() - 3).unwrap();
        bytes[3..5].copy_from_slice(&action_offset.to_le_bytes());
        let first_actions = [0x96, 1, 0, 2, 0]; // Push null; End.
        word(&mut bytes, 4 + first_actions.len() as u16);
        word(&mut bytes, 0x0009);
        bytes.extend_from_slice(&first_actions);
        word(&mut bytes, 0);
        word(&mut bytes, (32 << 9) | 0x100);
        bytes.extend_from_slice(&[4, 0]);
        bytes
    }
    fn legacy() -> Vec<u8> {
        // One shared Up/Over record, identity matrix, no explicit color.
        vec![9, 0, 3, 7, 0, 2, 0, 0, 0, 0]
    }
    fn sounds() -> Vec<u8> {
        let mut bytes = vec![23, 0, 0, 0, 42, 0, 0x3f];
        bytes.extend_from_slice(&10u32.to_le_bytes());
        bytes.extend_from_slice(&100u32.to_le_bytes());
        word(&mut bytes, 3);
        bytes.push(1);
        bytes.extend_from_slice(&44_100u32.to_le_bytes());
        word(&mut bytes, 16384);
        word(&mut bytes, 32768);
        bytes.extend_from_slice(&[0; 4]);
        bytes
    }

    #[test]
    fn modern_records_preserve_states_signed_transforms_effects_and_absolute_spans() {
        let bytes = modern();
        let button = Button::parse(34, &bytes, 100, 10).unwrap();
        assert_eq!(button.id, 23);
        assert!(button.track_as_menu());
        let record = &button.records[0];
        for state in [
            ButtonState::Up,
            ButtonState::Over,
            ButtonState::Down,
            ButtonState::HitTest,
        ] {
            assert!(record.in_state(state));
        }
        assert_eq!(record.matrix.translate_x, -3);
        assert_eq!(record.matrix.translate_y, 5);
        assert_eq!(record.color.unwrap().multiply, [256, 256, 128, 256]);
        assert_eq!(record.color.unwrap().add, [-3, 4, 0, 0]);
        assert_eq!(record.blend_mode, Some(3));
        assert_eq!(record.source.offset, 105);
        let filter = &record.filters.as_ref().unwrap()[0];
        assert_eq!(filter.kind, 1);
        assert_eq!(filter.payload.len, 9);
        let mut source = vec![0; 100];
        source.extend_from_slice(&bytes);
        assert_eq!(filter.payload.bytes(&source).unwrap(), [0; 9]);
        assert_eq!(
            button.actions[0].actions.bytes(&source).unwrap(),
            [0x96, 1, 0, 2, 0]
        );
        assert_eq!(button.actions[1].actions.bytes(&source).unwrap(), [4, 0]);
        let first = button.actions[0].conditions.unwrap();
        assert!(first.includes(ButtonTransition::IdleToOverUp));
        assert!(first.includes(ButtonTransition::OverDownToOverUp));
        assert!(!first.includes(ButtonTransition::OverUpToIdle));
        assert_eq!(first.key_code(), None);
        let second = button.actions[1].conditions.unwrap();
        assert_eq!(second.key_code(), Some(32));
        assert!(second.includes(ButtonTransition::OverDownToIdle));
        assert_eq!(button.actions[1].declared_size, Some(0));
    }

    #[test]
    fn legacy_button_and_action_free_modern_button_remain_distinct() {
        let button = Button::parse(7, &legacy(), 20, 1).unwrap();
        assert_eq!(button.flags, None);
        assert_eq!(button.action_offset, None);
        assert_eq!(button.records[0].color, None);
        assert_eq!(button.records[0].matrix, Matrix::default());
        assert_eq!(button.actions[0].conditions, None);
        assert_eq!(button.actions[0].actions, Span { offset: 29, len: 1 });
        let mut bytes = modern();
        let start = 3 + usize::from(u16::from_le_bytes(bytes[3..5].try_into().unwrap()));
        bytes.truncate(start);
        bytes[3..5].fill(0);
        let button = Button::parse(34, &bytes, 0, 10).unwrap();
        assert!(button.actions.is_empty());
        bytes.push(0);
        assert!(Button::parse(34, &bytes, 0, 10).is_err());
    }

    #[test]
    fn companions_preserve_rgb_transform_and_optional_sound_style_fields() {
        let color = ButtonCompanion::parse(23, &[9, 0, 0], 20, 2).unwrap();
        let ButtonCompanion::ColorTransform(color) = color else {
            panic!()
        };
        assert_eq!(color.color, ColorTransform::default());
        assert_eq!(color.button_id, 9);
        let bytes = sounds();
        let ButtonCompanion::Sounds(value) = ButtonCompanion::parse(17, &bytes, 50, 10).unwrap()
        else {
            panic!()
        };
        assert_eq!(value.sounds[0].sound_id, 0);
        assert_eq!(value.sounds[0].info, None);
        let info = value.sounds[1].info.as_ref().unwrap();
        assert!(info.stop() && info.no_multiple());
        assert_eq!(info.in_point, Some(10));
        assert_eq!(info.out_point, Some(100));
        assert_eq!(info.loop_count, Some(3));
        let envelope = &info.envelope.as_ref().unwrap()[0];
        assert_eq!(
            (
                envelope.position_44khz,
                envelope.left_level,
                envelope.right_level
            ),
            (44100, 16384, 32768)
        );
        assert_eq!(envelope.source, Span { offset: 68, len: 8 });
    }

    #[test]
    fn rejects_truncation_source_overflow_and_malformed_offsets_or_terminators() {
        for (tag, bytes) in [(7, legacy()), (34, modern())] {
            for cut in 0..bytes.len() {
                assert!(
                    Button::parse(tag, &bytes[..cut], 100, 10).is_err(),
                    "tag {tag}, cut {cut}"
                );
            }
            assert!(Button::parse(tag, &bytes, usize::MAX - 1, 10).is_err());
        }
        for value in [1u16, 2, 3, u16::MAX] {
            let mut bytes = modern();
            bytes[3..5].copy_from_slice(&value.to_le_bytes());
            assert!(Button::parse(34, &bytes, 0, 10).is_err());
        }
        let original = modern();
        let action = 3 + usize::from(u16::from_le_bytes(original[3..5].try_into().unwrap()));
        for size in [1u16, 3, u16::MAX] {
            let mut bytes = original.clone();
            bytes[action..action + 2].copy_from_slice(&size.to_le_bytes());
            assert!(Button::parse(34, &bytes, 0, 10).is_err());
        }
        let mut missing_end = original;
        *missing_end.last_mut().unwrap() = 4;
        assert!(Button::parse(34, &missing_end, 0, 10).is_err());
        let bytes = sounds();
        for cut in 0..bytes.len() {
            assert!(ButtonCompanion::parse(17, &bytes[..cut], 0, 10).is_err());
        }
    }

    #[test]
    fn rejects_reserved_flags_incompatible_versions_and_body_budget() {
        for index in [2, 5] {
            let mut bytes = modern();
            bytes[index] |= 0x80;
            assert!(Button::parse(34, &bytes, 0, 10).is_err());
        }
        assert!(Button::parse(34, &modern(), 0, 7).is_err());
        assert!(Button::parse(34, &modern(), 0, 2).is_err());
        assert!(Button::parse(7, &legacy(), 0, 0).is_err());
        assert!(Button::parse(99, &legacy(), 0, 10).is_err());
        assert!(Button::parse(7, &vec![0; MAX_BUTTON_BYTES + 1], 0, 10).is_err());
        assert!(ButtonCompanion::parse(23, &[1, 0, 0], 0, 1).is_err());
        assert!(ButtonCompanion::parse(17, &[1, 0, 7, 0, 0x80], 0, 10).is_err());
        let mut bytes = sounds();
        bytes[17] = 255;
        assert!(ButtonCompanion::parse(17, &bytes, 0, 10).is_err());
    }
}
