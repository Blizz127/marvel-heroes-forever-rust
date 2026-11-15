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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CodeId(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeOrigin {
    pub code: CodeId,
    pub source_offset: usize,
}
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
    pub member_visits: usize,
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
            member_visits: 1_000_000,
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
    AmbiguousPropertyFlags { old: u8, set: u8 },
}
#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub origin: Option<CodeOrigin>,
    pub source_offset: Option<usize>,
    pub opcode: Option<u8>,
    pub kind: ErrorKind,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AVM1 {:?} at {:?} (code {:?}, opcode {:?})",
            self.kind,
            self.source_offset,
            self.origin.map(|origin| origin.code),
            self.opcode
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
    pub origin: Option<CodeOrigin>,
    pub binding: u32,
    pub this: Value,
    pub args: &'a [Value],
    pub source_offset: Option<usize>,
}
/// Adobe's indexed movie properties; the host owns coordinate/state semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MovieProperty {
    X = 0,
    Y = 1,
    XScale = 2,
    YScale = 3,
    CurrentFrame = 4,
    TotalFrames = 5,
    Alpha = 6,
    Visible = 7,
    Width = 8,
    Height = 9,
    Rotation = 10,
    Target = 11,
    FramesLoaded = 12,
    Name = 13,
    DropTarget = 14,
    Url = 15,
    HighQuality = 16,
    FocusRect = 17,
    SoundBufferTime = 18,
    Quality = 19,
    MouseX = 20,
    MouseY = 21,
}
#[derive(Debug, Clone)]
pub struct PropertyRequest {
    pub origin: Option<CodeOrigin>,
    pub context: ObjectId,
    pub target: Value,
    pub property: MovieProperty,
    pub source_offset: Option<usize>,
}
#[derive(Debug, Clone)]
pub struct ClassRegistration {
    pub name: Text,
    /// None means explicit null/unregister; undefined is never converted to it.
    pub constructor: Option<ObjectId>,
    /// Call-site provenance only; the current AS2 root owns the registry context.
    pub origin: Option<CodeOrigin>,
    /// Definition may belong to an earlier-loaded movie. None for native/builtin functions.
    pub constructor_origin: Option<CodeOrigin>,
}
pub trait Host {
    fn call(&mut self, call: HostCall<'_>) -> std::result::Result<Value, String>;
    fn command(
        &mut self,
        target: ObjectId,
        command: HostCommand,
    ) -> std::result::Result<(), String>;
    /// Return true only after applying the association/removal to a real registry.
    fn register_class(
        &mut self,
        _registration: ClassRegistration,
    ) -> std::result::Result<bool, String> {
        Err("unhandled Object.registerClass registry request".into())
    }
    fn get_property(&mut self, request: PropertyRequest) -> std::result::Result<Value, String> {
        Err(format!(
            "unhandled movie-property read {:?}",
            request.property
        ))
    }
    fn set_property(
        &mut self,
        request: PropertyRequest,
        _value: Value,
    ) -> std::result::Result<(), String> {
        Err(format!(
            "unhandled movie-property write {:?}",
            request.property
        ))
    }
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
/// Script-object roles only. Neither role implies a Scene/display attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovieClipRole {
    UnboundPrototype,
    UnboundInstance,
}
#[derive(Debug, Clone)]
enum Kind {
    Plain,
    Array(u32),
    Script(ScriptFunction),
    Native(u32),
    ObjectConstructor,
    ArrayConstructor,
    FunctionConversion,
    MovieClipConstructor,
    MovieClip(MovieClipRole),
    AddProperty,
    RegisterClass,
    SetPropertyFlags,
    FunctionMethod(FunctionMethod),
    FunctionPrototype,
    ConstructorSuper {
        constructor: ObjectId,
        receiver: ObjectId,
    },
}
impl Kind {
    fn function_value(&self) -> bool {
        matches!(
            self,
            Self::Native(_)
                | Self::Script(_)
                | Self::ObjectConstructor
                | Self::ArrayConstructor
                | Self::FunctionConversion
                | Self::MovieClipConstructor
                | Self::AddProperty
                | Self::RegisterClass
                | Self::SetPropertyFlags
                | Self::FunctionMethod(_)
        )
    }
}
#[derive(Debug, Clone)]
enum Property {
    Data(Value),
    Accessor {
        getter: Value,
        setter: Option<Value>,
    },
}
#[derive(Debug, Clone, Copy)]
struct ClassBase {
    constructor: ObjectId,
    prototype: ObjectId,
    parent_prototype: ObjectId,
}
#[derive(Debug, Clone)]
struct Object {
    kind: Kind,
    properties: BTreeMap<Text, Property>,
    flags: BTreeMap<Text, u8>,
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
    movie_version: u8,
    loaded_movie_targets: BTreeMap<ObjectId, u8>,
    code: Vec<Arc<Program>>,
    code_bytes: usize,
    objects: Vec<Object>,
    properties: usize,
    member_visits: usize,
    string_units: usize,
    instructions: usize,
    host_calls: usize,
    depth: usize,
    current: Option<(usize, u8)>,
    current_code: Option<CodeId>,
    current_target: Option<ObjectId>,
    global: ObjectId,
    root: ObjectId,
    object_constructor: Option<ObjectId>,
    function_prototype: Option<ObjectId>,
    function_conversion: Option<ObjectId>,
    movie_clip_constructor: Option<ObjectId>,
    movie_clip_prototype: Option<ObjectId>,
    movie_clip_parent: Option<ObjectId>,
    movie_clip_event_names: BTreeMap<ObjectId, bool>,
    array_constructor: Option<ObjectId>,
    array_prototype: Option<ObjectId>,
    class_bases: BTreeMap<ObjectId, ClassBase>,
}
impl Vm {
    /// This execution profile supports SWF 7+; legacy coercions are not guessed.
    pub fn new(version: u8, limits: Limits) -> Result<Self> {
        let mut vm = Self {
            limits,
            movie_version: version,
            loaded_movie_targets: BTreeMap::new(),
            code: Vec::new(),
            code_bytes: 0,
            objects: Vec::new(),
            properties: 0,
            member_visits: 0,
            string_units: 0,
            instructions: 0,
            host_calls: 0,
            depth: 0,
            current: None,
            current_code: None,
            current_target: None,
            global: ObjectId(0),
            root: ObjectId(0),
            object_constructor: None,
            function_prototype: None,
            function_conversion: None,
            movie_clip_constructor: None,
            movie_clip_prototype: None,
            movie_clip_parent: None,
            movie_clip_event_names: BTreeMap::new(),
            array_constructor: None,
            array_prototype: None,
            class_bases: BTreeMap::new(),
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
    /// Integration must establish that this target belongs to a successfully
    /// loaded movie. Failed/empty definitions and arbitrary objects do not qualify.
    /// Every registered program executed in this context must share this profile.
    pub fn bind_loaded_movie_target(&mut self, target: ObjectId, header_version: u8) -> Result<()> {
        self.check_object(target)?;
        if header_version != self.movie_version {
            return self.fail(ErrorKind::Unsupported(
                "loaded movie target version mismatch",
            ));
        }
        self.loaded_movie_targets.insert(target, header_version);
        Ok(())
    }
    pub fn instruction_count(&self) -> usize {
        self.instructions
    }
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }
    fn origin(&self) -> Option<CodeOrigin> {
        self.current_code
            .zip(self.current)
            .map(|(code, (source_offset, _))| CodeOrigin {
                code,
                source_offset,
            })
    }
    fn fail<T>(&self, kind: ErrorKind) -> Result<T> {
        Err(Error {
            origin: self.origin(),
            source_offset: self.current.map(|v| v.0),
            opcode: self.current.map(|v| v.1),
            kind,
        })
    }
    fn error(&self, kind: ErrorKind) -> Error {
        Error {
            origin: self.origin(),
            source_offset: self.current.map(|v| v.0),
            opcode: self.current.map(|v| v.1),
            kind,
        }
    }
    fn alloc(&mut self, kind: Kind, prototype: Option<ObjectId>) -> Result<ObjectId> {
        let prototype = prototype.or_else(|| {
            kind.function_value()
                .then_some(self.function_prototype)
                .flatten()
        });
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
            flags: BTreeMap::new(),
            prototype,
        });
        if let Some(parent) = prototype {
            self.raw_member(
                id,
                text("__proto__"),
                Property::Data(Value::Object(parent)),
                3,
            )?;
        }
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
        if self
            .objects
            .iter()
            .any(|object| matches!(object.kind, Kind::Script(_)))
        {
            return self.fail(ErrorKind::Unsupported(
                "install Object before defining script functions",
            ));
        }
        let constructor = self.alloc(Kind::ObjectConstructor, None)?;
        let prototype = self.object()?;
        let function_prototype = self.alloc(Kind::FunctionPrototype, Some(prototype))?;
        self.function_prototype = Some(function_prototype);
        let prior_functions: Vec<_> = self
            .objects
            .iter()
            .enumerate()
            .filter(|(_, object)| {
                object.kind.function_value()
                    && object.prototype.is_none()
                    && !object.properties.contains_key(&text("__proto__"))
            })
            .map(|(index, _)| ObjectId(index))
            .collect();
        for id in prior_functions {
            self.raw_member(
                id,
                text("__proto__"),
                Property::Data(Value::Object(function_prototype)),
                3,
            )?;
            self.objects[id.0].prototype = Some(function_prototype);
        }
        let call = self.function_method(FunctionMethod::Call)?;
        let apply = self.function_method(FunctionMethod::Apply)?;
        self.set(function_prototype, "call", Value::Object(call))?;
        self.set(function_prototype, "apply", Value::Object(apply))?;
        self.set(constructor, "prototype", Value::Object(prototype))?;
        self.set(prototype, "constructor", Value::Object(constructor))?;
        let add_property = self.alloc(Kind::AddProperty, None)?;
        self.set(prototype, "addProperty", Value::Object(add_property))?;
        let register_class = self.alloc(Kind::RegisterClass, None)?;
        self.set(constructor, "registerClass", Value::Object(register_class))?;
        self.set(self.global, "Object", Value::Object(constructor))?;
        self.object_constructor = Some(constructor);
        Ok(constructor)
    }
    /// Install the native-backed single-argument Function conversion, separately
    /// from dynamic construction or arbitrary source compilation.
    pub fn install_function_conversion(&mut self) -> Result<ObjectId> {
        if let Some(id) = self.function_conversion {
            return Ok(id);
        }
        let Some(prototype) = self.function_prototype else {
            return self.fail(ErrorKind::Unsupported(
                "install Object before Function conversion",
            ));
        };
        if self.has_key(self.global, &text("Function"))? {
            return self.fail(ErrorKind::Unsupported("Function binding already exists"));
        }
        let function = self.alloc(Kind::FunctionConversion, None)?;
        self.set(function, "prototype", Value::Object(prototype))?;
        self.set(self.global, "Function", Value::Object(function))?;
        self.function_conversion = Some(function);
        Ok(function)
    }
    /// Install the native-backed unbound AS2 MovieClip object profile.
    /// It creates no display object, timeline instance, event queue or host binding.
    pub fn install_movie_clip_constructor(&mut self) -> Result<ObjectId> {
        if let Some(id) = self.movie_clip_constructor {
            return Ok(id);
        }
        let Some(object) = self.object_constructor else {
            return self.fail(ErrorKind::Unsupported("install Object before MovieClip"));
        };
        if self.has_key(self.global, &text("MovieClip"))? {
            return self.fail(ErrorKind::Unsupported("MovieClip binding already exists"));
        }
        let parent = self.object_id(self.get(object, "prototype")?)?;
        let constructor = self.alloc(Kind::MovieClipConstructor, None)?;
        let prototype = self.alloc(
            Kind::MovieClip(MovieClipRole::UnboundPrototype),
            Some(parent),
        )?;
        self.raw_member(
            constructor,
            text("prototype"),
            Property::Data(Value::Object(prototype)),
            3,
        )?;
        self.raw_member(
            prototype,
            text("constructor"),
            Property::Data(Value::Object(constructor)),
            3,
        )?;
        self.set(self.global, "MovieClip", Value::Object(constructor))?;
        self.movie_clip_constructor = Some(constructor);
        self.movie_clip_prototype = Some(prototype);
        self.movie_clip_parent = Some(parent);
        Ok(constructor)
    }
    /// Inspect an actual VM category; prototype inheritance alone cannot grant it.
    pub fn movie_clip_role(&self, id: ObjectId) -> Result<Option<MovieClipRole>> {
        self.check_object(id)?;
        Ok(match self.objects[id.0].kind {
            Kind::MovieClip(role) => Some(role),
            _ => None,
        })
    }
    /// Allocation-time native mouse-event-name marker, not listener registration.
    /// Only fresh typed unbound instances have this state; mutation is gated.
    pub fn movie_clip_has_mouse_event_names(&self, id: ObjectId) -> Result<Option<bool>> {
        self.check_object(id)?;
        Ok(self.movie_clip_event_names.get(&id).copied())
    }
    /// Installs the documented Array creation forms and the REA-verified empty
    /// Array prototype. Prototype mutation remains a separately gated profile.
    /// Install before defining script functions or creating array values.
    pub fn install_array_constructor(&mut self) -> Result<ObjectId> {
        if let Some(id) = self.array_constructor {
            return Ok(id);
        }
        let Some(object) = self.object_constructor else {
            return self.fail(ErrorKind::Unsupported("install Object before Array"));
        };
        if self.has_key(self.global, &text("Array"))? {
            return self.fail(ErrorKind::Unsupported("Array binding already exists"));
        }
        if self
            .objects
            .iter()
            .any(|object| matches!(object.kind, Kind::Script(_) | Kind::Array(_)))
        {
            return self.fail(ErrorKind::Unsupported(
                "install Array before scripts or array creation",
            ));
        }
        let parent = self.object_id(self.get(object, "prototype")?)?;
        let constructor = self.alloc(Kind::ArrayConstructor, None)?;
        let prototype = self.alloc(Kind::Array(0), Some(parent))?;
        self.set(constructor, "prototype", Value::Object(prototype))?;
        self.set(prototype, "constructor", Value::Object(constructor))?;
        self.set(self.global, "Array", Value::Object(constructor))?;
        self.array_constructor = Some(constructor);
        self.array_prototype = Some(prototype);
        Ok(constructor)
    }
    fn check_member_target(&self, id: ObjectId, key: &Text) -> Result<()> {
        if matches!(
            self.objects[id.0].kind,
            Kind::MovieClip(MovieClipRole::UnboundInstance)
        ) && key.as_ref() != text("__proto__").as_ref()
            && key.as_ref() != text("__constructor__").as_ref()
        {
            // Grounded instance lookup resolves existing ordinary stored methods.
            // Missing native members still reject at the end of get_key.
            if !ordinary_movie_clip_name(key) {
                return self.fail(ErrorKind::Unsupported(
                    "unbound MovieClip native/display member",
                ));
            }
        }
        if matches!(self.objects[id.0].kind, Kind::FunctionPrototype)
            && key.as_ref() == text("constructor").as_ref()
        {
            return self.fail(ErrorKind::Unsupported(
                "Function prototype constructor descriptor",
            ));
        }
        if matches!(self.objects[id.0].kind, Kind::ConstructorSuper { .. }) {
            return self.fail(ErrorKind::Unsupported("super member lookup/assignment"));
        }
        Ok(())
    }
    fn array_parent(&self) -> Result<Option<ObjectId>> {
        if let Some(constructor) = self.array_constructor {
            if self.get(constructor, "prototype")?
                != self.array_prototype.map_or(Value::Undefined, Value::Object)
            {
                return self.fail(ErrorKind::Unsupported(
                    "modified native Array prototype binding",
                ));
            }
        }
        Ok(self.array_prototype)
    }
    fn allocate_constructed_object(
        &mut self,
        constructor: ObjectId,
        prototype: Option<ObjectId>,
        argc: usize,
    ) -> Result<ObjectId> {
        let mut cursor = prototype;
        let mut has_movie_clip = false;
        for depth in 0..=self.limits.prototype_depth {
            let Some(id) = cursor else { break };
            if depth == self.limits.prototype_depth {
                return self.fail(ErrorKind::Budget("constructor prototype depth"));
            }
            if matches!(self.objects[id.0].kind, Kind::MovieClip(_)) {
                has_movie_clip = true;
                break;
            }
            cursor = self.objects[id.0].prototype;
        }
        // Native selection consults public constructor, separate from super metadata.
        let selected = match prototype {
            Some(id) => self.get(id, "constructor")?,
            None => Value::Undefined,
        };
        let clip = self.movie_clip_constructor;
        let selects_clip = clip.is_some_and(|id| selected == Value::Object(id));
        if has_movie_clip || selects_clip || clip == Some(constructor) {
            let Some(clip) = clip else {
                return self.fail(ErrorKind::Unsupported("uninstalled MovieClip allocator"));
            };
            let clip_prototype = self.movie_clip_prototype.unwrap();
            if argc != 0
                || !selects_clip
                || self.get(clip, "prototype")? != Value::Object(clip_prototype)
                || self.get(clip_prototype, "constructor")? != Value::Object(clip)
                || self.objects[clip_prototype.0].prototype != self.movie_clip_parent
            {
                return self.fail(ErrorKind::Unsupported(
                    "modified/argument MovieClip allocation profile",
                ));
            }
            let mut class = constructor;
            let mut current_prototype = prototype;
            for depth in 0..=self.limits.prototype_depth {
                if class == clip {
                    if current_prototype != Some(clip_prototype) {
                        return self.fail(ErrorKind::Unsupported(
                            "modified MovieClip constructor prototype",
                        ));
                    }
                    let event_names = self.movie_clip_prototype_event_marker(prototype.unwrap())?;
                    let object =
                        self.alloc(Kind::MovieClip(MovieClipRole::UnboundInstance), prototype)?;
                    self.raw_member(
                        object,
                        text("__constructor__"),
                        Property::Data(Value::Object(constructor)),
                        3,
                    )?;
                    self.movie_clip_event_names.insert(object, event_names);
                    return Ok(object);
                }
                if depth == self.limits.prototype_depth {
                    return self.fail(ErrorKind::Budget("constructor inheritance depth"));
                }
                let Some(base) = self.class_bases.get(&class).copied() else {
                    return self.fail(ErrorKind::Unsupported(
                        "unrecorded MovieClip constructor inheritance",
                    ));
                };
                if current_prototype != Some(base.prototype)
                    || self.get(class, "prototype")? != Value::Object(base.prototype)
                    || self.objects[base.prototype.0].prototype != Some(base.parent_prototype)
                    || self.get(base.prototype, "__constructor__")?
                        != Value::Object(base.constructor)
                    || self.objects[base.prototype.0]
                        .properties
                        .contains_key(&text("constructor"))
                {
                    return self.fail(ErrorKind::Unsupported(
                        "modified MovieClip subclass allocation",
                    ));
                }
                class = base.constructor;
                current_prototype = Some(base.parent_prototype);
            }
            unreachable!();
        }
        if let Value::Object(selected) = selected {
            if matches!(self.objects[selected.0].kind, Kind::ArrayConstructor) {
                return self.fail(ErrorKind::Unsupported("Array subclass allocation profile"));
            }
        }
        self.alloc(Kind::Plain, prototype)
    }
    fn movie_clip_prototype_event_marker(&mut self, mut prototype: ObjectId) -> Result<bool> {
        if self.movie_clip_role(prototype)? == Some(MovieClipRole::UnboundPrototype) {
            // Native category11 prototype assignment skips the visitor. Fresh marker=0.
            return Ok(false);
        }
        // REA's complete native MovieClip/Object prototype inventories contain no
        // matching event names. Unimplemented baseline methods stay unreadable;
        // this scan includes every stored script-added key on the retained chain.
        let names = [
            "onPress",
            "onRelease",
            "onReleaseOutside",
            "onRollOver",
            "onRollOut",
            "onDragOver",
            "onDragOut",
            "onPressAux",
            "onReleaseAux",
            "onReleaseOutsideAux",
            "onDragOverAux",
            "onDragOutAux",
        ];
        let mut found = false;
        for _ in 0..self.limits.prototype_depth {
            if !matches!(
                self.objects[prototype.0].kind,
                Kind::Plain | Kind::MovieClip(MovieClipRole::UnboundPrototype)
            ) {
                return self.fail(ErrorKind::Unsupported(
                    "MovieClip prototype visitor object class",
                ));
            }
            // Own/inherited keys, including hidden entries; values/getters are never read.
            self.visit_members(self.objects[prototype.0].properties.len())?;
            found |= self.objects[prototype.0]
                .properties
                .keys()
                .any(|key| names.iter().any(|name| key.as_ref() == text(name).as_ref()));
            match self.objects[prototype.0].prototype {
                Some(parent) => prototype = parent,
                None => return Ok(found),
            }
        }
        self.fail(ErrorKind::Budget("MovieClip prototype visitor depth"))
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

    /// Installs only the native-backed, currently supported flag intrinsic.
    pub fn install_property_flags(&mut self) -> Result<ObjectId> {
        match self.get(self.global, "ASSetPropFlags")? {
            Value::Object(id) if matches!(self.objects[id.0].kind, Kind::SetPropertyFlags) => {
                return Ok(id);
            }
            Value::Undefined => {}
            _ => {
                return self.fail(ErrorKind::Unsupported(
                    "ASSetPropFlags binding already exists",
                ));
            }
        }
        let id = self.alloc(Kind::SetPropertyFlags, None)?;
        self.set(self.global, "ASSetPropFlags", Value::Object(id))?;
        Ok(id)
    }
    /// Own stored members only; no prototype lookup and no getter execution.
    pub fn own_property_flags(&self, id: ObjectId, key: &str) -> Result<Option<u8>> {
        self.check_object(id)?;
        let key = text(key);
        Ok(self.objects[id.0]
            .properties
            .contains_key(&key)
            .then(|| self.own_flags(id, &key)))
    }
    /// Inspection order is lexical, not a claim about native hash enumeration.
    pub fn own_enumerable_names(&self, id: ObjectId) -> Result<Vec<Text>> {
        self.check_object(id)?;
        Ok(self.objects[id.0]
            .properties
            .keys()
            .filter(|key| self.own_flags(id, key) & 1 == 0)
            .cloned()
            .collect())
    }
    fn own_flags(&self, id: ObjectId, key: &Text) -> u8 {
        self.objects[id.0].flags.get(key).copied().unwrap_or(0)
    }
    fn raw_member(&mut self, id: ObjectId, key: Text, value: Property, flags: u8) -> Result<()> {
        self.reserve_property(id, &key)?;
        self.objects[id.0].flags.insert(key.clone(), flags);
        self.objects[id.0].properties.insert(key, value);
        Ok(())
    }
    fn visit_members(&mut self, count: usize) -> Result<()> {
        if count > self.limits.member_visits.saturating_sub(self.member_visits) {
            return self.fail(ErrorKind::Budget("member visits"));
        }
        self.member_visits += count;
        Ok(())
    }
    fn flag_mask(&self, value: &Value) -> Result<u8> {
        let Value::Number(number) = value else {
            return self.fail(ErrorKind::Unsupported(
                "nonnumeric ASSetPropFlags mask profile",
            ));
        };
        if !number.is_finite()
            || number.fract() != 0.0
            || *number < f64::from(i32::MIN)
            || *number > f64::from(u32::MAX)
        {
            return self.fail(ErrorKind::Unsupported(
                "ASSetPropFlags numeric coercion profile",
            ));
        }
        Ok((uint32(*number) & 7) as u8)
    }
    fn apply_property_flags(&mut self, args: &[Value]) -> Result<Value> {
        if !(3..=4).contains(&args.len()) {
            return self.fail(ErrorKind::Unsupported(
                "ASSetPropFlags argument-count profile",
            ));
        }
        let target = self.object_id(args[0].clone())?;
        let prototype = matches!(
            self.objects[target.0].kind,
            Kind::MovieClip(MovieClipRole::UnboundPrototype)
        );
        if !matches!(self.objects[target.0].kind, Kind::Plain) && !prototype {
            return self.fail(ErrorKind::Unsupported("ASSetPropFlags object class"));
        }
        let keys: Vec<Text> = match &args[1] {
            Value::Null if !prototype => {
                self.visit_members(self.objects[target.0].properties.len().saturating_mul(2))?;
                self.objects[target.0].properties.keys().cloned().collect()
            }
            Value::String(selector) => {
                // Exact own ASCII identifiers and nonempty comma-separated lists.
                // No trimming, case folding, coercion, getter or prototype walk.
                let count = selector.split(|unit| *unit == u16::from(b',')).count();
                self.visit_members(count.saturating_mul(2))?;
                let mut keys = Vec::with_capacity(count);
                for part in selector.split(|unit| *unit == u16::from(b',')) {
                    if part.is_empty()
                        || !part.iter().all(|unit| {
                            u8::try_from(*unit).is_ok_and(|unit| {
                                unit.is_ascii_alphanumeric() || matches!(unit, b'_' | b'$')
                            })
                        })
                    {
                        return self.fail(ErrorKind::Unsupported(
                            "ASSetPropFlags ASCII selector profile",
                        ));
                    }
                    let key = self.allocate_text(part.to_vec())?;
                    if !self.objects[target.0].properties.contains_key(&key)
                        || (prototype
                            && (!ordinary_movie_clip_name(&key)
                                || key.as_ref() == text("constructor").as_ref()))
                    {
                        return self.fail(ErrorKind::Unsupported(
                            "ASSetPropFlags missing/inherited/native member",
                        ));
                    }
                    keys.push(key);
                }
                keys
            }
            _ => return self.fail(ErrorKind::Unsupported("ASSetPropFlags selector profile")),
        };
        let set = self.flag_mask(&args[2])?;
        let clear = args
            .get(3)
            .map(|value| self.flag_mask(value))
            .transpose()?
            .or_else(|| {
                self.current_target
                    .and_then(|target| self.loaded_movie_targets.get(&target))
                    .map(|version| if *version == 5 { 7 } else { 0 })
            });
        // REA connects the cache to the header only for validated loaded targets.
        // Unbound/native/direct-call contexts retain outcome-independent gating.
        if clear.is_none() {
            for key in &keys {
                let old = self.own_flags(target, key);
                if (old | set) != ((old & !7) | set) {
                    return self.fail(ErrorKind::AmbiguousPropertyFlags { old, set });
                }
            }
        }
        for key in keys {
            let old = self.own_flags(target, &key);
            self.objects[target.0]
                .flags
                .insert(key, (old & !clear.unwrap_or(0)) | set);
        }
        Ok(Value::Undefined)
    }
    fn enumerate_names(&mut self, mut id: ObjectId) -> Result<Vec<Text>> {
        let mut seen = BTreeMap::<Text, bool>::new();
        let mut names = Vec::new();
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            if !matches!(self.objects[id.0].kind, Kind::Plain) {
                return self.fail(ErrorKind::Unsupported(
                    "enumeration on non-generic object class",
                ));
            }
            self.visit_members(self.objects[id.0].properties.len())?;
            for key in self.objects[id.0].properties.keys() {
                let visible = self.own_flags(id, key) & 1 == 0;
                if let Some(prior) = seen.get(key) {
                    if *prior || visible {
                        return self.fail(ErrorKind::Unsupported(
                            "native enumeration shadow/duplicate ordering",
                        ));
                    }
                }
                seen.insert(key.clone(), visible);
                if visible {
                    names.push(key.clone());
                }
                if names.len() > 1 {
                    return self.fail(ErrorKind::Unsupported(
                        "native enumeration hash order with multiple names",
                    ));
                }
            }
            if let Some(parent) = self.objects[id.0].prototype {
                id = parent;
            } else {
                return Ok(names);
            }
        }
        self.fail(ErrorKind::Budget("prototype lookup depth"))
    }
    fn delete_member(&mut self, id: ObjectId, key: &Text) -> Result<bool> {
        self.check_object(id)?;
        if !matches!(self.objects[id.0].kind, Kind::Plain) {
            return self.fail(ErrorKind::Unsupported("delete on non-generic object class"));
        }
        if !self.objects[id.0].properties.contains_key(key) || self.own_flags(id, key) & 2 != 0 {
            return Ok(false);
        }
        if key.as_ref() == text("__proto__").as_ref() {
            return self.fail(ErrorKind::Unsupported(
                "deleting unprotected prototype metadata",
            ));
        }
        self.objects[id.0].properties.remove(key);
        self.objects[id.0].flags.remove(key);
        Ok(true)
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
        let mut native_member_context = false;
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            let object = &self.objects[id.0];
            self.check_member_target(id, key)?;
            native_member_context |= matches!(object.kind, Kind::MovieClip(_));
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
                if native_member_context {
                    return self.fail(ErrorKind::Unsupported(
                        "unimplemented MovieClip native member",
                    ));
                }
                return Ok(Value::Undefined);
            }
        }
        self.fail(ErrorKind::Budget("prototype lookup depth"))
    }
    fn has_key(&self, mut id: ObjectId, key: &Text) -> Result<bool> {
        let mut native_member_context = false;
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            let object = &self.objects[id.0];
            self.check_member_target(id, key)?;
            native_member_context |= matches!(object.kind, Kind::MovieClip(_));
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
                if native_member_context {
                    return self.fail(ErrorKind::Unsupported(
                        "unimplemented MovieClip native member",
                    ));
                }
                return Ok(false);
            }
        }
        self.fail(ErrorKind::Budget("prototype lookup depth"))
    }
    fn descriptor(&self, mut id: ObjectId, key: &Text) -> Result<Option<Property>> {
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            let object = &self.objects[id.0];
            if let Kind::Array(length) = object.kind {
                if key.as_ref() == text("length").as_ref() {
                    return Ok(Some(Property::Data(Value::Number(f64::from(length)))));
                }
            }
            self.check_member_target(id, key)?;
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
    fn property_owner(&self, mut id: ObjectId, key: &Text) -> Result<Option<ObjectId>> {
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            if matches!(self.objects[id.0].kind, Kind::Array(_))
                && key.as_ref() == text("length").as_ref()
            {
                return Ok(Some(id));
            }
            if self.objects[id.0].properties.contains_key(key) {
                return Ok(Some(id));
            }
            if let Some(parent) = self.objects[id.0].prototype {
                id = parent;
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
                        | Kind::ArrayConstructor
                        | Kind::FunctionConversion
                        | Kind::MovieClipConstructor
                        | Kind::AddProperty
                        | Kind::RegisterClass
                        | Kind::SetPropertyFlags
                        | Kind::FunctionMethod(_)
                )
            })
        } else {
            false
        }
    }
    fn read_member(&mut self, id: ObjectId, key: &Text, host: &mut impl Host) -> Result<Value> {
        if let Some(Property::Accessor { getter, .. }) = self.descriptor(id, key)? {
            if matches!(self.objects[id.0].kind, Kind::MovieClip(_)) {
                return self.fail(ErrorKind::Unsupported("MovieClip accessor read profile"));
            }
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
            if matches!(self.objects[id.0].kind, Kind::MovieClip(_)) {
                return self.fail(ErrorKind::Unsupported(
                    "MovieClip accessor assignment profile",
                ));
            }
            let owner = self.property_owner(id, &key)?.unwrap();
            if self.own_flags(owner, &key) & 4 != 0 {
                return self.fail(ErrorKind::Unsupported(
                    "flagged accessor assignment native ordering",
                ));
            }
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
        if let Kind::MovieClip(role) = self.objects[id.0].kind {
            if role != MovieClipRole::UnboundPrototype
                || !ordinary_movie_clip_name(&key)
                || key.as_ref() == text("constructor").as_ref()
            {
                return self.fail(ErrorKind::Unsupported(
                    "unbound MovieClip member assignment",
                ));
            }
        }
        self.check_member_target(id, &key)?;
        if self.array_prototype == Some(id)
            && (key.as_ref() == text("length").as_ref() || array_index(&key).is_some())
        {
            return self.fail(ErrorKind::Unsupported(
                "Array prototype indexed/length mutation semantics",
            ));
        }
        self.check_value(&value)?;
        if matches!(self.descriptor(id, &key)?, Some(Property::Accessor { .. })) {
            return self.fail(ErrorKind::Unsupported(
                "accessor requires executable property assignment",
            ));
        }
        if let Some(owner) = self.property_owner(id, &key)? {
            if self.own_flags(owner, &key) & 4 != 0 {
                if owner != id {
                    return self.fail(ErrorKind::Unsupported(
                        "inherited readonly assignment native ordering",
                    ));
                }
                if key.as_ref() == text("__proto__").as_ref() {
                    return self.fail(ErrorKind::Unsupported(
                        "readonly prototype metadata assignment",
                    ));
                }
                return Ok(());
            }
        }
        if key.as_ref() == text("__proto__").as_ref() {
            if self.objects[id.0].properties.contains_key(&key) && self.own_flags(id, &key) != 3 {
                return self.fail(ErrorKind::Unsupported(
                    "prototype assignment after metadata flags changed",
                ));
            }
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
            self.raw_member(
                id,
                key,
                Property::Data(prototype.map_or(Value::Null, Value::Object)),
                3,
            )?;
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
    fn movie_property(&self, value: Value) -> Result<MovieProperty> {
        let number = self.number(&value)?;
        if !number.is_finite() || number.fract() != 0.0 || !(0.0..=21.0).contains(&number) {
            return self.fail(ErrorKind::Unsupported("invalid indexed movie property"));
        }
        Ok(match number as u8 {
            0 => MovieProperty::X,
            1 => MovieProperty::Y,
            2 => MovieProperty::XScale,
            3 => MovieProperty::YScale,
            4 => MovieProperty::CurrentFrame,
            5 => MovieProperty::TotalFrames,
            6 => MovieProperty::Alpha,
            7 => MovieProperty::Visible,
            8 => MovieProperty::Width,
            9 => MovieProperty::Height,
            10 => MovieProperty::Rotation,
            11 => MovieProperty::Target,
            12 => MovieProperty::FramesLoaded,
            13 => MovieProperty::Name,
            14 => MovieProperty::DropTarget,
            15 => MovieProperty::Url,
            16 => MovieProperty::HighQuality,
            17 => MovieProperty::FocusRect,
            18 => MovieProperty::SoundBufferTime,
            19 => MovieProperty::Quality,
            20 => MovieProperty::MouseX,
            _ => MovieProperty::MouseY,
        })
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
        self.member_visits = 0;
        self.depth = 0;
        self.current = None;
        self.current_code = None;
        self.current_target = None;
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
        self.invoke_in_context(callable, this, args, host, None)
    }
    fn invoke_in_context(
        &mut self,
        callable: Value,
        this: Value,
        args: &[Value],
        host: &mut impl Host,
        constructor_context: Option<ObjectId>,
    ) -> Result<Value> {
        if self.depth >= self.limits.call_depth {
            return self.fail(ErrorKind::Budget("function call depth"));
        }
        let caller = (self.current, self.current_code, self.current_target);
        self.depth += 1;
        let result = self.invoke_inner(callable, this, args, host, constructor_context);
        self.depth -= 1;
        self.current = caller.0;
        self.current_code = caller.1;
        self.current_target = caller.2;
        result
    }
    fn invoke_inner(
        &mut self,
        callable: Value,
        this: Value,
        args: &[Value],
        host: &mut impl Host,
        constructor_context: Option<ObjectId>,
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
            Kind::ConstructorSuper {
                constructor,
                receiver,
            } => {
                return self.invoke_in_context(
                    Value::Object(constructor),
                    Value::Object(receiver),
                    args,
                    host,
                    Some(constructor),
                );
            }
            Kind::FunctionPrototype => {
                return self.fail(ErrorKind::Unsupported(
                    "Function prototype invocation semantics",
                ));
            }
            Kind::Native(binding) => {
                self.host_budget()?;
                let value = host
                    .call(HostCall {
                        origin: self.origin(),
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
                            None | Some(Value::Null) | Some(Value::Undefined) => Vec::new(),
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
                                    "Function.apply primitive argument-array profile",
                                ));
                            }
                        }
                    }
                };
                return self.invoke(this, Value::Object(*receiver), &forwarded, host);
            }
            Kind::SetPropertyFlags => return self.apply_property_flags(args),
            Kind::RegisterClass => {
                if args.len() != 2 {
                    return self.fail(ErrorKind::Unsupported(
                        "Object.registerClass argument-count profile",
                    ));
                }
                let Value::String(name) = &args[0] else {
                    return self.fail(ErrorKind::Unsupported(
                        "Object.registerClass nonstring name",
                    ));
                };
                if name.is_empty() {
                    return self.fail(ErrorKind::Unsupported(
                        "Object.registerClass empty-name profile",
                    ));
                }
                let constructor = match &args[1] {
                    Value::Null => None,
                    value @ Value::Object(id) if self.callable(value) => Some(*id),
                    _ => {
                        return self.fail(ErrorKind::Unsupported(
                            "Object.registerClass requires function or explicit null",
                        ));
                    }
                };
                let constructor_origin =
                    constructor.and_then(|id| match &self.objects[id.0].kind {
                        Kind::Script(function) => Some(CodeOrigin {
                            code: function.code,
                            source_offset: function.offset,
                        }),
                        _ => None,
                    });
                let registration = ClassRegistration {
                    name: name.clone(),
                    constructor,
                    origin: self.origin(),
                    constructor_origin,
                };
                self.host_budget()?;
                let accepted = host
                    .register_class(registration)
                    .map_err(|message| self.error(ErrorKind::Host(message)))?;
                return Ok(Value::Bool(accepted));
            }
            Kind::AddProperty => {
                if args.iter().skip(1).any(|value| {
                    matches!(value, Value::Object(id)
                    if matches!(self.objects[id.0].kind, Kind::FunctionPrototype | Kind::ConstructorSuper { .. }))
                }) {
                    return self.fail(ErrorKind::Unsupported(
                        "Function prototype accessor callability",
                    ));
                }
                if args.len() != 3 {
                    return self.fail(ErrorKind::Unsupported("addProperty argument count"));
                }
                let (Value::Object(receiver), Value::String(name)) = (&this, &args[0]) else {
                    return Ok(Value::Bool(false));
                };
                if matches!(self.objects[receiver.0].kind, Kind::MovieClip(_)) {
                    return self.fail(ErrorKind::Unsupported("MovieClip addProperty profile"));
                }
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
                if self.own_flags(*receiver, name) != 0 {
                    return self.fail(ErrorKind::Unsupported(
                        "addProperty replacement of flagged member",
                    ));
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
            Kind::MovieClipConstructor => {
                // The native callback only tests category and returns that receiver;
                // allocation is a distinct NewObject/NewMethod operation above.
                return Ok(match this {
                    Value::Object(receiver)
                        if matches!(self.objects[receiver.0].kind, Kind::MovieClip(_)) =>
                    {
                        Value::Object(receiver)
                    }
                    _ => Value::Undefined,
                });
            }
            Kind::FunctionConversion => {
                if constructor_context.is_some() || args.len() != 1 {
                    return self.fail(ErrorKind::Unsupported(
                        "Function conversion arity/construction profile",
                    ));
                }
                if let Value::Object(argument) = args[0] {
                    if matches!(self.objects[argument.0].kind, Kind::ConstructorSuper { .. }) {
                        return self.fail(ErrorKind::Unsupported(
                            "Function conversion of super reference",
                        ));
                    }
                    if matches!(self.objects[argument.0].kind, Kind::Script(_)) {
                        return self.fail(ErrorKind::Unsupported(
                            "Function conversion ScriptFunction reference profile",
                        ));
                    }
                }
                return Ok(if self.callable(&args[0]) {
                    args[0].clone()
                } else {
                    Value::Null
                });
            }
            Kind::ArrayConstructor => {
                if constructor_context.is_some() {
                    return self.fail(ErrorKind::Unsupported(
                        "Array superclass receiver initialization",
                    ));
                }
                if let [Value::Number(length)] = args {
                    if !length.is_finite() || *length < 0.0 || length.fract() != 0.0 {
                        return self.fail(ErrorKind::Unsupported(
                            "Array numeric length conversion profile",
                        ));
                    }
                    if *length > f64::from(self.limits.array_length) {
                        return self.fail(ErrorKind::Budget("array length"));
                    }
                    return Ok(Value::Object(
                        self.alloc(Kind::Array(*length as u32), self.array_parent()?)?,
                    ));
                }
                return Ok(Value::Object(self.array(args)?));
            }
            Kind::ObjectConstructor => {
                if constructor_context.is_some() {
                    return self.fail(ErrorKind::Unsupported(
                        "Object superclass receiver initialization",
                    ));
                }
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
        if flags & 0x80 != 0 {
            return self.fail(ErrorKind::Unsupported("parent register preload"));
        }
        let super_reference = if constructor_context == Some(id)
            && self.class_bases.contains_key(&id)
            && flags & 0x20 == 0
        {
            Some(self.constructor_super(id, self.object_id(this.clone())?)?)
        } else {
            None
        };
        if flags & 0x10 != 0 && super_reference.is_none() {
            return self.fail(ErrorKind::Unsupported(
                "super preload requires recorded constructor context",
            ));
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
        if let Some(super_reference) = super_reference {
            self.set(activation, "super", Value::Object(super_reference))?;
        }
        let mut register = 1u8;
        for (mask, value) in [
            (1, this),
            (4, arguments.unwrap_or(Value::Undefined)),
            (
                0x10,
                super_reference.map_or(Value::Undefined, Value::Object),
            ),
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
    fn constructor_super(&mut self, class: ObjectId, receiver: ObjectId) -> Result<ObjectId> {
        let base = self.class_bases[&class];
        if self.get(class, "prototype")? != Value::Object(base.prototype)
            || self.get(base.prototype, "__constructor__")? != Value::Object(base.constructor)
            || self.objects[base.prototype.0].prototype != Some(base.parent_prototype)
            || self.get(base.constructor, "prototype")? != Value::Object(base.parent_prototype)
        {
            return self.fail(ErrorKind::Unsupported(
                "modified constructor inheritance for super",
            ));
        }
        let mut cursor = self.objects[receiver.0].prototype;
        let mut found = false;
        for _ in 0..self.limits.prototype_depth {
            let Some(id) = cursor else {
                break;
            };
            if id == base.prototype {
                found = true;
                break;
            }
            cursor = self.objects[id.0].prototype;
        }
        if !found {
            return self.fail(ErrorKind::Unsupported(
                "super receiver outside constructor prototype chain",
            ));
        }
        self.alloc(
            Kind::ConstructorSuper {
                constructor: base.constructor,
                receiver,
            },
            None,
        )
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
        let id = self.alloc(Kind::Array(len), self.array_parent()?)?;
        for (i, value) in values.iter().enumerate() {
            self.check_value(value)?;
            self.raw_member(id, text(&i.to_string()), Property::Data(value.clone()), 0)?;
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
        if matches!(a, Value::Null | Value::Undefined)
            || matches!(b, Value::Null | Value::Undefined)
        {
            return Ok(false);
        }
        if matches!(a, Value::Object(_)) || matches!(b, Value::Object(_)) {
            return self.fail(ErrorKind::Unsupported("object abstract equality coercion"));
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
            self.current_code = Some(code);
            self.current_target = Some(frame.target);
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
                        if matches!(self.objects[constructor.0].kind, Kind::FunctionConversion) {
                            return self.fail(ErrorKind::Unsupported(
                                "Function dynamic construction profile",
                            ));
                        }
                        let prototype = match self.get(constructor, "prototype")? {
                            Value::Object(id) => Some(id),
                            Value::Undefined | Value::Null => None,
                            _ => {
                                return self.fail(ErrorKind::Unsupported(
                                    "primitive constructor prototype",
                                ));
                            }
                        };
                        if matches!(
                            self.objects[constructor.0].kind,
                            Kind::ObjectConstructor | Kind::ArrayConstructor
                        ) {
                            // Native standard constructors select their constructed object;
                            // this is distinct from a script constructor's return value.
                            let object =
                                self.invoke(callable, Value::Object(frame.target), &args, host)?;
                            self.object_id(object.clone())?;
                            self.push(frame, object)?;
                        } else {
                            let object = self.allocate_constructed_object(
                                constructor,
                                prototype,
                                args.len(),
                            )?;
                            self.invoke_in_context(
                                callable,
                                Value::Object(object),
                                &args,
                                host,
                                Some(constructor),
                            )?;
                            // Adobe NewObject/NewMethod discard script constructor returns.
                            self.push(frame, Value::Object(object))?;
                        }
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
                                    | Kind::ArrayConstructor
                                    | Kind::FunctionConversion
                                    | Kind::MovieClipConstructor
                                    | Kind::AddProperty
                                    | Kind::RegisterClass
                                    | Kind::SetPropertyFlags
                                    | Kind::FunctionMethod(_)
                            ) {
                                return self.fail(ErrorKind::UnboundCall);
                            }
                        }
                        let parent = self.object_id(self.get(superclass, "prototype")?)?;
                        let prototype = self.alloc(Kind::Plain, Some(parent))?;
                        self.raw_member(
                            prototype,
                            text("__constructor__"),
                            Property::Data(Value::Object(superclass)),
                            3,
                        )?;
                        self.set(subclass, "prototype", Value::Object(prototype))?;
                        self.class_bases.insert(
                            subclass,
                            ClassBase {
                                constructor: superclass,
                                prototype,
                                parent_prototype: parent,
                            },
                        );
                    }
                    0x46 | 0x55 => {
                        let target = self.pop(frame)?;
                        let target = if instruction.opcode == 0x46 {
                            let name = self.to_text(&target)?;
                            self.variable(frame, &name, host)?
                        } else {
                            target
                        };
                        let target = self.object_id(target)?;
                        let names = self.enumerate_names(target)?;
                        if names.len().saturating_add(1)
                            > self.limits.stack_values.saturating_sub(frame.stack.len())
                        {
                            return self.fail(ErrorKind::Budget("operand stack"));
                        }
                        self.push(frame, Value::Null)?;
                        for name in names {
                            self.push(frame, Value::String(name))?;
                        }
                    }
                    0x3a => {
                        let key = self.pop(frame)?;
                        let key = self.to_text(&key)?;
                        let target = self.object_id(self.pop(frame)?)?;
                        let result = self.delete_member(target, &key)?;
                        self.push(frame, Value::Bool(result))?;
                    }
                    0x3e => return Ok(Flow::Return(self.pop(frame)?)),
                    0x22 => {
                        let property = self.movie_property(self.pop(frame)?)?;
                        let target = self.pop(frame)?;
                        self.host_budget()?;
                        let value = host
                            .get_property(PropertyRequest {
                                origin: Some(CodeOrigin {
                                    code,
                                    source_offset: instruction.span.offset,
                                }),
                                context: frame.target,
                                target,
                                property,
                                source_offset: Some(instruction.span.offset),
                            })
                            .map_err(|message| self.error(ErrorKind::Host(message)))?;
                        self.check_value(&value)?;
                        self.push(frame, value)?;
                    }
                    0x23 => {
                        let value = self.pop(frame)?;
                        let property = self.movie_property(self.pop(frame)?)?;
                        let target = self.pop(frame)?;
                        self.host_budget()?;
                        host.set_property(
                            PropertyRequest {
                                origin: Some(CodeOrigin {
                                    code,
                                    source_offset: instruction.span.offset,
                                }),
                                context: frame.target,
                                target,
                                property,
                                source_offset: Some(instruction.span.offset),
                            },
                            value,
                        )
                        .map_err(|message| self.error(ErrorKind::Host(message)))?;
                    }
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
                                    Kind::FunctionPrototype
                                        | Kind::ConstructorSuper { .. }
                                        | Kind::MovieClip(_)
                                ) {
                                    return self.fail(ErrorKind::Unsupported(
                                        "special prototype/super typeof semantics",
                                    ));
                                }
                                if matches!(
                                    self.objects[id.0].kind,
                                    Kind::Script(_)
                                        | Kind::Native(_)
                                        | Kind::ObjectConstructor
                                        | Kind::ArrayConstructor
                                        | Kind::FunctionConversion
                                        | Kind::MovieClipConstructor
                                        | Kind::AddProperty
                                        | Kind::RegisterClass
                                        | Kind::SetPropertyFlags
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

fn ordinary_movie_clip_name(key: &Text) -> bool {
    !key.is_empty()
        && key[0] != u16::from(b'_')
        && !key.starts_with(&[u16::from(b'o'), u16::from(b'n')])
        && key.iter().all(|v| {
            u8::try_from(*v).is_ok_and(|v| v.is_ascii_alphanumeric() || matches!(v, b'_' | b'$'))
        })
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
        let object = Value::Object(vm.object().unwrap());
        assert!(!vm.equal(&object, &Value::Null).unwrap());
        assert!(!vm.equal(&Value::Undefined, &object).unwrap());
        assert!(vm.equal(&object, &Value::Number(0.0)).is_err());
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
    fn array_constructor_creation_forms_and_prototype_limits_are_explicit() {
        let mut vm = setup();
        assert!(vm.install_array_constructor().is_err());
        vm.install_object_constructor().unwrap();
        let constructor = vm.install_array_constructor().unwrap();
        let root = vm.root();
        let create = |vm: &mut Vm, args: &[Value]| {
            let Value::Object(id) = vm
                .call(
                    Value::Object(constructor),
                    Value::Object(root),
                    args,
                    &mut RejectHost,
                )
                .unwrap()
                .value
            else {
                panic!()
            };
            id
        };
        let empty = create(&mut vm, &[]);
        assert_eq!(vm.get(empty, "length").unwrap(), Value::Number(0.0));
        let sized = create(&mut vm, &[Value::Number(3.0)]);
        assert_eq!(
            vm.read_property(sized, "length", &mut RejectHost)
                .unwrap()
                .value,
            Value::Number(3.0)
        );
        let values = create(&mut vm, &[Value::Number(2.0), Value::Number(5.0)]);
        assert_eq!(vm.get(values, "length").unwrap(), Value::Number(2.0));
        assert_eq!(vm.get(values, "1").unwrap(), Value::Number(5.0));
        let nested = create(&mut vm, &[Value::Object(values)]);
        assert_ne!(nested, values);
        assert_eq!(vm.get(nested, "0").unwrap(), Value::Object(values));
        let string = create(&mut vm, &[Value::string("3")]);
        assert_eq!(vm.get(string, "length").unwrap(), Value::Number(1.0));
        let Value::Object(prototype) = vm.get(constructor, "prototype").unwrap() else {
            panic!()
        };
        assert_eq!(
            vm.get(values, "__proto__").unwrap(),
            Value::Object(prototype)
        );
        assert_eq!(vm.get(prototype, "length").unwrap(), Value::Number(0.0));
        assert_eq!(vm.get(sized, "0").unwrap(), Value::Undefined);
        assert_eq!(vm.get(prototype, "0").unwrap(), Value::Undefined);
        assert!(vm.set(prototype, "0", Value::Null).is_err());
        assert!(
            vm.call(
                Value::Object(constructor),
                Value::Object(root),
                &[Value::Number(-1.0)],
                &mut RejectHost
            )
            .is_err()
        );
        assert!(
            vm.call(
                Value::Object(constructor),
                Value::Object(root),
                &[Value::Number(1.25)],
                &mut RejectHost
            )
            .is_err()
        );
        let huge = vm
            .call(
                Value::Object(constructor),
                Value::Object(root),
                &[Value::Number(1_000_001.0)],
                &mut RejectHost,
            )
            .unwrap_err();
        assert_eq!(huge.kind, ErrorKind::Budget("array length"));
        let mut bytes = push(&[Value::Number(0.0), Value::string("Array")]);
        bytes.push(0x40);
        let result = execute(&mut vm, &bytes).unwrap();
        let Value::Object(new) = result.stack[0] else {
            panic!()
        };
        assert_eq!(vm.get(new, "length").unwrap(), Value::Number(0.0));
    }
    #[test]
    fn nested_constructor_super_keeps_receiver_and_new_discards_script_return() {
        struct ConstructorHost(Vec<ObjectId>);
        impl Host for ConstructorHost {
            fn call(&mut self, call: HostCall<'_>) -> std::result::Result<Value, String> {
                assert_eq!(call.binding, 77);
                assert!(call.args.is_empty());
                let Value::Object(receiver) = call.this else {
                    panic!()
                };
                self.0.push(receiver);
                Ok(Value::Number(7.0))
            }
            fn command(&mut self, _: ObjectId, _: HostCommand) -> std::result::Result<(), String> {
                Err("unused".into())
            }
        }
        fn definition(name: &str, body: &[u8]) -> Vec<u8> {
            let mut payload = name.as_bytes().to_vec();
            payload.extend([0, 0, 0, 2, 0x19, 0]);
            payload.extend((body.len() as u16).to_le_bytes());
            let mut bytes = action(0x8e, &payload);
            bytes.extend(body);
            bytes
        }
        fn super_call() -> Vec<u8> {
            let mut bytes = push(&[Value::Number(0.0)]);
            bytes.extend(action(0x96, &[4, 2, 3]));
            bytes.push(0x52);
            bytes
        }
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        let root = vm.root();
        let base = vm.native(77).unwrap();
        let base_proto = vm.object().unwrap();
        vm.set(base, "prototype", Value::Object(base_proto))
            .unwrap();
        vm.set(root, "Base", Value::Object(base)).unwrap();
        let replacement = vm.object().unwrap();
        vm.set(root, "replacement", Value::Object(replacement))
            .unwrap();
        let mut b = push(&[Value::string("this")]);
        b.push(0x1c);
        b.extend(push(&[Value::string("baseResult")]));
        b.extend(super_call());
        b.push(0x4f);
        b.extend(push(&[Value::string("this")]));
        b.push(0x1c);
        b.extend(push(&[Value::string("b"), Value::Bool(true)]));
        b.push(0x4f);
        let mut c = super_call();
        c.push(0x17);
        c.extend(push(&[Value::string("this")]));
        c.push(0x1c);
        c.extend(push(&[Value::string("c"), Value::Bool(true)]));
        c.push(0x4f);
        c.extend(push(&[Value::string("replacement")]));
        c.extend([0x1c, 0x3e]);
        let mut setup_code = definition("B", &b);
        setup_code.extend(definition("C", &c));
        for (child, parent) in [("B", "Base"), ("C", "B")] {
            setup_code.extend(push(&[Value::string(child)]));
            setup_code.push(0x1c);
            setup_code.extend(push(&[Value::string(parent)]));
            setup_code.extend([0x1c, 0x69]);
        }
        execute(&mut vm, &setup_code).unwrap();
        let constructor = vm.get(root, "C").unwrap();
        let namespace = vm.object().unwrap();
        vm.set(namespace, "C", constructor.clone()).unwrap();
        vm.set(root, "space", Value::Object(namespace)).unwrap();
        let mut host = ConstructorHost(Vec::new());
        for opcode in [0x40, 0x53] {
            let mut bytes = push(&[Value::Number(0.0)]);
            if opcode == 0x53 {
                bytes.extend(push(&[Value::string("space")]));
                bytes.push(0x1c);
            }
            bytes.extend(push(&[Value::string("C")]));
            bytes.push(opcode);
            let code = vm
                .register(Arc::new(Program::parse(&bytes).unwrap()))
                .unwrap();
            let result = vm.execute(code, root, &mut host).unwrap();
            let Value::Object(instance) = result.stack[0] else {
                panic!()
            };
            assert_ne!(instance, replacement);
            assert_eq!(host.0.last(), Some(&instance));
            assert_eq!(vm.get(instance, "baseResult").unwrap(), Value::Number(7.0));
            assert_eq!(vm.get(instance, "b").unwrap(), Value::Bool(true));
            assert_eq!(vm.get(instance, "c").unwrap(), Value::Bool(true));
        }
        assert_ne!(host.0[0], host.0[1]);
        assert!(matches!(
            vm.call(constructor, Value::Object(replacement), &[], &mut host)
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported(_)
        ));
        let Value::Object(b) = vm.get(root, "B").unwrap() else {
            panic!()
        };
        let Value::Object(proto) = vm.get(b, "prototype").unwrap() else {
            panic!()
        };
        vm.set(proto, "__constructor__", Value::Null).unwrap();
        let mut bytes = push(&[Value::Number(0.0), Value::string("C")]);
        bytes.push(0x40);
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        assert!(matches!(
            vm.execute(code, root, &mut host).unwrap_err().kind,
            ErrorKind::Unsupported(_)
        ));
        assert_eq!(host.0.len(), 2);
    }
    #[test]
    fn movie_clip_allocation_keeps_typed_receiver_and_public_constructor_selection() {
        let mut vm = setup();
        assert!(vm.install_movie_clip_constructor().is_err());
        vm.install_object_constructor().unwrap();
        let movie_clip = vm.install_movie_clip_constructor().unwrap();
        assert_eq!(vm.install_movie_clip_constructor().unwrap(), movie_clip);
        let Value::Object(prototype) = vm.get(movie_clip, "prototype").unwrap() else {
            panic!()
        };
        assert_eq!(
            vm.movie_clip_role(prototype).unwrap(),
            Some(MovieClipRole::UnboundPrototype)
        );
        assert_eq!(
            vm.own_property_flags(prototype, "constructor").unwrap(),
            Some(3)
        );
        let root = vm.root();
        assert_eq!(
            vm.call(
                Value::Object(movie_clip),
                Value::Object(root),
                &[],
                &mut RejectHost
            )
            .unwrap()
            .value,
            Value::Undefined
        );
        assert_eq!(
            vm.call(
                Value::Object(movie_clip),
                Value::Object(prototype),
                &[],
                &mut RejectHost
            )
            .unwrap()
            .value,
            Value::Object(prototype)
        );
        let mut body = push(&[Value::Number(0.0)]);
        body.extend(action(0x96, &[4, 2, 3])); // zero args, super register2, undefined name
        body.extend([0x52, 0x3e]);
        let mut payload = b"Tween\0".to_vec();
        payload.extend([0, 0, 2, 0x19, 0]);
        payload.extend((body.len() as u16).to_le_bytes());
        let mut definition = action(0x8e, &payload);
        definition.extend(body);
        definition.extend(push(&[Value::string("Tween")]));
        definition.push(0x1c);
        definition.extend(push(&[Value::string("MovieClip")]));
        definition.extend([0x1c, 0x69]);
        execute(&mut vm, &definition).unwrap();
        let Value::Object(tween) = vm.get(root, "Tween").unwrap() else {
            panic!()
        };
        let Value::Object(tween_proto) = vm.get(tween, "prototype").unwrap() else {
            panic!()
        };
        assert_eq!(
            vm.get(tween_proto, "constructor").unwrap(),
            Value::Object(movie_clip)
        );
        assert_eq!(
            vm.get(tween_proto, "__constructor__").unwrap(),
            Value::Object(movie_clip)
        );
        for name in ["MovieClip", "Tween"] {
            let mut code = push(&[Value::Number(0.0), Value::string(name)]);
            code.push(0x40);
            let result = execute(&mut vm, &code).unwrap();
            assert_eq!(result.host_calls, 0);
            let Value::Object(instance) = result.stack[0] else {
                panic!()
            };
            assert_eq!(
                vm.movie_clip_role(instance).unwrap(),
                Some(MovieClipRole::UnboundInstance)
            );
            assert_eq!(
                vm.movie_clip_has_mouse_event_names(instance).unwrap(),
                Some(false)
            );
            let expected = if name == "Tween" { tween } else { movie_clip };
            assert_eq!(
                vm.get(instance, "__constructor__").unwrap(),
                Value::Object(expected)
            );
            assert_eq!(
                vm.own_property_flags(instance, "__constructor__").unwrap(),
                Some(3)
            );
            assert_eq!(
                vm.call(
                    Value::Object(movie_clip),
                    Value::Object(instance),
                    &[],
                    &mut RejectHost
                )
                .unwrap()
                .value,
                Value::Object(instance)
            );
            assert!(vm.get(instance, "_x").is_err());
            assert!(vm.has_key(instance, &text("missingNativeMember")).is_err());
            assert!(vm.set(instance, "ordinary", Value::Bool(true)).is_err());
        }
        let mut wrong_args = push(&[
            Value::Number(1.0),
            Value::Number(1.0),
            Value::string("Tween"),
        ]);
        wrong_args.push(0x40);
        assert!(execute(&mut vm, &wrong_args).is_err());
        // The allocated receiver retains its computed marker across the pure super call.
        vm.raw_member(
            tween_proto,
            text("onPress"),
            Property::Data(Value::Undefined),
            1,
        )
        .unwrap();
        let mut marked_new = push(&[Value::Number(0.0), Value::string("Tween")]);
        marked_new.push(0x40);
        let marked = execute(&mut vm, &marked_new).unwrap();
        let Value::Object(marked) = marked.stack[0] else {
            panic!()
        };
        assert_eq!(
            vm.movie_clip_has_mouse_event_names(marked).unwrap(),
            Some(true)
        );
        // A public rewrite controls allocation; hidden superclass alone must not grant type11.
        vm.set(tween_proto, "constructor", Value::Object(tween))
            .unwrap();
        let before = vm.object_count();
        let mut new = push(&[Value::Number(0.0), Value::string("Tween")]);
        new.push(0x40);
        assert!(matches!(
            execute(&mut vm, &new).unwrap_err().kind,
            ErrorKind::Unsupported(_)
        ));
        assert_eq!(vm.object_count(), before);
        assert_eq!(
            vm.get(tween_proto, "__constructor__").unwrap(),
            Value::Object(movie_clip)
        );
    }
    #[test]
    fn movie_clip_prototype_marker_visits_hidden_inherited_names_without_values() {
        let mut vm = setup();
        let object = vm.install_object_constructor().unwrap();
        let clip = vm.install_movie_clip_constructor().unwrap();
        let Value::Object(clip_proto) = vm.get(clip, "prototype").unwrap() else {
            panic!()
        };
        let Value::Object(object_proto) = vm.get(object, "prototype").unwrap() else {
            panic!()
        };
        let generic = vm.alloc(Kind::Plain, Some(clip_proto)).unwrap();
        assert!(!vm.movie_clip_prototype_event_marker(generic).unwrap());
        let getter = vm.native(99).unwrap();
        vm.raw_member(
            object_proto,
            text("onPress"),
            Property::Accessor {
                getter: Value::Object(getter),
                setter: None,
            },
            1,
        )
        .unwrap();
        vm.set(generic, "onPress", Value::Null).unwrap_err(); // inherited accessor requires executable write
        assert!(vm.movie_clip_prototype_event_marker(generic).unwrap());
        // Assigning a category11 prototype skips visitation even with event-named ancestors.
        assert!(!vm.movie_clip_prototype_event_marker(clip_proto).unwrap());
        assert_eq!(vm.host_calls, 0);
        vm.limits.member_visits = 0;
        assert!(matches!(
            vm.movie_clip_prototype_event_marker(generic)
                .unwrap_err()
                .kind,
            ErrorKind::Budget(_)
        ));
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
        let Value::Object(function_prototype) = vm.get(object, "__proto__").unwrap() else {
            panic!()
        };
        assert_eq!(
            vm.get(function_prototype, "__proto__").unwrap(),
            vm.get(object, "prototype").unwrap()
        );
        assert!(matches!(
            vm.call(
                Value::Object(function_prototype),
                Value::Object(root),
                &[],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::Unsupported(_)
        ));
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
        assert_eq!(
            vm.get(sub, "__proto__").unwrap(),
            Value::Object(function_prototype)
        );
        assert_eq!(
            vm.get(sub, "addProperty").unwrap(),
            vm.get(function_prototype, "addProperty").unwrap()
        );
        assert_eq!(vm.own_property_flags(sub, "addProperty").unwrap(), None);
        assert_eq!(vm.get(vm.global(), "Function").unwrap(), Value::Undefined);
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
                &[Value::Object(receiver), Value::Number(0.0)],
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
    fn function_conversion_gates_unproved_script_reference_and_preserves_empty_apply_lists() {
        let mut vm = setup();
        assert!(vm.install_function_conversion().is_err());
        vm.install_object_constructor().unwrap();
        vm.install_array_constructor().unwrap();
        let conversion = vm.install_function_conversion().unwrap();
        assert_eq!(vm.install_function_conversion().unwrap(), conversion);
        let root = vm.root();
        let mut body = push(&[Value::string("this")]);
        body.extend([0x1c, 0x3e]);
        execute(&mut vm, &function("f", &body)).unwrap();
        let f = vm.get(root, "f").unwrap();
        assert!(matches!(
            vm.call(
                Value::Object(conversion),
                Value::Object(root),
                &[f.clone()],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::Unsupported("Function conversion ScriptFunction reference profile")
        ));
        let Value::Object(f_id) = f else { panic!() };
        assert_eq!(
            vm.get(conversion, "prototype").unwrap(),
            vm.get(f_id, "__proto__").unwrap()
        );
        let receiver = vm.object().unwrap();
        let apply = vm.get(f_id, "apply").unwrap();
        for list in [Value::Null, Value::Undefined] {
            let r = vm
                .call(
                    apply.clone(),
                    Value::Object(f_id),
                    &[Value::Object(receiver), list],
                    &mut RejectHost,
                )
                .unwrap();
            assert_eq!(r.value, Value::Object(receiver));
            assert_eq!(r.host_calls, 0);
        }
        let native = vm.native(12).unwrap();
        assert_eq!(
            vm.call(
                Value::Object(conversion),
                Value::Object(root),
                &[Value::Object(native)],
                &mut RejectHost
            )
            .unwrap()
            .value,
            Value::Object(native)
        );
        for value in [
            Value::Null,
            Value::Undefined,
            Value::Bool(true),
            Value::Number(1.0),
            Value::string("function source"),
            Value::Object(receiver),
        ] {
            assert_eq!(
                vm.call(
                    Value::Object(conversion),
                    Value::Object(root),
                    &[value],
                    &mut RejectHost
                )
                .unwrap()
                .value,
                Value::Null
            );
        }
        assert!(
            vm.call(
                Value::Object(conversion),
                Value::Object(root),
                &[],
                &mut RejectHost
            )
            .is_err()
        );
        let mut nested = function("", &[0]);
        nested.push(0x3e);
        execute(&mut vm, &function("outer", &nested)).unwrap();
        let outer = vm.get(root, "outer").unwrap();
        let closure = vm
            .call(outer, Value::Object(root), &[], &mut RejectHost)
            .unwrap()
            .value;
        assert!(matches!(
            vm.call(
                Value::Object(conversion),
                Value::Object(root),
                &[closure],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::Unsupported(_)
        ));
        let mut new = push(&[Value::Number(0.0), Value::string("Function")]);
        new.push(0x40);
        assert!(matches!(
            execute(&mut vm, &new).unwrap_err().kind,
            ErrorKind::Unsupported(_)
        ));
    }
    #[test]
    fn native_observed_empty_pop_is_scoped_to_action_pop() {
        let result = execute(&mut setup(), &[0x17, 0x17, 0]).unwrap();
        assert!(result.stack.is_empty());
        assert_eq!(result.instructions, 3);
        let error = execute(&mut setup(), &[0x1c]).unwrap_err();
        assert_eq!(error.kind, ErrorKind::StackUnderflow);
    }
    struct InvalidPropertyHost;
    impl Host for InvalidPropertyHost {
        fn call(&mut self, _: HostCall<'_>) -> std::result::Result<Value, String> {
            Err("unbound".into())
        }
        fn command(&mut self, _: ObjectId, _: HostCommand) -> std::result::Result<(), String> {
            Err("unbound".into())
        }
        fn get_property(&mut self, request: PropertyRequest) -> std::result::Result<Value, String> {
            assert_eq!(request.property, MovieProperty::X);
            assert!(request.source_offset.is_some());
            Ok(Value::Object(ObjectId(usize::MAX)))
        }
    }
    #[test]
    fn movie_property_boundary_rejects_missing_bindings_bad_indices_and_ids() {
        let mut bytes = push(&[Value::string(""), Value::Number(0.0)]);
        bytes.push(0x22);
        assert!(matches!(
            execute(&mut setup(), &bytes).unwrap_err().kind,
            ErrorKind::Host(_)
        ));
        let mut vm = setup();
        let id = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        assert_eq!(
            vm.execute(id, vm.root(), &mut InvalidPropertyHost)
                .unwrap_err()
                .kind,
            ErrorKind::InvalidObject(ObjectId(usize::MAX))
        );
        let mut vm = Vm::new(
            15,
            Limits {
                host_calls: 0,
                ..Limits::default()
            },
        )
        .unwrap();
        assert_eq!(
            execute(&mut vm, &bytes).unwrap_err().kind,
            ErrorKind::Budget("host calls")
        );
        let mut bad = push(&[Value::string(""), Value::Number(22.0)]);
        bad.push(0x22);
        let mut vm = setup();
        assert!(matches!(
            execute(&mut vm, &bad).unwrap_err().kind,
            ErrorKind::Unsupported(_)
        ));
        assert_eq!(vm.host_calls, 0);
    }
    #[derive(Default)]
    struct RegistryHost {
        classes: BTreeMap<Text, ObjectId>,
        requests: Vec<ClassRegistration>,
    }
    impl Host for RegistryHost {
        fn call(&mut self, _: HostCall<'_>) -> std::result::Result<Value, String> {
            Err("unbound".into())
        }
        fn command(&mut self, _: ObjectId, _: HostCommand) -> std::result::Result<(), String> {
            Err("unbound".into())
        }
        fn register_class(
            &mut self,
            registration: ClassRegistration,
        ) -> std::result::Result<bool, String> {
            let accepted = if registration.name.as_ref() == text("rejected").as_ref() {
                false
            } else if let Some(constructor) = registration.constructor {
                self.classes.insert(registration.name.clone(), constructor);
                true
            } else {
                self.classes.remove(&registration.name).is_some()
            };
            self.requests.push(registration);
            Ok(accepted)
        }
    }
    fn registration_bytes(unregister: bool) -> Vec<u8> {
        let mut bytes = if unregister {
            push(&[Value::Null])
        } else {
            let mut value = push(&[Value::string("Ctor")]);
            value.push(0x1c);
            value
        };
        bytes.extend(push(&[
            Value::string("linkage"),
            Value::Number(2.0),
            Value::string("Object"),
        ]));
        bytes.push(0x1c);
        bytes.extend(push(&[Value::string("registerClass")]));
        bytes.push(0x52);
        bytes
    }
    #[test]
    fn class_registry_replaces_removes_and_distinguishes_definition_code_origins() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        let root = vm.root();
        let first_code = vm
            .register(Arc::new(Program::parse(&function("Ctor", &[])).unwrap()))
            .unwrap();
        vm.execute(first_code, root, &mut RejectHost).unwrap();
        let Value::Object(first) = vm.get(root, "Ctor").unwrap() else {
            panic!()
        };
        let registration = vm
            .register(Arc::new(
                Program::parse(&registration_bytes(false)).unwrap(),
            ))
            .unwrap();
        let mut host = RegistryHost::default();
        let result = vm.execute(registration, root, &mut host).unwrap();
        assert_eq!(result.stack, vec![Value::Bool(true)]);
        assert_eq!(host.classes.get(&text("linkage")), Some(&first));
        assert_eq!(host.requests[0].origin.unwrap().code, registration);
        assert_eq!(
            host.requests[0].constructor_origin,
            Some(CodeOrigin {
                code: first_code,
                source_offset: 0
            })
        );
        let second_code = vm
            .register(Arc::new(Program::parse(&function("Ctor", &[])).unwrap()))
            .unwrap();
        vm.execute(second_code, root, &mut RejectHost).unwrap();
        let Value::Object(second) = vm.get(root, "Ctor").unwrap() else {
            panic!()
        };
        assert_ne!(first, second);
        vm.execute(registration, root, &mut host).unwrap();
        assert_eq!(host.classes.get(&text("linkage")), Some(&second));
        assert_eq!(
            host.requests[1].constructor_origin,
            Some(CodeOrigin {
                code: second_code,
                source_offset: 0
            })
        );
        let unregister = vm
            .register(Arc::new(Program::parse(&registration_bytes(true)).unwrap()))
            .unwrap();
        vm.execute(unregister, root, &mut host).unwrap();
        assert!(host.classes.is_empty());
        assert_eq!(host.requests[2].constructor, None);
        assert_eq!(host.requests[2].constructor_origin, None);
        // Removing a registry entry neither destroys the old constructor nor mutates its prototype.
        assert!(matches!(
            vm.get(first, "prototype").unwrap(),
            Value::Object(_)
        ));
    }
    #[test]
    fn class_registry_is_not_a_success_stub_and_unverified_argument_cases_stop() {
        let mut vm = setup();
        let object = vm.install_object_constructor().unwrap();
        let method = vm.get(object, "registerClass").unwrap();
        let native = vm.native(31).unwrap();
        assert!(matches!(
            vm.call(
                method.clone(),
                Value::Object(object),
                &[Value::string("name"), Value::Object(native)],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::Host(_)
        ));
        let mut host = RegistryHost::default();
        for args in [
            vec![Value::string(""), Value::Object(native)],
            vec![Value::Number(1.0), Value::Object(native)],
            vec![Value::string("name"), Value::Undefined],
            vec![Value::string("name")],
            vec![Value::string("name"), Value::Null, Value::Null],
        ] {
            assert!(matches!(
                vm.call(method.clone(), Value::Object(object), &args, &mut host)
                    .unwrap_err()
                    .kind,
                ErrorKind::Unsupported(_)
            ));
        }
        assert!(host.requests.is_empty());
        let result = vm
            .call(
                method,
                Value::Object(object),
                &[Value::string("rejected"), Value::Object(native)],
                &mut host,
            )
            .unwrap();
        assert_eq!(result.value, Value::Bool(false));
        assert!(host.classes.is_empty());
        assert_eq!(host.requests[0].constructor, Some(native));
        assert_eq!(host.requests[0].constructor_origin, None);
    }
    #[test]
    fn registry_call_origin_is_restored_after_cross_program_accessor_lookup() {
        let mut vm = setup();
        let object = vm.install_object_constructor().unwrap();
        let root = vm.root();
        let method = vm.get(object, "registerClass").unwrap();
        vm.set(root, "registry_target", method).unwrap();
        let mut getter = push(&[Value::string("registry_target")]);
        getter.extend([0x1c, 0x3e]);
        let mut definitions = function("Ctor", &[]);
        definitions.extend(function("registry_getter", &getter));
        let definitions = vm
            .register(Arc::new(Program::parse(&definitions).unwrap()))
            .unwrap();
        vm.execute(definitions, root, &mut RejectHost).unwrap();
        let getter = vm.get(root, "registry_getter").unwrap();
        let Value::Object(prototype) = vm.get(object, "prototype").unwrap() else {
            panic!()
        };
        let add = vm.get(prototype, "addProperty").unwrap();
        vm.call(
            add,
            Value::Object(object),
            &[Value::string("registerClass"), getter, Value::Null],
            &mut RejectHost,
        )
        .unwrap();
        let program = Program::parse(&registration_bytes(false)).unwrap();
        let call_offset = program.instructions.last().unwrap().span.offset;
        let registration = vm.register(Arc::new(program)).unwrap();
        let mut host = RegistryHost::default();
        vm.execute(registration, root, &mut host).unwrap();
        assert_eq!(
            host.requests[0].origin,
            Some(CodeOrigin {
                code: registration,
                source_offset: call_offset
            })
        );
        assert_eq!(
            host.requests[0].constructor_origin.unwrap().code,
            definitions
        );
    }
    #[test]
    fn property_flags_visit_own_hidden_members_without_getters_or_parent_mutation() {
        let mut vm = setup();
        let object = vm.install_object_constructor().unwrap();
        let intrinsic = vm.install_property_flags().unwrap();
        let parent = vm.object().unwrap();
        vm.set(parent, "inherited", Value::Number(1.0)).unwrap();
        let child = vm.alloc(Kind::Plain, Some(parent)).unwrap();
        vm.set(child, "own", Value::Number(2.0)).unwrap();
        let Value::Object(object_proto) = vm.get(object, "prototype").unwrap() else {
            panic!()
        };
        let add = vm.get(object_proto, "addProperty").unwrap();
        let native_getter = vm.native(99).unwrap();
        vm.call(
            add,
            Value::Object(child),
            &[
                Value::string("accessor"),
                Value::Object(native_getter),
                Value::Null,
            ],
            &mut RejectHost,
        )
        .unwrap();
        let result = vm
            .call(
                Value::Object(intrinsic),
                Value::Object(vm.global()),
                &[
                    Value::Object(child),
                    Value::Null,
                    Value::Number(1.0),
                    Value::Number(0.0),
                ],
                &mut RejectHost,
            )
            .unwrap();
        assert_eq!(result.value, Value::Undefined);
        assert_eq!(result.host_calls, 0);
        assert_eq!(vm.own_property_flags(child, "__proto__").unwrap(), Some(3));
        assert_eq!(vm.own_property_flags(child, "accessor").unwrap(), Some(1));
        assert_eq!(vm.own_property_flags(child, "own").unwrap(), Some(1));
        assert_eq!(vm.own_property_flags(parent, "inherited").unwrap(), Some(0));
        assert!(vm.own_enumerable_names(child).unwrap().is_empty());
        assert!(matches!(
            vm.read_property(child, "accessor", &mut RejectHost)
                .unwrap_err()
                .kind,
            ErrorKind::Host(_)
        ));
        vm.call(
            Value::Object(intrinsic),
            Value::Object(vm.global()),
            &[
                Value::Object(child),
                Value::Null,
                Value::Number(0.0),
                Value::Number(1.0),
            ],
            &mut RejectHost,
        )
        .unwrap();
        assert_eq!(vm.own_property_flags(child, "accessor").unwrap(), Some(0));
        assert_eq!(vm.own_property_flags(child, "__proto__").unwrap(), Some(2));
    }
    #[test]
    fn selected_own_flags_preserve_values_and_reject_unproved_selectors_atomically() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        let clip = vm.install_movie_clip_constructor().unwrap();
        let flags = vm.install_property_flags().unwrap();
        let Value::Object(prototype) = vm.get(clip, "prototype").unwrap() else {
            panic!()
        };
        let f = vm.native(55).unwrap();
        for name in ["tween__start", "tween__to", "tween__run"] {
            vm.set(prototype, name, Value::Object(f)).unwrap();
        }
        let call = |vm: &mut Vm, selector: Value| {
            vm.call(
                Value::Object(flags),
                Value::Object(vm.global()),
                &[
                    Value::Object(prototype),
                    selector,
                    Value::Number(1.0),
                    Value::Number(0.0),
                ],
                &mut RejectHost,
            )
        };
        assert_eq!(
            call(&mut vm, Value::string("tween__start,tween__to,tween__run"))
                .unwrap()
                .value,
            Value::Undefined
        );
        for name in ["tween__start", "tween__to", "tween__run"] {
            assert_eq!(vm.own_property_flags(prototype, name).unwrap(), Some(1));
            assert_eq!(vm.get(prototype, name).unwrap(), Value::Object(f));
        }
        for bad in [
            "",
            "tween__start,",
            ",tween__start",
            "tween__start,,tween__to",
            "tween__start, missing",
            "tween__start,missing",
            "constructor",
            "é",
        ] {
            assert!(call(&mut vm, Value::string(bad)).is_err());
            assert_eq!(
                vm.own_property_flags(prototype, "constructor").unwrap(),
                Some(3)
            );
        }
        assert!(call(&mut vm, Value::Null).is_err());
        let plain = vm.object().unwrap();
        vm.set(plain, "dispatchQueue", Value::Object(f)).unwrap();
        let result = vm
            .call(
                Value::Object(flags),
                Value::Object(vm.global()),
                &[
                    Value::Object(plain),
                    Value::string("dispatchQueue"),
                    Value::Number(4.0),
                    Value::Number(0.0),
                ],
                &mut RejectHost,
            )
            .unwrap();
        assert_eq!(result.host_calls, 0);
        assert_eq!(
            vm.own_property_flags(plain, "dispatchQueue").unwrap(),
            Some(4)
        );
        vm.set(plain, "dispatchQueue", Value::Null).unwrap();
        assert_eq!(vm.get(plain, "dispatchQueue").unwrap(), Value::Object(f));
    }
    #[test]
    fn property_flag_consumers_and_value_replacement_preserve_attributes() {
        let mut vm = setup();
        let intrinsic = vm.install_property_flags().unwrap();
        let object = vm.object().unwrap();
        vm.set(object, "field", Value::Number(1.0)).unwrap();
        vm.call(
            Value::Object(intrinsic),
            Value::Object(vm.global()),
            &[
                Value::Object(object),
                Value::Null,
                Value::Number(15.0),
                Value::Number(0.0),
            ],
            &mut RejectHost,
        )
        .unwrap();
        assert_eq!(vm.own_property_flags(object, "field").unwrap(), Some(7));
        vm.set(object, "field", Value::Number(2.0)).unwrap();
        assert_eq!(vm.get(object, "field").unwrap(), Value::Number(1.0));
        assert!(!vm.delete_member(object, &text("field")).unwrap());
        assert!(vm.enumerate_names(object).unwrap().is_empty());
        vm.call(
            Value::Object(intrinsic),
            Value::Object(vm.global()),
            &[
                Value::Object(object),
                Value::Null,
                Value::Number(0.0),
                Value::Number(6.0),
            ],
            &mut RejectHost,
        )
        .unwrap();
        vm.set(object, "field", Value::string("replacement"))
            .unwrap();
        assert_eq!(vm.own_property_flags(object, "field").unwrap(), Some(1));
        assert!(vm.delete_member(object, &text("field")).unwrap());
        assert_eq!(vm.own_property_flags(object, "field").unwrap(), None);
        assert!(!vm.delete_member(object, &text("missing")).unwrap());
        vm.set(object, "only", Value::Number(0.0)).unwrap();
        assert_eq!(vm.enumerate_names(object).unwrap(), vec![text("only")]);
        vm.set(object, "second", Value::Number(0.0)).unwrap();
        assert!(vm.enumerate_names(object).is_err());
    }
    #[test]
    fn flag_consuming_actions_use_their_real_stack_contracts() {
        let mut vm = setup();
        let object = vm.object().unwrap();
        vm.set(vm.root(), "subject", Value::Object(object)).unwrap();
        vm.raw_member(
            object,
            text("locked"),
            Property::Data(Value::Number(1.0)),
            7,
        )
        .unwrap();
        let mut bytes = push(&[Value::string("subject")]);
        bytes.push(0x1c);
        bytes.extend(push(&[Value::string("locked"), Value::Number(2.0)]));
        bytes.push(0x4f);
        bytes.extend(push(&[Value::string("subject")]));
        bytes.push(0x1c);
        bytes.extend(push(&[Value::string("locked")]));
        bytes.push(0x3a);
        assert_eq!(
            execute(&mut vm, &bytes).unwrap().stack,
            vec![Value::Bool(false)]
        );
        assert_eq!(vm.get(object, "locked").unwrap(), Value::Number(1.0));
        vm.set(object, "visible", Value::Null).unwrap();
        for opcode in [0x46, 0x55] {
            let mut bytes = push(&[Value::string("subject")]);
            if opcode == 0x55 {
                bytes.push(0x1c);
            }
            bytes.push(opcode);
            assert_eq!(
                execute(&mut vm, &bytes).unwrap().stack,
                vec![Value::Null, Value::string("visible")]
            );
        }
        let intrinsic = vm.install_property_flags().unwrap();
        vm.call(
            Value::Object(intrinsic),
            Value::Object(vm.global()),
            &[
                Value::Object(object),
                Value::Null,
                Value::Number(1.0),
                Value::Number(7.0),
            ],
            &mut RejectHost,
        )
        .unwrap();
        assert_eq!(vm.own_property_flags(object, "locked").unwrap(), Some(1));
        assert_eq!(vm.own_property_flags(object, "visible").unwrap(), Some(1));
    }
    #[test]
    fn loaded_target_default_follows_captured_activation_not_receiver_or_nested_getter() {
        let mut vm = setup();
        let intrinsic = vm.install_property_flags().unwrap();
        let root = vm.root();
        vm.bind_loaded_movie_target(root, 15).unwrap();
        assert!(vm.bind_loaded_movie_target(root, 14).is_err());
        assert!(
            vm.bind_loaded_movie_target(ObjectId(usize::MAX), 15)
                .is_err()
        );
        let other = vm.object().unwrap();
        let parent = vm.object().unwrap();
        let victim = vm.alloc(Kind::Plain, Some(parent)).unwrap();
        vm.set(victim, "field", Value::Number(42.0)).unwrap();
        vm.set(vm.global(), "victim", Value::Object(victim))
            .unwrap();
        let getter_code = vm
            .register(Arc::new(Program::parse(&function("getter", &[0])).unwrap()))
            .unwrap();
        vm.execute(getter_code, other, &mut RejectHost).unwrap();
        let getter = vm.get(other, "getter").unwrap();
        vm.raw_member(
            victim,
            text("probe"),
            Property::Accessor {
                getter,
                setter: None,
            },
            0,
        )
        .unwrap();
        let mut flags = push(&[Value::Number(1.0), Value::Null, Value::string("victim")]);
        flags.push(0x1c);
        flags.extend(push(&[Value::Number(3.0), Value::string("ASSetPropFlags")]));
        flags.extend([0x3d, 0x3e]);
        let mut body = push(&[Value::string("victim")]);
        body.push(0x1c);
        body.extend(push(&[Value::string("probe")]));
        body.extend([0x4e, 0x17]);
        body.extend(&flags);
        execute(&mut vm, &function("bound", &body)).unwrap();
        let callable = vm.get(root, "bound").unwrap();
        vm.call(callable, Value::Object(other), &[], &mut RejectHost)
            .unwrap();
        assert_eq!(vm.own_property_flags(victim, "field").unwrap(), Some(1));
        assert_eq!(vm.own_property_flags(victim, "__proto__").unwrap(), Some(3));
        assert_eq!(vm.get(victim, "field").unwrap(), Value::Number(42.0));
        let foreign_code = vm
            .register(Arc::new(
                Program::parse(&function("foreign", &flags)).unwrap(),
            ))
            .unwrap();
        vm.execute(foreign_code, other, &mut RejectHost).unwrap();
        let foreign = vm.get(other, "foreign").unwrap();
        assert!(matches!(
            vm.call(foreign, Value::Object(root), &[], &mut RejectHost)
                .unwrap_err()
                .kind,
            ErrorKind::AmbiguousPropertyFlags { old: 3, set: 1 }
        ));
        assert!(matches!(
            vm.call(
                Value::Object(intrinsic),
                Value::Object(root),
                &[Value::Object(victim), Value::Null, Value::Number(1.0)],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::AmbiguousPropertyFlags { old: 3, set: 1 }
        ));
    }
    #[test]
    fn ambiguous_default_flag_clearing_is_atomic_and_member_work_is_bounded() {
        let mut vm = setup();
        let intrinsic = vm.install_property_flags().unwrap();
        let parent = vm.object().unwrap();
        let object = vm.alloc(Kind::Plain, Some(parent)).unwrap();
        vm.set(object, "normal", Value::Number(0.0)).unwrap();
        let error = vm
            .call(
                Value::Object(intrinsic),
                Value::Object(vm.global()),
                &[Value::Object(object), Value::Null, Value::Number(1.0)],
                &mut RejectHost,
            )
            .unwrap_err();
        assert_eq!(
            error.kind,
            ErrorKind::AmbiguousPropertyFlags { old: 3, set: 1 }
        );
        assert_eq!(vm.own_property_flags(object, "normal").unwrap(), Some(0));
        assert_eq!(vm.own_property_flags(object, "__proto__").unwrap(), Some(3));
        let independent = vm.object().unwrap();
        vm.set(independent, "x", Value::Null).unwrap();
        vm.call(
            Value::Object(intrinsic),
            Value::Object(vm.global()),
            &[Value::Object(independent), Value::Null, Value::Number(1.0)],
            &mut RejectHost,
        )
        .unwrap();
        assert_eq!(vm.own_property_flags(independent, "x").unwrap(), Some(1));
        vm.raw_member(independent, text("high"), Property::Data(Value::Null), 0x80)
            .unwrap();
        vm.call(
            Value::Object(intrinsic),
            Value::Object(vm.global()),
            &[Value::Object(independent), Value::Null, Value::Number(1.0)],
            &mut RejectHost,
        )
        .unwrap();
        assert_eq!(
            vm.own_property_flags(independent, "high").unwrap(),
            Some(0x81)
        );
        let mut vm = Vm::new(
            15,
            Limits {
                member_visits: 1,
                ..Limits::default()
            },
        )
        .unwrap();
        let intrinsic = vm.install_property_flags().unwrap();
        let object = vm.object().unwrap();
        vm.set(object, "x", Value::Null).unwrap();
        assert_eq!(
            vm.call(
                Value::Object(intrinsic),
                Value::Object(vm.global()),
                &[
                    Value::Object(object),
                    Value::Null,
                    Value::Number(1.0),
                    Value::Number(0.0)
                ],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::Budget("member visits")
        );
        assert_eq!(vm.own_property_flags(object, "x").unwrap(), Some(0));
    }
}
