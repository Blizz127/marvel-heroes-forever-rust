//! Bounded AVM1 action decoder with a separate, partial execution core in `vm`.
//! Decoding itself never executes actions or invokes host callbacks.
//!
//! See docs/AVM1_FORMAT.md for source provenance and measured coverage.
#[path = "avm1/vm.rs"]
pub mod vm;

use std::collections::BTreeMap;
use std::io;

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub max_bytes: usize,
    pub max_instructions: usize,
    pub max_nesting: usize,
    pub max_string_bytes: usize,
    pub max_total_string_bytes: usize,
    /// Combined limit for pushed values, pool entries and function parameters.
    pub max_values: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_bytes: 64 * 1024 * 1024,
            max_instructions: 1_000_000,
            max_nesting: 64,
            max_string_bytes: 65_536,
            max_total_string_bytes: 32 * 1024 * 1024,
            max_values: 1_000_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub offset: usize,
    pub len: usize,
}
impl Span {
    pub fn end(self) -> Option<usize> {
        self.offset.checked_add(self.len)
    }
    /// Index into the complete source movie (not an offset-adjusted slice).
    pub fn bytes(self, source: &[u8]) -> io::Result<&[u8]> {
        source
            .get(self.offset..self.end().ok_or_else(|| invalid("span overflow"))?)
            .ok_or_else(|| invalid("span outside source"))
    }
}

