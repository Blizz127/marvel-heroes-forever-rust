//! Deliberately partial AVM1 executor. Every unsupported operation fails explicitly.
//! No file, network, native-client, or URL side effects exist in this module.
use super::{ByteString, Function, Literal, Operands, Program};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

pub type Text = Arc<[u16]>;
pub fn text(value: &str) -> Text {
    value.encode_utf16().collect::<Vec<_>>().into()
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeId(pub usize);
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(Text),
    Object(ObjectId),
}
impl Value {
    pub fn string(value: &str) -> Self {
        Self::String(text(value))
    }
    pub fn truthy(&self) -> bool {
        match self {
            Self::Undefined | Self::Null => false,
            Self::Bool(v) => *v,
            Self::Number(v) => *v != 0.0 && !v.is_nan(),
            Self::String(v) => !v.is_empty(),
            Self::Object(_) => true,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Limits {
    pub instructions: usize,
    pub call_depth: usize,
    pub stack_values: usize,
    pub objects: usize,
    pub properties: usize,
    pub string_units: usize,
    pub programs: usize,
    pub code_bytes: usize,
    pub host_calls: usize,
    pub arguments: usize,
    pub array_length: u32,
    pub prototype_depth: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            instructions: 250_000,
            call_depth: 64,
            stack_values: 16_384,
            objects: 100_000,
            properties: 1_000_000,
            string_units: 8_000_000,
            programs: 20_000,
            code_bytes: 64 * 1024 * 1024,
            host_calls: 10_000,
            arguments: 4096,
            array_length: 1_000_000,
            prototype_depth: 256,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    Budget(&'static str),
    Unsupported(&'static str),
    UnsupportedOpcode(u8),
    StackUnderflow,
    InvalidObject(ObjectId),
    InvalidCode(CodeId),
    InvalidRegister(u8),
    InvalidBranch(i128),
    InvalidConstant(u16),
    InvalidArgumentCount,
    InvalidUtf8,
    UnboundCall,
    Host(String),
    InvalidArrayLength,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub source_offset: Option<usize>,
    pub opcode: Option<u8>,
    pub kind: ErrorKind,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AVM1 {:?} at {:?} (opcode {:?})",
            self.kind, self.source_offset, self.opcode
        )
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum HostCommand {
    NextFrame,
    PreviousFrame,
    Play,
    Stop,
    ToggleQuality,
    StopSounds,
    GotoFrame(u16),
    GotoLabel(Text),
    GotoFrame2 {
        frame: Value,
        flags: u8,
        scene_bias: Option<u16>,
    },
    Trace(Value),
    GetUrl {
        url: Text,
        target: Text,
    },
    GetUrl2 {
        url: Value,
        target: Value,
        flags: u8,
    },
}
#[derive(Debug, Clone)]
pub struct HostCall<'a> {
    pub binding: u32,
    pub this: Value,
    pub args: &'a [Value],
    pub source_offset: Option<usize>,
}
pub trait Host {
    fn call(&mut self, call: HostCall<'_>) -> std::result::Result<Value, String>;
    fn command(
        &mut self,
        target: ObjectId,
        command: HostCommand,
    ) -> std::result::Result<(), String>;
}
#[derive(Debug, Default)]
pub struct RejectHost;
impl Host for RejectHost {
    fn call(&mut self, call: HostCall<'_>) -> std::result::Result<Value, String> {
        Err(format!("unhandled native binding {}", call.binding))
    }
    fn command(&mut self, _: ObjectId, command: HostCommand) -> std::result::Result<(), String> {
        Err(format!("unhandled host command {command:?}"))
    }
}
#[derive(Debug, Clone)]
pub struct Execution {
    pub value: Value,
    /// Remaining root operand stack, useful for explicit isolated-code probes.
    pub stack: Vec<Value>,
    pub instructions: usize,
    pub host_calls: usize,
}
#[derive(Debug, Clone)]
struct ScriptFunction {
    target: ObjectId,
    code: CodeId,
    offset: usize,
    scopes: Vec<ObjectId>,
    pool: Arc<Vec<Value>>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionMethod {
    Call,
    Apply,
}
#[derive(Debug, Clone)]
enum Kind {
    Plain,
    Array(u32),
    Script(ScriptFunction),
    Native(u32),
    ObjectConstructor,
    AddProperty,
    FunctionMethod(FunctionMethod),
}
#[derive(Debug, Clone)]
enum Property {
    Data(Value),
    Accessor {
        getter: Value,
        setter: Option<Value>,
    },
}
#[derive(Debug, Clone)]
struct Object {
    kind: Kind,
    properties: BTreeMap<Text, Property>,
    prototype: Option<ObjectId>,
}
struct Frame {
    stack: Vec<Value>,
    registers: Vec<Value>,
    scopes: Vec<ObjectId>,
    pool: Arc<Vec<Value>>,
    target: ObjectId,
    this: Value,
    this_visible: bool,
    locals: ObjectId,
}
enum Flow {
    End,
    Return(Value),
    Jump(usize),
}

pub struct Vm {
    limits: Limits,
    code: Vec<Arc<Program>>,
    code_bytes: usize,
    objects: Vec<Object>,
    properties: usize,
    string_units: usize,
    instructions: usize,
    host_calls: usize,
    depth: usize,
    current: Option<(usize, u8)>,
    global: ObjectId,
    root: ObjectId,
    object_constructor: Option<ObjectId>,
}
impl Vm {
    /// This execution profile supports SWF 7+; legacy coercions are not guessed.
    pub fn new(version: u8, limits: Limits) -> Result<Self> {
        let mut vm = Self {
            limits,
            code: Vec::new(),
            code_bytes: 0,
            objects: Vec::new(),
            properties: 0,
            string_units: 0,
            instructions: 0,
            host_calls: 0,
            depth: 0,
            current: None,
            global: ObjectId(0),
            root: ObjectId(0),
            object_constructor: None,
        };
        if !(7..=19).contains(&version) {
            return vm.fail(ErrorKind::Unsupported(
                "SWF execution version outside 7..19",
            ));
        }
        if vm.limits.call_depth > 128 {
            return vm.fail(ErrorKind::Budget("call-depth hard maximum 128"));
        }
        vm.global = vm.object()?;
        vm.root = vm.object()?;
        vm.set(vm.global, "_global", Value::Object(vm.global))?;
        vm.set(vm.root, "_root", Value::Object(vm.root))?;
        vm.set(vm.global, "undefined", Value::Undefined)?;
        vm.set(vm.global, "NaN", Value::Number(f64::NAN))?;
        vm.set(vm.global, "Infinity", Value::Number(f64::INFINITY))?;
        Ok(vm)
    }
    pub fn global(&self) -> ObjectId {
        self.global
    }
    pub fn root(&self) -> ObjectId {
        self.root
    }
    pub fn instruction_count(&self) -> usize {
        self.instructions
    }
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }
    fn fail<T>(&self, kind: ErrorKind) -> Result<T> {
        Err(Error {
            source_offset: self.current.map(|v| v.0),
            opcode: self.current.map(|v| v.1),
            kind,
        })
    }
    fn error(&self, kind: ErrorKind) -> Error {
        Error {
            source_offset: self.current.map(|v| v.0),
            opcode: self.current.map(|v| v.1),
            kind,
        }
    }
    fn alloc(&mut self, kind: Kind, prototype: Option<ObjectId>) -> Result<ObjectId> {
        if self.objects.len() >= self.limits.objects {
            return self.fail(ErrorKind::Budget("objects"));
        }
        if let Some(p) = prototype {
            self.check_object(p)?;
        }
        let id = ObjectId(self.objects.len());
        self.objects.push(Object {
            kind,
            properties: BTreeMap::new(),
            prototype,
        });
        Ok(id)
    }
    pub fn object(&mut self) -> Result<ObjectId> {
        self.alloc(Kind::Plain, None)
    }
    pub fn native(&mut self, binding: u32) -> Result<ObjectId> {
        self.alloc(Kind::Native(binding), None)
    }
    /// Create a standard method value for an explicitly installed function
    /// prototype. Does not fabricate a global Function constructor/cast binding.
    pub fn function_method(&mut self, method: FunctionMethod) -> Result<ObjectId> {
        self.alloc(Kind::FunctionMethod(method), None)
    }

    /// Install only the documented Object constructor/prototype. Optional native
    /// methods and unrelated namespace objects are not fabricated.
    pub fn install_object_constructor(&mut self) -> Result<ObjectId> {
        if let Some(id) = self.object_constructor {
            return Ok(id);
        }
        if self.has_key(self.global, &text("Object"))? {
            return self.fail(ErrorKind::Unsupported("Object binding already exists"));
        }
        let constructor = self.alloc(Kind::ObjectConstructor, None)?;
        let prototype = self.object()?;
        self.set(constructor, "prototype", Value::Object(prototype))?;
        self.set(prototype, "constructor", Value::Object(constructor))?;
        let add_property = self.alloc(Kind::AddProperty, None)?;
        self.set(prototype, "addProperty", Value::Object(add_property))?;
        self.set(self.global, "Object", Value::Object(constructor))?;
        self.object_constructor = Some(constructor);
        Ok(constructor)
    }
    fn ordinary_object(&mut self) -> Result<ObjectId> {
        let prototype = if let Some(constructor) = self.object_constructor {
            match self.get(constructor, "prototype")? {
                Value::Object(id) => Some(id),
                Value::Null => None,
                _ => return self.fail(ErrorKind::Unsupported("invalid Object prototype")),
            }
        } else {
            None
        };
        self.alloc(Kind::Plain, prototype)
    }

    pub fn register(&mut self, program: Arc<Program>) -> Result<CodeId> {
        if self.code.len() >= self.limits.programs {
            return self.fail(ErrorKind::Budget("registered programs"));
        }
        if program.span.len > self.limits.code_bytes.saturating_sub(self.code_bytes) {
            return self.fail(ErrorKind::Budget("registered code bytes"));
        }
        self.code_bytes += program.span.len;
        let id = CodeId(self.code.len());
        self.code.push(program);
        Ok(id)
    }
    fn check_object(&self, id: ObjectId) -> Result<()> {
        if id.0 >= self.objects.len() {
            self.fail(ErrorKind::InvalidObject(id))
        } else {
            Ok(())
        }
    }
    fn check_value(&self, value: &Value) -> Result<()> {
        if let Value::Object(id) = value {
            self.check_object(*id)
        } else {
            Ok(())
        }
    }
    fn allocate_text(&mut self, units: Vec<u16>) -> Result<Text> {
        if units.len() > self.limits.string_units.saturating_sub(self.string_units) {
            return self.fail(ErrorKind::Budget("allocated string units"));
        }
        self.string_units += units.len();
        Ok(units.into())
    }
    fn decode_text(&mut self, value: &ByteString) -> Result<Text> {
        let value = value
            .utf8()
            .map_err(|_| self.error(ErrorKind::InvalidUtf8))?;
        if value.encode_utf16().count() > self.limits.string_units.saturating_sub(self.string_units)
        {
            return self.fail(ErrorKind::Budget("allocated string units"));
        }
        self.allocate_text(value.encode_utf16().collect())
    }
    pub fn get(&self, id: ObjectId, key: &str) -> Result<Value> {
        self.get_key(id, &text(key))
    }
    pub fn set(&mut self, id: ObjectId, key: &str, value: Value) -> Result<()> {
        self.set_key(id, text(key), value)
    }
    fn get_key(&self, mut id: ObjectId, key: &Text) -> Result<Value> {
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            let object = &self.objects[id.0];
            if key.as_ref() == text("__proto__").as_ref() {
                return Ok(object.prototype.map_or(Value::Null, Value::Object));
            }
            if let Kind::Array(len) = object.kind {
                if key.as_ref() == text("length").as_ref() {
                    return Ok(Value::Number(f64::from(len)));
                }
            }
            if let Some(property) = object.properties.get(key) {
                return match property {
                    Property::Data(value) => Ok(value.clone()),
                    Property::Accessor { .. } => self.fail(ErrorKind::Unsupported(
                        "accessor requires executable property access",
                    )),
                };
            }
            if let Some(next) = object.prototype {
                id = next;
            } else {
                return Ok(Value::Undefined);
            }
        }
        self.fail(ErrorKind::Budget("prototype lookup depth"))
    }
    fn has_key(&self, mut id: ObjectId, key: &Text) -> Result<bool> {
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            let object = &self.objects[id.0];
            if object.properties.contains_key(key)
                || key.as_ref() == text("__proto__").as_ref()
                || (matches!(object.kind, Kind::Array(_))
                    && key.as_ref() == text("length").as_ref())
            {
                return Ok(true);
            }
            if let Some(next) = object.prototype {
                id = next;
            } else {
                return Ok(false);
            }
        }
        self.fail(ErrorKind::Budget("prototype lookup depth"))
    }
    fn descriptor(&self, mut id: ObjectId, key: &Text) -> Result<Option<Property>> {
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            let object = &self.objects[id.0];
            if let Some(property) = object.properties.get(key) {
                return Ok(Some(property.clone()));
            }
            if let Some(next) = object.prototype {
                id = next;
            } else {
                return Ok(None);
            }
        }
        self.fail(ErrorKind::Budget("prototype lookup depth"))
    }
    fn callable(&self, value: &Value) -> bool {
        if let Value::Object(id) = value {
            self.objects.get(id.0).is_some_and(|object| {
                matches!(
                    object.kind,
                    Kind::Script(_)
                        | Kind::Native(_)
                        | Kind::ObjectConstructor
                        | Kind::AddProperty
                        | Kind::FunctionMethod(_)
                )
            })
        } else {
            false
        }
    }
    fn read_member(&mut self, id: ObjectId, key: &Text, host: &mut impl Host) -> Result<Value> {
        if let Some(Property::Accessor { getter, .. }) = self.descriptor(id, key)? {
            self.invoke(getter, Value::Object(id), &[], host)
        } else {
            self.get_key(id, key)
        }
    }
    fn write_member(
        &mut self,
        id: ObjectId,
        key: Text,
        value: Value,
        host: &mut impl Host,
    ) -> Result<()> {
        self.check_value(&value)?;
        if let Some(Property::Accessor { setter, .. }) = self.descriptor(id, &key)? {
            if let Some(setter) = setter {
                self.invoke(setter, Value::Object(id), &[value], host)?;
            }
            // Adobe explicitly specifies ignored writes for a null setter.
            Ok(())
        } else {
            self.set_key(id, key, value)
        }
    }
    pub fn read_property(
        &mut self,
        id: ObjectId,
        key: &str,
        host: &mut impl Host,
    ) -> Result<Execution> {
        self.reset_execution();
        let value = self.read_member(id, &text(key), host)?;
        Ok(Execution {
            value,
            stack: Vec::new(),
            instructions: self.instructions,
            host_calls: self.host_calls,
        })
    }
    pub fn write_property(
        &mut self,
        id: ObjectId,
        key: &str,
        value: Value,
        host: &mut impl Host,
    ) -> Result<Execution> {
        self.reset_execution();
        self.write_member(id, text(key), value, host)?;
        Ok(Execution {
            value: Value::Undefined,
            stack: Vec::new(),
            instructions: self.instructions,
            host_calls: self.host_calls,
        })
    }
    fn reserve_property(&mut self, id: ObjectId, key: &Text) -> Result<()> {
        if !self.objects[id.0].properties.contains_key(key) {
            if self.properties >= self.limits.properties {
                return self.fail(ErrorKind::Budget("properties"));
            }
            if key.len() > self.limits.string_units.saturating_sub(self.string_units) {
                return self.fail(ErrorKind::Budget("property name units"));
            }
            self.properties += 1;
            self.string_units += key.len();
        }
        Ok(())
    }
    fn set_key(&mut self, id: ObjectId, key: Text, value: Value) -> Result<()> {
        self.check_object(id)?;
        self.check_value(&value)?;
        if matches!(self.descriptor(id, &key)?, Some(Property::Accessor { .. })) {
            return self.fail(ErrorKind::Unsupported(
                "accessor requires executable property assignment",
            ));
        }
        if key.as_ref() == text("__proto__").as_ref() {
            let prototype = match value {
                Value::Null => None,
                Value::Object(p) => Some(p),
                _ => return self.fail(ErrorKind::Unsupported("primitive prototype assignment")),
            };
            let mut cursor = prototype;
            let mut traversed = 0;
            while let Some(p) = cursor {
                if traversed >= self.limits.prototype_depth {
                    return self.fail(ErrorKind::Budget("prototype lookup depth"));
                }
                traversed += 1;
                if p == id {
                    return self.fail(ErrorKind::Unsupported("cyclic prototype assignment"));
                }
                cursor = self.objects[p.0].prototype;
            }
            self.objects[id.0].prototype = prototype;
            return Ok(());
        }
        if matches!(self.objects[id.0].kind, Kind::Array(_)) {
            if key.as_ref() == text("length").as_ref() {
                let number = self.number(&value)?;
                let length = uint32(number);
                if f64::from(length) != number || length > self.limits.array_length {
                    return self.fail(ErrorKind::InvalidArrayLength);
                }
                self.objects[id.0]
                    .properties
                    .retain(|k, _| array_index(k).is_none_or(|i| i < length));
                self.objects[id.0].kind = Kind::Array(length);
                return Ok(());
            }
            if let Some(index) = array_index(&key) {
                if index >= self.limits.array_length {
                    return self.fail(ErrorKind::Budget("array length"));
                }
                if let Kind::Array(ref mut len) = self.objects[id.0].kind {
                    *len = (*len).max(index + 1);
                }
            }
        }
        self.reserve_property(id, &key)?;
        self.objects[id.0]
            .properties
            .insert(key, Property::Data(value));
        Ok(())
    }
    fn push(&self, frame: &mut Frame, value: Value) -> Result<()> {
        if frame.stack.len() >= self.limits.stack_values {
            return self.fail(ErrorKind::Budget("operand stack"));
        }
        frame.stack.push(value);
        Ok(())
    }
    fn pop(&self, frame: &mut Frame) -> Result<Value> {
        frame
            .stack
            .pop()
            .ok_or_else(|| self.error(ErrorKind::StackUnderflow))
    }
    fn object_id(&self, value: Value) -> Result<ObjectId> {
        if let Value::Object(id) = value {
            self.check_object(id)?;
            Ok(id)
        } else {
            self.fail(ErrorKind::Unsupported(
                "primitive property receiver or missing object binding",
            ))
        }
    }
    fn number(&self, value: &Value) -> Result<f64> {
        match value {
            Value::Number(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1.0 } else { 0.0 }),
            Value::Undefined => Ok(f64::NAN),
            Value::Null => self.fail(ErrorKind::Unsupported(
                "unverified AVM1 null-to-number conversion",
            )),
            Value::Object(_) => self.fail(ErrorKind::Unsupported(
                "object primitive coercion requires valueOf/toString",
            )),
            Value::String(s) => {
                if s.is_empty() || s.iter().any(|u| *u > 127) {
                    return self.fail(ErrorKind::Unsupported(
                        "empty/non-ASCII numeric string conversion",
                    ));
                }
                let s = String::from_utf16(s).map_err(|_| self.error(ErrorKind::InvalidUtf8))?;
                if s.trim() != s {
                    return self.fail(ErrorKind::Unsupported(
                        "whitespace numeric string conversion",
                    ));
                }
                if s == "Infinity" || s == "+Infinity" {
                    return Ok(f64::INFINITY);
                }
                if s == "-Infinity" {
                    return Ok(f64::NEG_INFINITY);
                }
                if s.starts_with("0x") || s.starts_with("0X") {
                    return self.fail(ErrorKind::Unsupported(
                        "hexadecimal numeric string conversion",
                    ));
                }
                if !decimal(&s) {
                    return Ok(f64::NAN);
                }
                Ok(s.parse::<f64>().unwrap_or(f64::NAN))
            }
        }
    }
    fn to_text(&mut self, value: &Value) -> Result<Text> {
        match value {
            Value::String(s) => Ok(s.clone()),
            Value::Undefined => Ok(text("undefined")),
            Value::Null => Ok(text("null")),
            Value::Bool(v) => Ok(text(if *v { "true" } else { "false" })),
            Value::Number(v) => {
                if v.is_nan() {
                    return Ok(text("NaN"));
                }
                if *v == 0.0 {
                    return Ok(text("0"));
                }
                if v.is_infinite() {
                    return Ok(text(if v.is_sign_negative() {
                        "-Infinity"
                    } else {
                        "Infinity"
                    }));
                }
                // Exact integral decimal conversion only; general decimal formatting awaits native evidence.
                if v.fract() != 0.0 || v.abs() > 9_007_199_254_740_991.0 {
                    return self.fail(ErrorKind::Unsupported(
                        "general number-to-string formatting",
                    ));
                }
                self.allocate_text(format!("{v:.0}").encode_utf16().collect())
            }
            Value::Object(_) => self.fail(ErrorKind::Unsupported(
                "object primitive coercion requires valueOf/toString",
            )),
        }
    }
    fn variable(&mut self, frame: &Frame, key: &Text, host: &mut impl Host) -> Result<Value> {
        if frame.this_visible && key.as_ref() == text("this").as_ref() {
            return Ok(frame.this.clone());
        }
        if key.as_ref() == text("_global").as_ref() {
            return Ok(Value::Object(self.global));
        }
        if key.as_ref() == text("_root").as_ref() {
            return Ok(Value::Object(self.root));
        }
        self.simple_name(key)?;
        if key.as_ref() == text("super").as_ref() {
            return self.fail(ErrorKind::Unsupported("super lookup"));
        }
        for id in &frame.scopes {
            if self.has_key(*id, key)? {
                return self.read_member(*id, key, host);
            }
        }
        Ok(Value::Undefined)
    }
    fn simple_name(&self, key: &Text) -> Result<()> {
        if key.iter().any(|v| matches!(*v, 46 | 47 | 58)) {
            self.fail(ErrorKind::Unsupported(
                "qualified variable or movieclip path",
            ))
        } else {
            Ok(())
        }
    }
    fn set_variable(
        &mut self,
        frame: &Frame,
        key: Text,
        value: Value,
        local: bool,
        host: &mut impl Host,
    ) -> Result<()> {
        self.simple_name(&key)?;
        let id = if local {
            frame.locals
        } else {
            let mut found = frame.target;
            for id in &frame.scopes {
                if self.has_key(*id, &key)? {
                    found = *id;
                    break;
                }
            }
            found
        };
        if local {
            self.set_key(id, key, value)
        } else {
            self.write_member(id, key, value, host)
        }
    }
    fn reset_execution(&mut self) {
        self.instructions = 0;
        self.host_calls = 0;
        self.depth = 0;
        self.current = None;
    }
    pub fn execute(
        &mut self,
        code: CodeId,
        target: ObjectId,
        host: &mut impl Host,
    ) -> Result<Execution> {
        self.reset_execution();
        self.check_object(target)?;
        let program = self
            .code
            .get(code.0)
            .cloned()
            .ok_or_else(|| self.error(ErrorKind::InvalidCode(code)))?;
        let mut frame = Frame {
            stack: Vec::new(),
            registers: vec![Value::Undefined; 4],
            scopes: vec![target, self.global],
            pool: Arc::new(Vec::new()),
            target,
            this: Value::Object(target),
            this_visible: true,
            locals: target,
        };
        let value = match self.run(code, &program, &mut frame, host)? {
            Flow::Return(v) => v,
            Flow::End => Value::Undefined,
            Flow::Jump(p) => return self.fail(ErrorKind::InvalidBranch(p as i128)),
        };
        Ok(Execution {
            value,
            stack: frame.stack,
            instructions: self.instructions,
            host_calls: self.host_calls,
        })
    }
    pub fn call(
        &mut self,
        callable: Value,
        this: Value,
        args: &[Value],
        host: &mut impl Host,
    ) -> Result<Execution> {
        self.reset_execution();
        let value = self.invoke(callable, this, args, host)?;
        Ok(Execution {
            value,
            stack: Vec::new(),
            instructions: self.instructions,
            host_calls: self.host_calls,
        })
    }
    fn host_budget(&mut self) -> Result<()> {
        if self.host_calls >= self.limits.host_calls {
            return self.fail(ErrorKind::Budget("host calls"));
        }
        self.host_calls += 1;
        Ok(())
    }
    fn command(
        &mut self,
        target: ObjectId,
        command: HostCommand,
        host: &mut impl Host,
    ) -> Result<()> {
        self.host_budget()?;
        host.command(target, command)
            .map_err(|s| self.error(ErrorKind::Host(s)))
    }
    fn invoke(
        &mut self,
        callable: Value,
        this: Value,
        args: &[Value],
        host: &mut impl Host,
    ) -> Result<Value> {
        if self.depth >= self.limits.call_depth {
            return self.fail(ErrorKind::Budget("function call depth"));
        }
        self.depth += 1;
        let result = self.invoke_inner(callable, this, args, host);
        self.depth -= 1;
        result
    }
    fn invoke_inner(
        &mut self,
        callable: Value,
        this: Value,
        args: &[Value],
        host: &mut impl Host,
    ) -> Result<Value> {
        if args.len() > self.limits.arguments {
            return self.fail(ErrorKind::Budget("arguments"));
        }
        self.check_value(&this)?;
        for arg in args {
            self.check_value(arg)?;
        }
        let Value::Object(id) = callable else {
            return self.fail(ErrorKind::UnboundCall);
        };
        self.check_object(id)?;
        let function = match self.objects[id.0].kind.clone() {
            Kind::Native(binding) => {
                self.host_budget()?;
                let value = host
                    .call(HostCall {
                        binding,
                        this,
                        args,
                        source_offset: self.current.map(|v| v.0),
                    })
                    .map_err(|s| self.error(ErrorKind::Host(s)))?;
                self.check_value(&value)?;
                return Ok(value);
            }
            Kind::Script(function) => function,
            Kind::FunctionMethod(method) => {
                if !self.callable(&this) {
                    return self.fail(ErrorKind::UnboundCall);
                }
                let Some(Value::Object(receiver)) = args.first() else {
                    return self.fail(ErrorKind::Unsupported(
                        "Function method null/primitive thisObject",
                    ));
                };
                let forwarded = match method {
                    FunctionMethod::Call => args[1..].to_vec(),
                    FunctionMethod::Apply => {
                        if args.len() > 2 {
                            return self
                                .fail(ErrorKind::Unsupported("Function.apply extra arguments"));
                        }
                        match args.get(1) {
                            None => Vec::new(),
                            Some(Value::Object(array)) => {
                                let Kind::Array(length) = self.objects[array.0].kind else {
                                    return self.fail(ErrorKind::Unsupported(
                                        "Function.apply non-Array arguments",
                                    ));
                                };
                                if length as usize > self.limits.arguments {
                                    return self.fail(ErrorKind::Budget("arguments"));
                                }
                                let mut forwarded = Vec::new();
                                for i in 0..length {
                                    forwarded.push(self.read_member(
                                        *array,
                                        &text(&i.to_string()),
                                        host,
                                    )?);
                                }
                                forwarded
                            }
                            _ => {
                                return self.fail(ErrorKind::Unsupported(
                                    "Function.apply null/undefined argument-array profile",
                                ));
                            }
                        }
                    }
                };
                return self.invoke(this, Value::Object(*receiver), &forwarded, host);
            }
            Kind::AddProperty => {
                if args.len() != 3 {
                    return self.fail(ErrorKind::Unsupported("addProperty argument count"));
                }
                let (Value::Object(receiver), Value::String(name)) = (&this, &args[0]) else {
                    return Ok(Value::Bool(false));
                };
                if name.is_empty()
                    || !self.callable(&args[1])
                    || (!matches!(args[2], Value::Null) && !self.callable(&args[2]))
                {
                    return Ok(Value::Bool(false));
                }
                if name.as_ref() == text("__proto__").as_ref()
                    || matches!(self.objects[receiver.0].kind, Kind::Array(_))
                        && name.as_ref() == text("length").as_ref()
                {
                    return self.fail(ErrorKind::Unsupported("accessor over intrinsic property"));
                }
                self.reserve_property(*receiver, name)?;
                let setter = if matches!(args[2], Value::Null) {
                    None
                } else {
                    Some(args[2].clone())
                };
                self.objects[receiver.0].properties.insert(
                    name.clone(),
                    Property::Accessor {
                        getter: args[1].clone(),
                        setter,
                    },
                );
                return Ok(Value::Bool(true));
            }
            Kind::ObjectConstructor => {
                if args.len() > 1 {
                    return self.fail(ErrorKind::Unsupported("Object constructor extra arguments"));
                }
                return match args.first() {
                    Some(Value::Object(object)) => Ok(Value::Object(*object)),
                    None | Some(Value::Null) | Some(Value::Undefined) => {
                        Ok(Value::Object(self.ordinary_object()?))
                    }
                    _ => self.fail(ErrorKind::Unsupported(
                        "Object primitive wrapper construction",
                    )),
                };
            }
            _ => return self.fail(ErrorKind::UnboundCall),
        };
        let program = self.code[function.code.0].clone();
        let Some(instruction) = program.instruction_at(function.offset) else {
            return self.fail(ErrorKind::InvalidCode(function.code));
        };
        let Operands::Function(definition) = &instruction.operands else {
            return self.fail(ErrorKind::InvalidCode(function.code));
        };
        let flags = definition.flags.unwrap_or(0);
        if flags & 3 == 3 || flags & 12 == 12 || flags & 0x30 == 0x30 {
            return self.fail(ErrorKind::Unsupported(
                "conflicting function preload/suppress flags",
            ));
        }
        // Super and parent need prototype/movie hierarchy behavior not yet provided.
        if flags & (0x10 | 0x80) != 0 {
            return self.fail(ErrorKind::Unsupported("super/parent register preload"));
        }
        if flags & 0xfe00 != 0 {
            return self.fail(ErrorKind::Unsupported("reserved Function2 flags"));
        }
        let activation = self.object()?;
        let target = function.target;
        let mut scopes = vec![activation];
        scopes.extend(function.scopes.iter().copied());
        let mut frame = Frame {
            stack: Vec::new(),
            registers: vec![
                Value::Undefined;
                definition.register_count.map_or(4, |n| usize::from(n) + 1)
            ],
            scopes,
            pool: function.pool.clone(),
            target,
            this: this.clone(),
            this_visible: flags & 2 == 0,
            locals: activation,
        };
        for (index, param) in definition.parameters.iter().enumerate() {
            let value = args.get(index).cloned().unwrap_or(Value::Undefined);
            if let Some(register) = param.register.filter(|r| *r != 0) {
                self.write_register(&mut frame, register, value)?;
            } else {
                let key = self.decode_text(&param.name)?;
                self.set_key(activation, key, value)?;
            }
        }
        let arguments = if flags & 8 == 0 || flags & 4 != 0 {
            let array = self.array(args)?;
            self.set(array, "callee", Value::Object(id))?;
            if flags & 8 == 0 {
                self.set(activation, "arguments", Value::Object(array))?;
            }
            Some(Value::Object(array))
        } else {
            None
        };
        if flags & 2 == 0 {
            self.set(activation, "this", this.clone())?;
        }
        let mut register = 1u8;
        for (mask, value) in [
            (1, this),
            (4, arguments.unwrap_or(Value::Undefined)),
            (0x40, Value::Object(self.root)),
            (0x100, Value::Object(self.global)),
        ] {
            if flags & mask != 0 {
                self.write_register(&mut frame, register, value)?;
                register += 1;
            }
        }
        let result = self.run(function.code, &definition.body, &mut frame, host);
        match result? {
            Flow::Return(v) => Ok(v),
            Flow::End => Ok(Value::Undefined),
            Flow::Jump(p) => self.fail(ErrorKind::InvalidBranch(p as i128)),
        }
    }
    fn write_register(&self, frame: &mut Frame, index: u8, value: Value) -> Result<()> {
        let slot = frame
            .registers
            .get_mut(usize::from(index))
            .ok_or_else(|| self.error(ErrorKind::InvalidRegister(index)))?;
        *slot = value;
        Ok(())
    }
    fn array(&mut self, values: &[Value]) -> Result<ObjectId> {
        let len = u32::try_from(values.len())
            .map_err(|_| self.error(ErrorKind::Budget("array length")))?;
        if len > self.limits.array_length {
            return self.fail(ErrorKind::Budget("array length"));
        }
        let id = self.alloc(Kind::Array(len), None)?;
        for (i, value) in values.iter().enumerate() {
            self.set(id, &i.to_string(), value.clone())?;
        }
        Ok(id)
    }
    fn args(&self, frame: &mut Frame) -> Result<Vec<Value>> {
        let count = self.number(&self.pop(frame)?)?;
        if !count.is_finite() || count < 0.0 || count.fract() != 0.0 {
            return self.fail(ErrorKind::InvalidArgumentCount);
        }
        if count > self.limits.arguments as f64 {
            return self.fail(ErrorKind::Budget("arguments"));
        }
        let count = count as usize;
        if count > frame.stack.len() {
            return self.fail(ErrorKind::StackUnderflow);
        }
        let mut args = Vec::with_capacity(count);
        for _ in 0..count {
            args.push(self.pop(frame)?);
        }
        Ok(args)
    }
    fn define(
        &mut self,
        code: CodeId,
        offset: usize,
        definition: &Function,
        frame: &mut Frame,
        host: &mut impl Host,
    ) -> Result<()> {
        let function = self.alloc(
            Kind::Script(ScriptFunction {
                target: frame.target,
                code,
                offset,
                scopes: frame.scopes.clone(),
                pool: frame.pool.clone(),
            }),
            None,
        )?;
        let prototype = self.ordinary_object()?;
        self.set(function, "prototype", Value::Object(prototype))?;
        self.set(prototype, "constructor", Value::Object(function))?;
        if definition.name.0.is_empty() {
            self.push(frame, Value::Object(function))
        } else {
            let name = self.decode_text(&definition.name)?;
            self.set_variable(frame, name, Value::Object(function), true, host)
        }
    }
    fn strict_equal(&self, a: &Value, b: &Value) -> bool {
        a == b
    }
    fn equal(&self, a: &Value, b: &Value) -> Result<bool> {
        if std::mem::discriminant(a) == std::mem::discriminant(b) {
            return Ok(self.strict_equal(a, b));
        }
        if matches!(
            (a, b),
            (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null)
        ) {
            return Ok(true);
        }
        if matches!(a, Value::Object(_)) || matches!(b, Value::Object(_)) {
            return self.fail(ErrorKind::Unsupported("object abstract equality coercion"));
        }
        if matches!(a, Value::Null | Value::Undefined)
            || matches!(b, Value::Null | Value::Undefined)
        {
            return Ok(false);
        }
        Ok(self.number(a)? == self.number(b)?)
    }
    fn run(
        &mut self,
        code: CodeId,
        program: &Program,
        frame: &mut Frame,
        host: &mut impl Host,
    ) -> Result<Flow> {
        let mut pc = 0;
        while let Some(instruction) = program.instructions.get(pc) {
            self.current = Some((instruction.span.offset, instruction.opcode));
            if self.instructions >= self.limits.instructions {
                return self.fail(ErrorKind::Budget("instructions"));
            }
            self.instructions += 1;
            let mut jump = None;
            match &instruction.operands {
                Operands::Push(values) => {
                    for literal in values {
                        let value = match literal {
                            Literal::String(s) => Value::String(self.decode_text(s)?),
                            Literal::FloatBits(v) => Value::Number(f64::from(f32::from_bits(*v))),
                            Literal::DoubleBits(v) => Value::Number(f64::from_bits(*v)),
                            Literal::Integer(v) => Value::Number(f64::from(*v as i32)),
                            Literal::Null => Value::Null,
                            Literal::Undefined => Value::Undefined,
                            Literal::Boolean(v) => Value::Bool(*v != 0),
                            Literal::Register(r) => {
                                frame
                                    .registers
                                    .get(usize::from(*r))
                                    .cloned()
                                    .ok_or_else(|| self.error(ErrorKind::InvalidRegister(*r)))?
                            }
                            Literal::Constant8(i) => {
                                frame.pool.get(usize::from(*i)).cloned().ok_or_else(|| {
                                    self.error(ErrorKind::InvalidConstant(u16::from(*i)))
                                })?
                            }
                            Literal::Constant16(i) => frame
                                .pool
                                .get(usize::from(*i))
                                .cloned()
                                .ok_or_else(|| self.error(ErrorKind::InvalidConstant(*i)))?,
                            Literal::Unknown { .. } => {
                                return self.fail(ErrorKind::Unsupported("unknown Push type"));
                            }
                        };
                        self.push(frame, value)?;
                    }
                }
                Operands::ConstantPool(pool) => {
                    let mut values = Vec::new();
                    for value in pool {
                        values.push(Value::String(self.decode_text(value)?));
                    }
                    frame.pool = Arc::new(values);
                }
                Operands::StoreRegister(r) => {
                    let value = frame
                        .stack
                        .last()
                        .cloned()
                        .ok_or_else(|| self.error(ErrorKind::StackUnderflow))?;
                    self.write_register(frame, *r, value)?;
                }
                Operands::Function(definition) => {
                    self.define(code, instruction.span.offset, definition, frame, host)?
                }
                Operands::Branch(branch) => {
                    if instruction.opcode == 0x99 || self.pop(frame)?.truthy() {
                        jump =
                            Some(usize::try_from(branch.target).map_err(|_| {
                                self.error(ErrorKind::InvalidBranch(branch.target))
                            })?);
                    }
                }
                Operands::With(body) => {
                    let object = self.object_id(self.pop(frame)?)?;
                    frame.scopes.insert(0, object);
                    if self.depth >= self.limits.call_depth {
                        return self.fail(ErrorKind::Budget("execution nesting"));
                    }
                    self.depth += 1;
                    let result = self.run(code, body, frame, host);
                    self.depth -= 1;
                    frame.scopes.remove(0);
                    match result? {
                        Flow::End => {}
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Jump(p) => jump = Some(p),
                    }
                }
                Operands::GotoFrame(frame_number) => {
                    self.command(frame.target, HostCommand::GotoFrame(*frame_number), host)?
                }
                Operands::GotoLabel(label) => {
                    let label = self.decode_text(label)?;
                    self.command(frame.target, HostCommand::GotoLabel(label), host)?;
                }
                Operands::GotoFrame2 { flags, scene_bias } => {
                    let value = self.pop(frame)?;
                    self.command(
                        frame.target,
                        HostCommand::GotoFrame2 {
                            frame: value,
                            flags: *flags,
                            scene_bias: *scene_bias,
                        },
                        host,
                    )?;
                }
                Operands::GetUrl { url, target } => {
                    let url = self.decode_text(url)?;
                    let target = self.decode_text(target)?;
                    self.command(frame.target, HostCommand::GetUrl { url, target }, host)?;
                }
                Operands::GetUrl2(flags) => {
                    let target = self.pop(frame)?;
                    let url = self.pop(frame)?;
                    self.command(
                        frame.target,
                        HostCommand::GetUrl2 {
                            url,
                            target,
                            flags: *flags,
                        },
                        host,
                    )?;
                }
                Operands::None => match instruction.opcode {
                    0 => return Ok(Flow::End),
                    4..=9 => {
                        let command = match instruction.opcode {
                            4 => HostCommand::NextFrame,
                            5 => HostCommand::PreviousFrame,
                            6 => HostCommand::Play,
                            7 => HostCommand::Stop,
                            8 => HostCommand::ToggleQuality,
                            _ => HostCommand::StopSounds,
                        };
                        self.command(frame.target, command, host)?;
                    }
                    0x17 => {
                        // Owned native opcode 0x17 restores the zero-depth stack
                        // sentinel and returns normally. Other pops stay checked.
                        frame.stack.pop();
                    }
                    0x4c => {
                        let value = frame
                            .stack
                            .last()
                            .cloned()
                            .ok_or_else(|| self.error(ErrorKind::StackUnderflow))?;
                        self.push(frame, value)?;
                    }
                    0x4d => {
                        let a = self.pop(frame)?;
                        let b = self.pop(frame)?;
                        self.push(frame, a)?;
                        self.push(frame, b)?;
                    }
                    0x12 => {
                        let value = !self.pop(frame)?.truthy();
                        self.push(frame, Value::Bool(value))?;
                    }
                    0x10 | 0x11 => {
                        let a = self.pop(frame)?.truthy();
                        let b = self.pop(frame)?.truthy();
                        self.push(
                            frame,
                            Value::Bool(if instruction.opcode == 0x10 {
                                a && b
                            } else {
                                a || b
                            }),
                        )?;
                    }
                    0x1c => {
                        let value = self.pop(frame)?;
                        let key = self.to_text(&value)?;
                        let value = self.variable(frame, &key, host)?;
                        self.push(frame, value)?;
                    }
                    0x1d | 0x3c => {
                        let value = self.pop(frame)?;
                        let key = self.pop(frame)?;
                        let key = self.to_text(&key)?;
                        self.set_variable(frame, key, value, instruction.opcode == 0x3c, host)?;
                    }
                    0x41 => {
                        let key = self.pop(frame)?;
                        let key = self.to_text(&key)?;
                        self.set_variable(frame, key, Value::Undefined, true, host)?;
                    }
                    0x4e => {
                        let key = self.pop(frame)?;
                        let key = self.to_text(&key)?;
                        let object = self.object_id(self.pop(frame)?)?;
                        let value = self.read_member(object, &key, host)?;
                        self.push(frame, value)?;
                    }
                    0x4f => {
                        let value = self.pop(frame)?;
                        let key = self.pop(frame)?;
                        let key = self.to_text(&key)?;
                        let object = self.object_id(self.pop(frame)?)?;
                        self.write_member(object, key, value, host)?;
                    }
                    0x42 => {
                        let args = self.args(frame)?;
                        let array = self.array(&args)?;
                        self.push(frame, Value::Object(array))?;
                    }
                    0x43 => {
                        let count = self.number(&self.pop(frame)?)?;
                        if !count.is_finite() || count < 0.0 || count.fract() != 0.0 {
                            return self.fail(ErrorKind::InvalidArgumentCount);
                        }
                        if count > self.limits.arguments as f64 {
                            return self.fail(ErrorKind::Budget("object literal properties"));
                        }
                        let object = self.ordinary_object()?;
                        for _ in 0..count as usize {
                            let value = self.pop(frame)?;
                            let key = self.pop(frame)?;
                            let key = self.to_text(&key)?;
                            self.set_key(object, key, value)?;
                        }
                        self.push(frame, Value::Object(object))?;
                    }
                    0x3d => {
                        let name = self.pop(frame)?;
                        let name = self.to_text(&name)?;
                        let args = self.args(frame)?;
                        let callable = self.variable(frame, &name, host)?;
                        let value =
                            self.invoke(callable, Value::Object(frame.target), &args, host)?;
                        self.push(frame, value)?;
                    }
                    0x52 => {
                        let method = self.pop(frame)?;
                        let receiver = self.pop(frame)?;
                        let args = self.args(frame)?;
                        let callable = if matches!(method, Value::Undefined)
                            || matches!(&method,Value::String(s) if s.is_empty())
                        {
                            receiver.clone()
                        } else {
                            let key = self.to_text(&method)?;
                            let object = self.object_id(receiver.clone())?;
                            self.read_member(object, &key, host)?
                        };
                        let value = self.invoke(callable, receiver, &args, host)?;
                        self.push(frame, value)?;
                    }
                    0x40 | 0x53 => {
                        let callable = if instruction.opcode == 0x40 {
                            let name = self.pop(frame)?;
                            let name = self.to_text(&name)?;
                            self.variable(frame, &name, host)?
                        } else {
                            let name = self.pop(frame)?;
                            let receiver = self.pop(frame)?;
                            if matches!(name, Value::Undefined)
                                || matches!(&name,Value::String(s)if s.is_empty())
                            {
                                receiver
                            } else {
                                let key = self.to_text(&name)?;
                                let id = self.object_id(receiver)?;
                                self.read_member(id, &key, host)?
                            }
                        };
                        let args = self.args(frame)?;
                        let constructor = self.object_id(callable.clone())?;
                        let prototype = match self.get(constructor, "prototype")? {
                            Value::Object(id) => Some(id),
                            Value::Undefined | Value::Null => None,
                            _ => {
                                return self.fail(ErrorKind::Unsupported(
                                    "primitive constructor prototype",
                                ));
                            }
                        };
                        let object = self.alloc(Kind::Plain, prototype)?;
                        let returned = self.invoke(callable, Value::Object(object), &args, host)?;
                        self.push(
                            frame,
                            if matches!(returned, Value::Object(_)) {
                                returned
                            } else {
                                Value::Object(object)
                            },
                        )?;
                    }
                    0x69 => {
                        let superclass = self.object_id(self.pop(frame)?)?;
                        let subclass = self.object_id(self.pop(frame)?)?;
                        for class in [superclass, subclass] {
                            if !matches!(
                                self.objects[class.0].kind,
                                Kind::Script(_)
                                    | Kind::Native(_)
                                    | Kind::ObjectConstructor
                                    | Kind::AddProperty
                                    | Kind::FunctionMethod(_)
                            ) {
                                return self.fail(ErrorKind::UnboundCall);
                            }
                        }
                        let parent = self.object_id(self.get(superclass, "prototype")?)?;
                        let prototype = self.alloc(Kind::Plain, Some(parent))?;
                        self.set(prototype, "__constructor__", Value::Object(superclass))?;
                        self.set(subclass, "prototype", Value::Object(prototype))?;
                    }
                    0x3e => return Ok(Flow::Return(self.pop(frame)?)),
                    0x26 => {
                        let value = self.pop(frame)?;
                        self.command(frame.target, HostCommand::Trace(value), host)?;
                    }
                    0x49 | 0x66 => {
                        let a = self.pop(frame)?;
                        let b = self.pop(frame)?;
                        let value = if instruction.opcode == 0x66 {
                            self.strict_equal(&a, &b)
                        } else {
                            self.equal(&a, &b)?
                        };
                        self.push(frame, Value::Bool(value))?;
                    }
                    0x47 => {
                        let a = self.pop(frame)?;
                        let b = self.pop(frame)?;
                        let value = if matches!(a, Value::String(_))
                            || matches!(b, Value::String(_))
                        {
                            let b = self.to_text(&b)?;
                            let a = self.to_text(&a)?;
                            let mut units = Vec::new();
                            if b.len().saturating_add(a.len())
                                > self.limits.string_units.saturating_sub(self.string_units)
                            {
                                return self.fail(ErrorKind::Budget("concatenated string units"));
                            }
                            units.extend_from_slice(&b);
                            units.extend_from_slice(&a);
                            Value::String(self.allocate_text(units)?)
                        } else {
                            Value::Number(self.number(&b)? + self.number(&a)?)
                        };
                        self.push(frame, value)?;
                    }
                    0x48 | 0x67 => {
                        let a = self.pop(frame)?;
                        let b = self.pop(frame)?;
                        let value = if let (Value::String(a), Value::String(b)) = (&a, &b) {
                            if instruction.opcode == 0x48 {
                                b < a
                            } else {
                                b > a
                            }
                        } else {
                            let a = self.number(&a)?;
                            let b = self.number(&b)?;
                            if a.is_nan() || b.is_nan() {
                                return self
                                    .fail(ErrorKind::Unsupported("NaN relational result profile"));
                            }
                            if instruction.opcode == 0x48 {
                                b < a
                            } else {
                                b > a
                            }
                        };
                        self.push(frame, Value::Bool(value))?;
                    }
                    0x0b | 0x0c | 0x0d | 0x3f | 0x60..=0x65 => {
                        let a_value = self.pop(frame)?;
                        let b_value = self.pop(frame)?;
                        if instruction.opcode <= 0x0d
                            && (!matches!(a_value, Value::Number(_))
                                || !matches!(b_value, Value::Number(_)))
                        {
                            return self.fail(ErrorKind::Unsupported(
                                "legacy arithmetic operand coercion",
                            ));
                        }
                        let a = self.number(&a_value)?;
                        let b = self.number(&b_value)?;
                        let value = match instruction.opcode {
                            0x0b => b - a,
                            0x0c => b * a,
                            0x0d => b / a,
                            0x3f => b % a,
                            0x60 => f64::from((uint32(b) & uint32(a)) as i32),
                            0x61 => f64::from((uint32(b) | uint32(a)) as i32),
                            0x62 => f64::from((uint32(b) ^ uint32(a)) as i32),
                            0x63 => f64::from(uint32(b).wrapping_shl(uint32(a) & 31) as i32),
                            0x64 => f64::from((uint32(b) as i32) >> (uint32(a) & 31)),
                            _ => f64::from(uint32(b) >> (uint32(a) & 31)),
                        };
                        self.push(frame, Value::Number(value))?;
                    }
                    0x18 | 0x4a | 0x50 | 0x51 => {
                        let value = self.number(&self.pop(frame)?)?;
                        let value = match instruction.opcode {
                            0x18 => {
                                if value.is_nan() {
                                    0.0
                                } else {
                                    value.trunc()
                                }
                            }
                            0x50 => value + 1.0,
                            0x51 => value - 1.0,
                            _ => value,
                        };
                        self.push(frame, Value::Number(value))?;
                    }
                    0x4b => {
                        let value = self.pop(frame)?;
                        let value = self.to_text(&value)?;
                        self.push(frame, Value::String(value))?;
                    }
                    0x44 => {
                        let value = self.pop(frame)?;
                        let name = match value {
                            Value::Undefined => "undefined",
                            Value::Null => "null",
                            Value::Bool(_) => "boolean",
                            Value::Number(_) => "number",
                            Value::String(_) => "string",
                            Value::Object(id) => {
                                self.check_object(id)?;
                                if matches!(
                                    self.objects[id.0].kind,
                                    Kind::Script(_)
                                        | Kind::Native(_)
                                        | Kind::ObjectConstructor
                                        | Kind::AddProperty
                                        | Kind::FunctionMethod(_)
                                ) {
                                    "function"
                                } else {
                                    "object"
                                }
                            }
                        };
                        self.push(frame, Value::string(name))?;
                    }
                    _ => return self.fail(ErrorKind::UnsupportedOpcode(instruction.opcode)),
                },
                _ => return self.fail(ErrorKind::UnsupportedOpcode(instruction.opcode)),
            }
            if let Some(target) = jump {
                if Some(target) == program.span.end() {
                    return Ok(Flow::End);
                }
                if target < program.span.offset || Some(target) > program.span.end() {
                    return Ok(Flow::Jump(target));
                }
                match program
                    .instructions
                    .binary_search_by_key(&target, |i| i.span.offset)
                {
                    Ok(index) => pc = index,
                    Err(_) => return self.fail(ErrorKind::InvalidBranch(target as i128)),
                }
            } else {
                pc += 1;
            }
        }
        Ok(Flow::End)
    }
}
fn uint32(number: f64) -> u32 {
    if !number.is_finite() || number == 0.0 {
        0
    } else {
        number.trunc().rem_euclid(4_294_967_296.0) as u32
    }
}
fn decimal(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = usize::from(bytes.first().is_some_and(|c| *c == b'+' || *c == b'-'));
    let mut digits = 0;
    while bytes.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
        digits += 1;
    }
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return false;
    }
    if bytes.get(i).is_some_and(|c| *c == b'e' || *c == b'E') {
        i += 1;
        if bytes.get(i).is_some_and(|c| *c == b'+' || *c == b'-') {
            i += 1;
        }
        let start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == start {
            return false;
        }
    }
    i == bytes.len()
}
fn array_index(key: &Text) -> Option<u32> {
    let s = String::from_utf16(key).ok()?;
    let v = s.parse::<u32>().ok()?;
    if v != u32::MAX && v.to_string() == s {
        Some(v)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn action(op: u8, payload: &[u8]) -> Vec<u8> {
        let mut out = vec![op];
        out.extend((payload.len() as u16).to_le_bytes());
        out.extend(payload);
        out
    }
    fn push(values: &[Value]) -> Vec<u8> {
        let mut payload = Vec::new();
        for value in values {
            match value {
                Value::Number(n) => {
                    payload.push(6);
                    let bits = n.to_bits();
                    payload.extend(((bits >> 32) as u32).to_le_bytes());
                    payload.extend((bits as u32).to_le_bytes());
                }
                Value::String(s) => {
                    payload.push(0);
                    payload.extend(String::from_utf16(s).unwrap().as_bytes());
                    payload.push(0);
                }
                Value::Bool(v) => payload.extend([5, u8::from(*v)]),
                Value::Null => payload.push(2),
                Value::Undefined => payload.push(3),
                _ => panic!(),
            }
        }
        action(0x96, &payload)
    }
    fn setup() -> Vm {
        Vm::new(15, Limits::default()).unwrap()
    }
    fn execute(vm: &mut Vm, bytes: &[u8]) -> Result<Execution> {
        let code = vm.register(Arc::new(Program::parse(bytes).unwrap()))?;
        vm.execute(code, vm.root(), &mut RejectHost)
    }
    fn function(name: &str, body: &[u8]) -> Vec<u8> {
        let mut payload = name.as_bytes().to_vec();
        payload.extend([0, 0, 0]);
        payload.extend((body.len() as u16).to_le_bytes());
        let mut out = action(0x9b, &payload);
        out.extend(body);
        out
    }

    #[test]
    fn primitive_values_and_conservative_coercions() {
        let mut vm = setup();
        assert!(!Value::Number(f64::NAN).truthy());
        assert!(!Value::Number(-0.0).truthy());
        assert!(Value::string("0").truthy());
        assert!(vm.strict_equal(&Value::Number(0.0), &Value::Number(-0.0)));
        assert!(!vm.strict_equal(&Value::Number(f64::NAN), &Value::Number(f64::NAN)));
        assert!(vm.equal(&Value::Null, &Value::Undefined).unwrap());
        assert!(
            vm.equal(&Value::string("12.5"), &Value::Number(12.5))
                .unwrap()
        );
        assert!(vm.number(&Value::string("")).is_err());
        assert!(vm.number(&Value::Null).is_err());
        assert!(vm.number(&Value::string("0x10")).is_err());
        assert!(vm.to_text(&Value::Number(1.25)).is_err());
        assert_eq!(vm.to_text(&Value::Number(-0.0)).unwrap(), text("0"));
        assert_eq!(uint32(-1.0), u32::MAX);
        assert_eq!(uint32(4_294_967_297.0), 1);
        assert_eq!(uint32(f64::INFINITY), 0);
    }
    #[test]
    fn arithmetic_order_register_and_integer_bits() {
        let mut bytes = push(&[Value::Number(8.0), Value::Number(2.0)]);
        bytes.push(0x0b);
        bytes.extend(action(0x87, &[2]));
        bytes.push(0x17);
        bytes.extend(action(0x96, &[4, 2, 7, 255, 255, 255, 255]));
        let result = execute(&mut setup(), &bytes).unwrap();
        assert_eq!(result.stack, vec![Value::Number(6.0), Value::Number(-1.0)]);
        let mut bytes = push(&[Value::Number(4_294_967_297.0), Value::Number(1.0)]);
        bytes.push(0x63);
        assert_eq!(
            execute(&mut setup(), &bytes).unwrap().stack,
            vec![Value::Number(2.0)]
        );
    }
    #[test]
    fn functions_capture_pool_and_code_survives_later_registration() {
        let body = action(0x96, &[8, 0]);
        let mut returning = body;
        returning.push(0x3e);
        let mut bytes = action(0x88, &[1, 0, b'a', 0]);
        bytes.extend(function("f", &returning));
        bytes.extend(action(0x88, &[1, 0, b'b', 0]));
        let mut vm = setup();
        execute(&mut vm, &bytes).unwrap();
        execute(&mut vm, &[0]).unwrap();
        let f = vm.get(vm.root(), "f").unwrap();
        let result = vm
            .call(f, Value::Object(vm.root()), &[], &mut RejectHost)
            .unwrap();
        assert_eq!(result.value, Value::string("a"));
    }
    #[test]
    fn function_arguments_and_return() {
        let mut payload = vec![b'f', 0, 1, 0, b'x', 0];
        let mut body = push(&[Value::string("x")]);
        body.extend([0x1c, 0x3e]);
        payload.extend((body.len() as u16).to_le_bytes());
        let mut bytes = action(0x9b, &payload);
        bytes.extend(body);
        bytes.extend(push(&[
            Value::Number(42.0),
            Value::Number(1.0),
            Value::string("f"),
        ]));
        bytes.push(0x3d);
        let result = execute(&mut setup(), &bytes).unwrap();
        assert_eq!(result.stack, vec![Value::Number(42.0)]);
    }
    #[test]
    fn array_and_prototype_properties_are_observable() {
        let mut vm = setup();
        let array = vm.array(&[Value::Number(1.0), Value::Number(2.0)]).unwrap();
        assert_eq!(vm.get(array, "length").unwrap(), Value::Number(2.0));
        vm.set(array, "length", Value::Number(1.0)).unwrap();
        assert_eq!(vm.get(array, "1").unwrap(), Value::Undefined);
        vm.set(array, "4", Value::Number(9.0)).unwrap();
        assert_eq!(vm.get(array, "length").unwrap(), Value::Number(5.0));
        let p = vm.object().unwrap();
        let child = vm.object().unwrap();
        vm.set(p, "x", Value::Number(3.0)).unwrap();
        vm.set(child, "__proto__", Value::Object(p)).unwrap();
        assert_eq!(vm.get(child, "x").unwrap(), Value::Number(3.0));
        assert!(vm.set(p, "__proto__", Value::Object(child)).is_err());
    }
    #[test]
    fn rejects_unknown_and_missing_binding_without_success() {
        let error = execute(&mut setup(), &[0x2c]).unwrap_err();
        assert_eq!(error.kind, ErrorKind::UnsupportedOpcode(0x2c));
        assert_eq!(error.source_offset, Some(0));
        let mut bytes = push(&[Value::Number(0.0), Value::string("missing")]);
        bytes.push(0x3d);
        assert_eq!(
            execute(&mut setup(), &bytes).unwrap_err().kind,
            ErrorKind::UnboundCall
        );
        assert!(matches!(
            execute(&mut setup(), &[0x07]).unwrap_err().kind,
            ErrorKind::Host(_)
        ));
        assert_eq!(
            execute(&mut setup(), &action(0x96, &[8, 0]))
                .unwrap_err()
                .kind,
            ErrorKind::InvalidConstant(0)
        );
    }
    #[test]
    fn budgets_stop_loops_recursion_stack_and_allocations() {
        let mut vm = Vm::new(
            15,
            Limits {
                instructions: 7,
                ..Limits::default()
            },
        )
        .unwrap();
        let error = execute(&mut vm, &action(0x99, &(-5i16).to_le_bytes())).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Budget("instructions"));
        assert_eq!(vm.instruction_count(), 7);
        let mut body = push(&[Value::Number(0.0), Value::string("f")]);
        body.extend([0x3d, 0x3e]);
        let mut bytes = function("f", &body);
        bytes.extend(push(&[Value::Number(0.0), Value::string("f")]));
        bytes.push(0x3d);
        let mut vm = Vm::new(
            15,
            Limits {
                call_depth: 3,
                ..Limits::default()
            },
        )
        .unwrap();
        assert_eq!(
            execute(&mut vm, &bytes).unwrap_err().kind,
            ErrorKind::Budget("function call depth")
        );
        let mut vm = Vm::new(
            15,
            Limits {
                stack_values: 1,
                ..Limits::default()
            },
        )
        .unwrap();
        assert_eq!(
            execute(&mut vm, &push(&[Value::Number(1.0), Value::Number(2.0)]))
                .unwrap_err()
                .kind,
            ErrorKind::Budget("operand stack")
        );
        let mut vm = Vm::new(
            15,
            Limits {
                objects: 2,
                ..Limits::default()
            },
        )
        .unwrap();
        assert_eq!(vm.object().unwrap_err().kind, ErrorKind::Budget("objects"));
        let mut vm = Vm::new(
            15,
            Limits {
                properties: 5,
                ..Limits::default()
            },
        )
        .unwrap();
        assert_eq!(
            vm.set(vm.root(), "extra", Value::Null).unwrap_err().kind,
            ErrorKind::Budget("properties")
        );
    }
    struct BoundHost {
        calls: usize,
    }
    impl Host for BoundHost {
        fn call(&mut self, call: HostCall<'_>) -> std::result::Result<Value, String> {
            assert_eq!(call.binding, 17);
            assert_eq!(call.args, &[Value::Number(7.0)]);
            self.calls += 1;
            Ok(Value::Number(42.0))
        }
        fn command(&mut self, _: ObjectId, _: HostCommand) -> std::result::Result<(), String> {
            Err("unsupported".into())
        }
    }
    #[test]
    fn only_explicit_native_binding_invokes_host() {
        let mut vm = setup();
        let native = vm.native(17).unwrap();
        vm.set(vm.root(), "bound", Value::Object(native)).unwrap();
        let mut bytes = push(&[
            Value::Number(7.0),
            Value::Number(1.0),
            Value::string("bound"),
        ]);
        bytes.push(0x3d);
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        let mut host = BoundHost { calls: 0 };
        let result = vm.execute(code, vm.root(), &mut host).unwrap();
        assert_eq!(result.stack, vec![Value::Number(42.0)]);
        assert_eq!(host.calls, 1);
        assert_eq!(result.host_calls, 1);
    }
    #[test]
    fn inherited_undefined_shadows_outer_scope() {
        let mut vm = setup();
        let prototype = vm.object().unwrap();
        let inner = vm.object().unwrap();
        vm.set(prototype, "shadow", Value::Undefined).unwrap();
        vm.set(inner, "__proto__", Value::Object(prototype))
            .unwrap();
        vm.set(vm.global(), "shadow", Value::Number(7.0)).unwrap();
        let frame = Frame {
            stack: Vec::new(),
            registers: vec![Value::Undefined; 4],
            scopes: vec![inner, vm.global()],
            pool: Arc::new(Vec::new()),
            target: vm.root(),
            this: Value::Object(inner),
            this_visible: true,
            locals: inner,
        };
        assert_eq!(
            vm.variable(&frame, &text("shadow"), &mut RejectHost)
                .unwrap(),
            Value::Undefined
        );
    }
    #[test]
    fn object_constructor_and_extends_do_not_invoke_superclass() {
        let mut vm = setup();
        let object = vm.install_object_constructor().unwrap();
        assert_eq!(vm.install_object_constructor().unwrap(), object);
        let root = vm.root();
        let made = vm
            .call(
                Value::Object(object),
                Value::Object(root),
                &[],
                &mut RejectHost,
            )
            .unwrap()
            .value;
        let Value::Object(made) = made else { panic!() };
        assert_eq!(
            vm.get(made, "__proto__").unwrap(),
            vm.get(object, "prototype").unwrap()
        );
        let superclass = vm.native(77).unwrap();
        let parent = vm.object().unwrap();
        vm.set(parent, "inherited", Value::Number(19.0)).unwrap();
        vm.set(superclass, "prototype", Value::Object(parent))
            .unwrap();
        vm.set(root, "Super", Value::Object(superclass)).unwrap();
        let mut bytes = function("Sub", &[]);
        bytes.extend(push(&[Value::string("Sub")]));
        bytes.push(0x1c);
        bytes.extend(push(&[Value::string("Super")]));
        bytes.extend([0x1c, 0x69]);
        let result = execute(&mut vm, &bytes).unwrap();
        assert_eq!(result.host_calls, 0);
        let Value::Object(sub) = vm.get(root, "Sub").unwrap() else {
            panic!()
        };
        let Value::Object(prototype) = vm.get(sub, "prototype").unwrap() else {
            panic!()
        };
        assert_eq!(
            vm.get(prototype, "__proto__").unwrap(),
            Value::Object(parent)
        );
        assert_eq!(
            vm.get(prototype, "__constructor__").unwrap(),
            Value::Object(superclass)
        );
        assert_eq!(vm.get(prototype, "inherited").unwrap(), Value::Number(19.0));
    }
    #[test]
    fn accessors_are_lazy_inherited_and_execute_with_instance_receiver() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        let mut getter = push(&[Value::string("this")]);
        getter.push(0x1c);
        getter.extend(push(&[Value::string("stored")]));
        getter.extend([0x4e, 0x3e]);
        let mut setter = push(&[Value::string("this")]);
        setter.push(0x1c);
        setter.extend(push(&[Value::string("stored"), Value::string("value")]));
        setter.extend([0x1c, 0x4f]);
        let mut declaration = function("getter", &getter);
        let mut params = vec![b's', 0, 1, 0, b'v', b'a', b'l', b'u', b'e', 0];
        params.extend((setter.len() as u16).to_le_bytes());
        declaration.extend(action(0x9b, &params));
        declaration.extend(setter);
        execute(&mut vm, &declaration).unwrap();
        let prototype = vm.ordinary_object().unwrap();
        let instance = vm.object().unwrap();
        vm.set(instance, "__proto__", Value::Object(prototype))
            .unwrap();
        vm.set(instance, "stored", Value::Number(5.0)).unwrap();
        let getter = vm.get(vm.root(), "getter").unwrap();
        let setter = vm.get(vm.root(), "s").unwrap();
        let add = vm.get(prototype, "addProperty").unwrap();
        let registered = vm
            .call(
                add.clone(),
                Value::Object(prototype),
                &[Value::string("field"), getter.clone(), setter],
                &mut RejectHost,
            )
            .unwrap();
        assert_eq!(registered.value, Value::Bool(true));
        assert_eq!(registered.instructions, 0);
        assert_eq!(vm.get(instance, "stored").unwrap(), Value::Number(5.0));
        assert!(vm.get(instance, "field").is_err());
        assert_eq!(
            vm.read_property(instance, "field", &mut RejectHost)
                .unwrap()
                .value,
            Value::Number(5.0)
        );
        vm.write_property(instance, "field", Value::Number(42.0), &mut RejectHost)
            .unwrap();
        assert_eq!(vm.get(instance, "stored").unwrap(), Value::Number(42.0));
        assert_eq!(vm.get(prototype, "stored").unwrap(), Value::Undefined);
        vm.call(
            add.clone(),
            Value::Object(prototype),
            &[Value::string("field"), getter, Value::Null],
            &mut RejectHost,
        )
        .unwrap();
        vm.write_property(instance, "field", Value::Number(99.0), &mut RejectHost)
            .unwrap();
        assert_eq!(vm.get(instance, "stored").unwrap(), Value::Number(42.0));
        let result = vm
            .call(
                add,
                Value::Object(prototype),
                &[Value::string("bad"), Value::Number(1.0), Value::Null],
                &mut RejectHost,
            )
            .unwrap();
        assert_eq!(result.value, Value::Bool(false));
        assert!(!vm.has_key(prototype, &text("bad")).unwrap());
    }
    #[test]
    fn call_apply_forward_explicit_receiver_and_arrays_with_bounded_recursion() {
        let mut vm = setup();
        let mut body = push(&[Value::string("this")]);
        body.extend([0x1c, 0x3e]);
        execute(&mut vm, &function("f", &body)).unwrap();
        let function = vm.get(vm.root(), "f").unwrap();
        let receiver = vm.object().unwrap();
        let call = vm.function_method(FunctionMethod::Call).unwrap();
        let apply = vm.function_method(FunctionMethod::Apply).unwrap();
        let result = vm
            .call(
                Value::Object(call),
                function.clone(),
                &[Value::Object(receiver)],
                &mut RejectHost,
            )
            .unwrap();
        assert_eq!(result.value, Value::Object(receiver));
        let empty = vm.array(&[]).unwrap();
        let result = vm
            .call(
                Value::Object(apply),
                function.clone(),
                &[Value::Object(receiver), Value::Object(empty)],
                &mut RejectHost,
            )
            .unwrap();
        assert_eq!(result.value, Value::Object(receiver));
        assert!(
            vm.call(
                Value::Object(apply),
                function,
                &[Value::Object(receiver), Value::Null],
                &mut RejectHost
            )
            .is_err()
        );
        let mut vm = Vm::new(
            15,
            Limits {
                call_depth: 3,
                ..Limits::default()
            },
        )
        .unwrap();
        let apply = vm.function_method(FunctionMethod::Apply).unwrap();
        let arguments = vm.array(&[]).unwrap();
        vm.set(arguments, "0", Value::Object(apply)).unwrap();
        vm.set(arguments, "1", Value::Object(arguments)).unwrap();
        let error = vm
            .call(
                Value::Object(apply),
                Value::Object(apply),
                &[Value::Object(apply), Value::Object(arguments)],
                &mut RejectHost,
            )
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Budget("function call depth"));
    }
    #[test]
    fn native_observed_empty_pop_is_scoped_to_action_pop() {
        let result = execute(&mut setup(), &[0x17, 0x17, 0]).unwrap();
        assert!(result.stack.is_empty());
        assert_eq!(result.instructions, 3);
        let error = execute(&mut setup(), &[0x1c]).unwrap_err();
        assert_eq!(error.kind, ErrorKind::StackUnderflow);
    }
}