/// Bytes excluding the NUL terminator. No guessed legacy codepage conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteString(pub Vec<u8>);
impl ByteString {
    pub fn utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    String(ByteString),
    FloatBits(u32),
    Null,
    Undefined,
    Register(u8),
    /// Preserve even noncanonical Boolean encodings.
    Boolean(u8),
    DoubleBits(u64),
    Integer(u32),
    Constant8(u8),
    Constant16(u16),
    /// Unknown push type: the rest of this action cannot be split reliably.
    Unknown {
        kind: u8,
        remaining: Span,
    },
}
impl Literal {
    /// Caller supplies the active runtime pool; this does not infer control flow.
    pub fn constant<'a>(&self, pool: &'a [ByteString]) -> Option<&'a ByteString> {
        let index = match self {
            Self::Constant8(v) => usize::from(*v),
            Self::Constant16(v) => usize::from(*v),
            _ => return None,
        };
        pool.get(index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    Instruction(usize),
    BlockEnd,
    /// An offset inside this block, but not at a decoded action boundary.
    InsideRecordOrTrailing,
    /// May be meaningful for With/Try; structural decoding does not execute it.
    OutsideBlock,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub displacement: i16,
    /// Absolute source offset, signed to preserve branches before source zero.
    pub target: i128,
    pub destination: Destination,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    /// None for DefineFunction; Some(0) denotes a named Function2 parameter.
    pub register: Option<u8>,
    pub name: ByteString,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: ByteString,
    pub parameters: Vec<Parameter>,
    pub register_count: Option<u8>,
    /// Little-endian flag word, preserved including reserved bits.
    pub flags: Option<u16>,
    pub body: Program,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatchTarget {
    Name(ByteString),
    Register(u8),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TryBlock {
    pub flags: u8,
    pub catch_target: CatchTarget,
    pub try_body: Program,
    pub catch_body: Program,
    pub finally_body: Program,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operands {
    None,
    GotoFrame(u16),
    GetUrl {
        url: ByteString,
        target: ByteString,
    },
    StoreRegister(u8),
    ConstantPool(Vec<ByteString>),
    WaitForFrame {
        frame: Option<u16>,
        skip_count: u8,
    },
    SetTarget(ByteString),
    GotoLabel(ByteString),
    Function(Function),
    Try(TryBlock),
    With(Program),
    Push(Vec<Literal>),
    Branch(Branch),
    /// Flags retained verbatim; never performs a network request.
    GetUrl2(u8),
    GotoFrame2 {
        flags: u8,
        scene_bias: Option<u16>,
    },
    /// Unknown opcode; the length-delimited payload is retained in Instruction.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: u8,
    /// Header and declared payload only. Nested bodies may follow this span.
    pub record: Span,
    pub payload: Span,
    /// Complete enclosing instruction including any following nested bodies.
    pub span: Span,
    pub operands: Operands,
}
impl Instruction {
    pub fn name(&self) -> Option<&'static str> {
        opcode_name(self.opcode)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub span: Span,
    /// Includes End (0x00) if present; sized nested blocks need not contain End.
    pub instructions: Vec<Instruction>,
    /// Bytes after first End are preserved, never interpreted as instructions.
    pub trailing: Span,
    pub terminated: bool,
}
impl Program {
    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        Self::parse_at(bytes, 0)
    }
    pub fn parse_at(bytes: &[u8], source_offset: usize) -> io::Result<Self> {
        Self::parse_with_limits(bytes, source_offset, Limits::default())
    }
    pub fn parse_with_limits(
        bytes: &[u8],
        source_offset: usize,
        limits: Limits,
    ) -> io::Result<Self> {
        if bytes.len() > limits.max_bytes {
            return Err(invalid("AVM1 input exceeds byte limit"));
        }
        if limits.max_nesting > 256 {
            return Err(invalid("AVM1 nesting limit exceeds hard maximum 256"));
        }
        source_offset
            .checked_add(bytes.len())
            .ok_or_else(|| invalid("AVM1 source span overflow"))?;
        let mut decoder = Decoder {
            limits,
            instructions: 0,
            values: 0,
            strings: 0,
        };
        decoder.program(bytes, source_offset, 0)
    }
    /// Find an exact decoded boundary, including nested bodies. This is a source
    /// lookup, not permission for a runtime to branch across function scopes.
    pub fn instruction_at(&self, source_offset: usize) -> Option<&Instruction> {
        let index = self
            .instructions
            .partition_point(|i| i.span.offset <= source_offset);
        let instruction = self.instructions.get(index.checked_sub(1)?)?;
        if instruction.span.offset == source_offset {
            return Some(instruction);
        }
        if source_offset >= instruction.span.end()? {
            return None;
        }
        match &instruction.operands {
            Operands::Function(f) => f.body.instruction_at(source_offset),
            Operands::With(body) => body.instruction_at(source_offset),
            Operands::Try(block) => block
                .try_body
                .instruction_at(source_offset)
                .or_else(|| block.catch_body.instruction_at(source_offset))
                .or_else(|| block.finally_body.instruction_at(source_offset)),
            _ => None,
        }
    }
    pub fn coverage(&self) -> Coverage {
        let mut result = Coverage::default();
        self.add_coverage(&mut result, 0);
        result
    }
    fn add_coverage(&self, out: &mut Coverage, depth: usize) {
        out.blocks += 1;
        out.max_nesting = out.max_nesting.max(depth);
        out.trailing_bytes += self.trailing.len;
        for instruction in &self.instructions {
            *out.opcodes.entry(instruction.opcode).or_default() += 1;
            match &instruction.operands {
                Operands::Unknown => {
                    *out.unknown_opcodes.entry(instruction.opcode).or_default() += 1;
                }
                Operands::Push(values) => {
                    for value in values {
                        if let Literal::Unknown { kind, .. } = value {
                            *out.unknown_push_types.entry(*kind).or_default() += 1;
                        }
                    }
                }
                Operands::Branch(branch) => {
                    out.branches += 1;
                    match branch.destination {
                        Destination::Instruction(_) | Destination::BlockEnd => {}
                        Destination::InsideRecordOrTrailing => out.branches_inside_records += 1,
                        Destination::OutsideBlock => out.branches_outside_blocks += 1,
                    }
                }
                Operands::Function(function) => function.body.add_coverage(out, depth + 1),
                Operands::With(body) => body.add_coverage(out, depth + 1),
                Operands::Try(block) => {
                    block.try_body.add_coverage(out, depth + 1);
                    block.catch_body.add_coverage(out, depth + 1);
                    block.finally_body.add_coverage(out, depth + 1);
                }
                _ => {}
            }
        }
    }
}
#[derive(Debug, Clone, Default)]
pub struct Coverage {
    pub opcodes: BTreeMap<u8, usize>,
    pub unknown_opcodes: BTreeMap<u8, usize>,
    pub unknown_push_types: BTreeMap<u8, usize>,
    pub blocks: usize,
    pub max_nesting: usize,
    pub trailing_bytes: usize,
    pub branches: usize,
    pub branches_inside_records: usize,
    pub branches_outside_blocks: usize,
}

struct Decoder {
    limits: Limits,
    instructions: usize,
    values: usize,
    strings: usize,
}
impl Decoder {
    fn value(&mut self) -> io::Result<()> {
        if self.values >= self.limits.max_values {
            return Err(invalid("AVM1 value budget exhausted"));
        }
        self.values += 1;
        Ok(())
    }
    fn string(&mut self, r: &mut Reader<'_>) -> io::Result<ByteString> {
        let available = r.bytes.len() - r.pos;
        let inspect = available.min(self.limits.max_string_bytes.saturating_add(1));
        let len = r.bytes[r.pos..r.pos + inspect]
            .iter()
            .position(|b| *b == 0)
            .ok_or_else(|| invalid_at(r.absolute(), "unterminated or over-limit AVM1 string"))?;
        if len
            > self
                .limits
                .max_total_string_bytes
                .saturating_sub(self.strings)
        {
            return Err(invalid("AVM1 string budget exhausted"));
        }
        self.strings += len;
        let bytes = r.take(len)?.to_vec();
        r.u8()?;
        Ok(ByteString(bytes))
    }
    fn nested(&mut self, r: &mut Reader<'_>, len: usize, depth: usize) -> io::Result<Program> {
        let offset = r.absolute();
        self.program(r.take(len)?, offset, depth + 1)
    }
    fn program(&mut self, bytes: &[u8], offset: usize, depth: usize) -> io::Result<Program> {
        if depth > self.limits.max_nesting {
            return Err(invalid("AVM1 nesting budget exhausted"));
        }
        let mut r = Reader {
            bytes,
            pos: 0,
            offset,
        };
        let mut instructions = Vec::new();
        let mut terminated = false;
        while r.pos < bytes.len() {
            if self.instructions >= self.limits.max_instructions {
                return Err(invalid("AVM1 instruction budget exhausted"));
            }
            self.instructions += 1;
            let start = r.absolute();
            let opcode = r.u8()?;
            let len = if opcode >= 0x80 {
                usize::from(r.u16()?)
            } else {
                0
            };
            let payload = Span {
                offset: r.absolute(),
                len,
            };
            let data = r.take(len)?;
            let record = Span {
                offset: start,
                len: r.absolute() - start,
            };
            let mut p = Reader {
                bytes: data,
                pos: 0,
                offset: payload.offset,
            };
            let operands = match opcode {
                0x81 => Operands::GotoFrame(p.u16()?),
                0x83 => Operands::GetUrl {
                    url: self.string(&mut p)?,
                    target: self.string(&mut p)?,
                },
                0x87 => Operands::StoreRegister(p.u8()?),
                0x88 => {
                    let count = p.u16()?;
                    let mut pool = Vec::new();
                    for _ in 0..count {
                        self.value()?;
                        pool.push(self.string(&mut p)?);
                    }
                    Operands::ConstantPool(pool)
                }
                0x8a => Operands::WaitForFrame {
                    frame: Some(p.u16()?),
                    skip_count: p.u8()?,
                },
                0x8b => Operands::SetTarget(self.string(&mut p)?),
                0x8c => Operands::GotoLabel(self.string(&mut p)?),
                0x8d => Operands::WaitForFrame {
                    frame: None,
                    skip_count: p.u8()?,
                },
                0x8e | 0x9b => {
                    let name = self.string(&mut p)?;
                    let count = p.u16()?;
                    let register_count = if opcode == 0x8e { Some(p.u8()?) } else { None };
                    let flags = if opcode == 0x8e { Some(p.u16()?) } else { None };
                    let mut parameters = Vec::new();
                    for _ in 0..count {
                        self.value()?;
                        let register = if opcode == 0x8e { Some(p.u8()?) } else { None };
                        parameters.push(Parameter {
                            register,
                            name: self.string(&mut p)?,
                        });
                    }
                    let body_len = usize::from(p.u16()?);
                    p.finish()?;
                    let body = self.nested(&mut r, body_len, depth)?;
                    Operands::Function(Function {
                        name,
                        parameters,
                        register_count,
                        flags,
                        body,
                    })
                }
                0x8f => {
                    let flags = p.u8()?;
                    let try_len = usize::from(p.u16()?);
                    let catch_len = usize::from(p.u16()?);
                    let finally_len = usize::from(p.u16()?);
                    let catch_target = if flags & 4 != 0 {
                        CatchTarget::Register(p.u8()?)
                    } else {
                        CatchTarget::Name(self.string(&mut p)?)
                    };
                    p.finish()?;
                    let try_body = self.nested(&mut r, try_len, depth)?;
                    let catch_body = self.nested(&mut r, catch_len, depth)?;
                    let finally_body = self.nested(&mut r, finally_len, depth)?;
                    Operands::Try(TryBlock {
                        flags,
                        catch_target,
                        try_body,
                        catch_body,
                        finally_body,
                    })
                }
                0x94 => {
                    let body_len = usize::from(p.u16()?);
                    p.finish()?;
                    Operands::With(self.nested(&mut r, body_len, depth)?)
                }
                0x96 => {
                    let mut values = Vec::new();
                    while p.pos < p.bytes.len() {
                        self.value()?;
                        let kind = p.u8()?;
                        let value = match kind {
                            0 => Literal::String(self.string(&mut p)?),
                            1 => Literal::FloatBits(p.u32()?),
                            2 => Literal::Null,
                            3 => Literal::Undefined,
                            4 => Literal::Register(p.u8()?),
                            5 => Literal::Boolean(p.u8()?),
                            6 => {
                                // SWF DOUBLE stores high little-endian word first.
                                let high = u64::from(p.u32()?);
                                let low = u64::from(p.u32()?);
                                Literal::DoubleBits((high << 32) | low)
                            }
                            7 => Literal::Integer(p.u32()?),
                            8 => Literal::Constant8(p.u8()?),
                            9 => Literal::Constant16(p.u16()?),
                            _ => {
                                let remaining = Span {
                                    offset: p.absolute(),
                                    len: p.bytes.len() - p.pos,
                                };
                                p.pos = p.bytes.len();
                                Literal::Unknown { kind, remaining }
                            }
                        };
                        values.push(value);
                    }
                    Operands::Push(values)
                }
                0x99 | 0x9d => {
                    let displacement = p.u16()? as i16;
                    Operands::Branch(Branch {
                        displacement,
                        target: r.absolute() as i128 + i128::from(displacement),
                        destination: Destination::OutsideBlock,
                    })
                }
                0x9a => Operands::GetUrl2(p.u8()?),
                0x9f => {
                    let flags = p.u8()?;
                    let scene_bias = if flags & 2 != 0 { Some(p.u16()?) } else { None };
                    Operands::GotoFrame2 { flags, scene_bias }
                }
                _ if opcode_name(opcode).is_some() => Operands::None,
                _ => {
                    p.pos = p.bytes.len();
                    Operands::Unknown
                }
            };
            p.finish()?;
            instructions.push(Instruction {
                opcode,
                record,
                payload,
                span: Span {
                    offset: start,
                    len: r.absolute() - start,
                },
                operands,
            });
            if opcode == 0 {
                terminated = true;
                break;
            }
        }
        let boundaries: BTreeMap<usize, usize> = instructions
            .iter()
            .enumerate()
            .map(|(i, instruction)| (instruction.span.offset, i))
            .collect();
        let block_end = offset + bytes.len();
        for instruction in &mut instructions {
            if let Operands::Branch(branch) = &mut instruction.operands {
                branch.destination =
                    if branch.target < offset as i128 || branch.target > block_end as i128 {
                        Destination::OutsideBlock
                    } else if branch.target == block_end as i128 {
                        Destination::BlockEnd
                    } else if let Some(index) = boundaries.get(&(branch.target as usize)) {
                        Destination::Instruction(*index)
                    } else {
                        Destination::InsideRecordOrTrailing
                    };
            }
        }
        Ok(Program {
            span: Span {
                offset,
                len: bytes.len(),
            },
            instructions,
            terminated,
            trailing: Span {
                offset: r.absolute(),
                len: bytes.len() - r.pos,
            },
        })
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
    offset: usize,
}
impl<'a> Reader<'a> {
    fn absolute(&self) -> usize {
        self.offset + self.pos
    }
    fn take(&mut self, len: usize) -> io::Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| invalid("AVM1 length overflow"))?;
        let result = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| invalid_at(self.absolute(), "truncated AVM1 operand or nested body"))?;
        self.pos = end;
        Ok(result)
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> io::Result<u16> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn u32(&mut self) -> io::Result<u32> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn finish(&self) -> io::Result<()> {
        if self.pos != self.bytes.len() {
            Err(invalid_at(
                self.absolute(),
                "unexpected bytes in known AVM1 operand",
            ))
        } else {
            Ok(())
        }
    }
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn invalid_at(offset: usize, message: &str) -> io::Error {
    invalid(&format!("{message} at source offset {offset}"))
}

/// Documented Adobe AVM1 opcodes. Unknown/extension bytes return None.
pub fn opcode_name(opcode: u8) -> Option<&'static str> {
    Some(match opcode {
        0x00 => "End",
        0x04 => "NextFrame",
        0x05 => "PreviousFrame",
        0x06 => "Play",
        0x07 => "Stop",
        0x08 => "ToggleQuality",
        0x09 => "StopSounds",
        0x0a => "Add",
        0x0b => "Subtract",
        0x0c => "Multiply",
        0x0d => "Divide",
        0x0e => "Equals",
        0x0f => "Less",
        0x10 => "And",
        0x11 => "Or",
        0x12 => "Not",
        0x13 => "StringEquals",
        0x14 => "StringLength",
        0x15 => "StringExtract",
        0x17 => "Pop",
        0x18 => "ToInteger",
        0x1c => "GetVariable",
        0x1d => "SetVariable",
        0x20 => "SetTarget2",
        0x21 => "StringAdd",
        0x22 => "GetProperty",
        0x23 => "SetProperty",
        0x24 => "CloneSprite",
        0x25 => "RemoveSprite",
        0x26 => "Trace",
        0x27 => "StartDrag",
        0x28 => "EndDrag",
        0x29 => "StringLess",
        0x2a => "Throw",
        0x2b => "CastOp",
        0x2c => "ImplementsOp",
        0x30 => "RandomNumber",
        0x31 => "MBStringLength",
        0x32 => "CharToAscii",
        0x33 => "AsciiToChar",
        0x34 => "GetTime",
        0x35 => "MBStringExtract",
        0x36 => "MBCharToAscii",
        0x37 => "MBAsciiToChar",
        0x3a => "Delete",
        0x3b => "Delete2",
        0x3c => "DefineLocal",
        0x3d => "CallFunction",
        0x3e => "Return",
        0x3f => "Modulo",
        0x40 => "NewObject",
        0x41 => "DefineLocal2",
        0x42 => "InitArray",
        0x43 => "InitObject",
        0x44 => "TypeOf",
        0x45 => "TargetPath",
        0x46 => "Enumerate",
        0x47 => "Add2",
        0x48 => "Less2",
        0x49 => "Equals2",
        0x4a => "ToNumber",
        0x4b => "ToString",
        0x4c => "PushDuplicate",
        0x4d => "StackSwap",
        0x4e => "GetMember",
        0x4f => "SetMember",
        0x50 => "Increment",
        0x51 => "Decrement",
        0x52 => "CallMethod",
        0x53 => "NewMethod",
        0x54 => "InstanceOf",
        0x55 => "Enumerate2",
        0x60 => "BitAnd",
        0x61 => "BitOr",
        0x62 => "BitXor",
        0x63 => "BitLShift",
        0x64 => "BitRShift",
        0x65 => "BitURShift",
        0x66 => "StrictEquals",
        0x67 => "Greater",
        0x68 => "StringGreater",
        0x69 => "Extends",
        0x81 => "GotoFrame",
        0x83 => "GetURL",
        0x87 => "StoreRegister",
        0x88 => "ConstantPool",
        0x8a => "WaitForFrame",
        0x8b => "SetTarget",
        0x8c => "GotoLabel",
        0x8d => "WaitForFrame2",
        0x8e => "DefineFunction2",
        0x8f => "Try",
        0x94 => "With",
        0x96 => "Push",
        0x99 => "Jump",
        0x9a => "GetURL2",
        0x9b => "DefineFunction",
        0x9d => "If",
        0x9e => "Call",
        0x9f => "GotoFrame2",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(opcode: u8, payload: &[u8]) -> Vec<u8> {
        assert!(opcode >= 0x80);
        let mut bytes = vec![opcode];
        bytes.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        bytes.extend_from_slice(payload);
        bytes
    }

    #[test]
    fn push_values_preserve_bits_and_resolve_only_supplied_pool() {
        let payload = [
            0, b'x', 0, 1, 0, 0, 0x80, 0x3f, 2, 3, 4, 7, 5, 2, 6, 0, 0, 0xe8, 0x3f, 0, 0, 0, 0, 7,
            0xff, 0xff, 0xff, 0xff, 8, 1, 9, 0, 1,
        ];
        let program = Program::parse(&action(0x96, &payload)).unwrap();
        let Operands::Push(values) = &program.instructions[0].operands else {
            panic!()
        };
        assert_eq!(values.len(), 10);
        assert_eq!(values[1], Literal::FloatBits(1.0f32.to_bits()));
        assert_eq!(values[5], Literal::Boolean(2));
        assert_eq!(values[6], Literal::DoubleBits(0.75f64.to_bits()));
        assert_eq!(values[7], Literal::Integer(u32::MAX));
        assert_eq!(values[9], Literal::Constant16(256));
        let pool = [ByteString(b"zero".to_vec()), ByteString(b"one".to_vec())];
        assert_eq!(values[8].constant(&pool).unwrap().utf8().unwrap(), "one");
        assert!(values[9].constant(&pool).is_none());
    }

    #[test]
    fn pools_and_strings_keep_original_encoding() {
        let mut bytes = action(0x88, &[2, 0, b'a', 0, 0xff, 0]);
        bytes.extend(action(0x96, &[8, 1]));
        let program = Program::parse(&bytes).unwrap();
        let Operands::ConstantPool(pool) = &program.instructions[0].operands else {
            panic!()
        };
        assert_eq!(pool.len(), 2);
        assert!(pool[1].utf8().is_err());
        let Operands::Push(values) = &program.instructions[1].operands else {
            panic!()
        };
        assert_eq!(values[0].constant(pool), Some(&pool[1]));
    }

    #[test]
    fn functions_have_external_bodies_and_absolute_offsets() {
        let mut bytes = action(0x9b, &[b'f', 0, 1, 0, b'x', 0, 2, 0]);
        bytes.extend([0x04, 0x3e, 0x07, 0]);
        let program = Program::parse_at(&bytes, 100).unwrap();
        let first = &program.instructions[0];
        assert_eq!(
            first.record,
            Span {
                offset: 100,
                len: 11
            }
        );
        assert_eq!(
            first.span,
            Span {
                offset: 100,
                len: 13
            }
        );
        let Operands::Function(f) = &first.operands else {
            panic!()
        };
        assert_eq!(
            f.body.span,
            Span {
                offset: 111,
                len: 2
            }
        );
        assert_eq!(f.parameters[0].name.utf8().unwrap(), "x");
        assert_eq!(f.parameters[0].register, None);
        assert!(!f.body.terminated);
        assert_eq!(program.instructions[1].opcode, 7);
        assert!(program.terminated);
    }

    #[test]
    fn function2_flags_registers_and_nested_function() {
        let mut inner = action(0x9b, &[0, 0, 0, 1, 0]);
        inner.push(0x3e);
        let mut payload = vec![0, 1, 0, 5, 0x81, 1, 3, b'p', 0];
        payload.extend_from_slice(&(inner.len() as u16).to_le_bytes());
        let mut bytes = action(0x8e, &payload);
        bytes.extend(inner);
        let program = Program::parse(&bytes).unwrap();
        let Operands::Function(f) = &program.instructions[0].operands else {
            panic!()
        };
        assert_eq!(f.register_count, Some(5));
        assert_eq!(f.flags, Some(0x181));
        assert_eq!(f.parameters[0].register, Some(3));
        assert_eq!(program.coverage().max_nesting, 2);
        assert!(matches!(
            f.body.instructions[0].operands,
            Operands::Function(_)
        ));
    }

    #[test]
    fn try_bodies_use_sizes_and_may_branch_past_catch() {
        let mut bytes = action(0x8f, &[5, 5, 0, 1, 0, 0, 0, 2]);
        bytes.extend(action(0x99, &[1, 0]));
        bytes.push(0x3e);
        bytes.push(0x07);
        let program = Program::parse(&bytes).unwrap();
        let first = &program.instructions[0];
        assert_eq!(first.record.len, 11);
        assert_eq!(first.span.len, 17);
        let Operands::Try(block) = &first.operands else {
            panic!()
        };
        assert_eq!(block.catch_target, CatchTarget::Register(2));
        assert_eq!(block.try_body.span.len, 5);
        assert_eq!(block.catch_body.span.len, 1);
        assert!(block.finally_body.instructions.is_empty());
        let Operands::Branch(branch) = &block.try_body.instructions[0].operands else {
            panic!()
        };
        assert_eq!(branch.target, 17);
        assert_eq!(branch.destination, Destination::OutsideBlock);
        assert_eq!(program.instructions[1].span.offset, 17);
        assert_eq!(program.instruction_at(11).unwrap().opcode, 0x99);
        assert_eq!(program.instruction_at(16).unwrap().opcode, 0x3e);
        assert!(program.instruction_at(12).is_none());
        assert_eq!(program.coverage().branches_outside_blocks, 1);
    }

    #[test]
    fn with_and_named_catch_keep_nested_boundaries() {
        let mut bytes = action(0x94, &[1, 0]);
        bytes.push(0x04);
        bytes.extend(action(0x8f, &[3, 1, 0, 1, 0, 1, 0, b'e', 0]));
        bytes.extend([0x2a, 0x17, 0x07]);
        let program = Program::parse(&bytes).unwrap();
        let Operands::With(body) = &program.instructions[0].operands else {
            panic!()
        };
        assert_eq!(body.span, Span { offset: 5, len: 1 });
        let Operands::Try(block) = &program.instructions[1].operands else {
            panic!()
        };
        assert_eq!(
            block.catch_target,
            CatchTarget::Name(ByteString(b"e".to_vec()))
        );
        assert_eq!(block.finally_body.instructions[0].opcode, 7);
    }

    #[test]
    fn branch_displacements_classify_boundaries_without_execution() {
        let mut bytes = action(0x99, &(-5i16).to_le_bytes());
        bytes.extend(action(0x9d, &[10, 0]));
        bytes.extend(action(0x99, &(-9i16).to_le_bytes()));
        bytes.extend(action(0x99, &(-100i16).to_le_bytes()));
        let program = Program::parse_at(&bytes, 1).unwrap();
        let branches: Vec<_> = program
            .instructions
            .iter()
            .map(|i| {
                let Operands::Branch(b) = &i.operands else {
                    panic!()
                };
                b
            })
            .collect();
        assert_eq!(branches[0].target, 1);
        assert_eq!(branches[0].destination, Destination::Instruction(0));
        assert_eq!(branches[1].destination, Destination::BlockEnd);
        assert_eq!(branches[2].destination, Destination::InsideRecordOrTrailing);
        assert_eq!(branches[3].target, -79);
        assert_eq!(branches[3].destination, Destination::OutsideBlock);
    }

    #[test]
    fn unknown_opcodes_and_push_types_remain_bounded() {
        let mut bytes = action(0xfe, &[0x96, 0, 0]);
        bytes.push(0x7f);
        bytes.extend(action(0x96, &[2, 0xfd, 0xff, 0xff]));
        bytes.extend([0x07, 0, 0xff]);
        let program = Program::parse(&bytes).unwrap();
        assert_eq!(program.instructions.len(), 5);
        assert_eq!(program.trailing.len, 1);
        assert_eq!(program.coverage().unknown_opcodes.len(), 2);
        assert_eq!(program.coverage().unknown_push_types.get(&0xfd), Some(&1));
        let Operands::Push(values) = &program.instructions[2].operands else {
            panic!()
        };
        assert_eq!(values.len(), 2);
        let Literal::Unknown { remaining, .. } = &values[1] else {
            panic!()
        };
        assert_eq!(remaining.bytes(&bytes).unwrap(), &[0xff, 0xff]);
    }

    #[test]
    fn malformed_lengths_and_known_operands_fail_closed() {
        for bytes in [
            vec![0x96],
            vec![0x96, 1],
            vec![0x96, 2, 0, 0],
            action(0x96, &[0, b'x']),
            action(0x96, &[6, 1, 2, 3]),
            action(0x81, &[1]),
            action(0x81, &[1, 0, 9]),
            action(0x94, &[1, 0]),
            action(0x9b, &[0, 0, 0, 1, 0]),
            action(0x88, &[2, 0, b'a', 0]),
            action(0x9e, &[0]),
            action(0x9f, &[2]),
        ] {
            assert!(Program::parse(&bytes).is_err(), "accepted {bytes:?}");
        }
        assert!(Program::parse_at(&[0], usize::MAX).is_err());
        assert!(
            Span {
                offset: usize::MAX,
                len: 1
            }
            .bytes(&[])
            .is_err()
        );
    }

    #[test]
    fn budgets_apply_across_nested_bodies_and_pool_entries() {
        let mut bytes = action(0x94, &[2, 0]);
        bytes.extend([4, 4]);
        assert!(
            Program::parse_with_limits(
                &bytes,
                0,
                Limits {
                    max_nesting: 0,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        assert!(
            Program::parse_with_limits(
                &bytes,
                0,
                Limits {
                    max_instructions: 2,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        assert!(
            Program::parse_with_limits(
                &bytes,
                0,
                Limits {
                    max_bytes: 1,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        let pool = action(0x88, &[2, 0, b'a', b'b', 0, b'c', 0]);
        assert!(
            Program::parse_with_limits(
                &pool,
                0,
                Limits {
                    max_values: 1,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        assert!(
            Program::parse_with_limits(
                &pool,
                0,
                Limits {
                    max_string_bytes: 1,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        assert!(
            Program::parse_with_limits(
                &pool,
                0,
                Limits {
                    max_total_string_bytes: 2,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        assert!(
            Program::parse_with_limits(
                &[],
                0,
                Limits {
                    max_nesting: 257,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        assert!(
            Program::parse_with_limits(
                &action(0x96, &[0, 0]),
                0,
                Limits {
                    max_string_bytes: 0,
                    ..Limits::default()
                }
            )
            .is_ok()
        );
    }

    #[test]
    fn fixed_operands_and_optional_scene_bias() {
        let cases = [
            (0x81, vec![2, 0]),
            (0x83, vec![b'u', 0, b't', 0]),
            (0x87, vec![3]),
            (0x8a, vec![2, 0, 4]),
            (0x8b, vec![b't', 0]),
            (0x8c, vec![b'l', 0]),
            (0x8d, vec![7]),
            (0x9a, vec![0xc1]),
            (0x9e, vec![]),
            (0x9f, vec![1]),
            (0x9f, vec![3, 9, 0]),
        ];
        for (opcode, payload) in cases {
            let p = Program::parse(&action(opcode, &payload)).unwrap();
            assert_eq!(p.instructions.len(), 1);
            assert_eq!(p.instructions[0].payload.len, payload.len());
        }
    }
}
