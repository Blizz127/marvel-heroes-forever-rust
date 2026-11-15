//! Deliberately partial AVM1 executor. Every unsupported operation fails explicitly.
//! No file, network, native-client, or URL side effects exist in this module.
use super::{ByteString, Function, Literal, Operands, Program};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

#[path = "applied_super.rs"]
mod applied_super;
#[path = "attached.rs"]
mod attached;
#[path = "attached_events.rs"]
mod attached_events;
#[path = "../attached_name_profile.rs"]
pub mod attached_name_profile;
#[path = "attached_textfield.rs"]
mod attached_textfield;
#[path = "deferred.rs"]
mod deferred;
#[path = "execution_store.rs"]
mod execution_store;
#[path = "external_interface.rs"]
mod external_interface;
#[path = "focus_rect.rs"]
mod focus_rect;
#[path = "geom_namespace.rs"]
mod geom_namespace;
#[path = "initial_prototype.rs"]
mod initial_prototype;
#[path = "point.rs"]
mod point;
#[path = "rectangle.rs"]
mod rectangle;
#[cfg(test)]
#[path = "saved_checks.rs"]
mod saved_checks;
#[path = "saved_runtime.rs"]
mod saved_runtime;
#[path = "saved_state.rs"]
pub mod saved_state;
#[cfg(test)]
#[path = "shared_call_checks.rs"]
mod shared_call_checks;
#[path = "shared_calls.rs"]
mod shared_calls;
#[path = "string.rs"]
mod string;
use applied_super::AttachedSuperCall;
use attached::AttachedTarget;
pub use attached::{
    AttachedChild, AttachedClipId, AttachedClipSource, AttachedClipSpec, AttachedFocusRectInfo,
    AttachedHandleSnapshot, AttachedInfo, AttachedMember, AttachedMovieBinding,
    AttachedNameSnapshot, AttachedStorageId, AttachedTabEnabledInfo, AttachedVisibilityBaseProfile,
    AttachedVisibilityInfo, AttachedVisibilityPolicy, AttachedWrite, DisplayTargetId,
    LoadedMovieSourceId,
};
pub use attached_events::{AttachedEventKind, AttachedEventState};
pub use attached_name_profile::{
    AttachedNameClass, BaselineMembership, BaselineOwners, ExpectedProfile, NativeNameProfile,
    ProfileError, ReservedReadName, UnsupportedName,
};
pub use attached_textfield::{
    AttachedTextFieldId, AttachedTextFieldInfo, AttachedTextFieldSource, AttachedTextFieldSpec,
    AttachedTextFieldStorageId, TextFieldMember, TextFieldMemberRequest, TextFieldPrototypeId,
    TextFieldWatchState,
};
pub use deferred::{BuiltinFactory, DeferredBuiltinName};
use execution_store::{ExecutionStore, RegisterBank, StoreError};
pub use external_interface::ExternalInterfaceObjects;
pub use focus_rect::AttachedFocusRectState;
pub use geom_namespace::GeomNamespace;
pub use initial_prototype::{
    InitialMovieClipPrototypeProfile, InitialMovieClipPrototypeReason,
    InitialMovieClipPrototypeSelection,
};
pub use point::PointObjects;
pub use rectangle::RectangleObjects;
pub use saved_runtime::{SavedActionInfo, SavedEnvironmentId, SavedExecution};
pub use saved_state::{SavedActionId, SavedActionSource, SavedEnvironmentState, SavedMode};
pub use shared_calls::SavedScriptCallProfile;
use shared_calls::SharedCallContext;
pub use string::StringObjects;

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
    /// A supported live display reference, distinct from its owner and storage.
    DisplayClip(AttachedClipId),
    DisplayTextField(AttachedTextFieldId),
    /// Retained native tag11 name; factory/cache belong to the active context.
    DeferredBuiltin(DeferredBuiltinName),
}

/// Read-only value classification. Callable identity does not promise that a
/// native/script invocation profile is implemented; deferred tokens stay unresolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueCallability {
    Callable,
    NonCallable,
    DeferredBuiltin,
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
            Self::Object(_)
            | Self::DisplayClip(_)
            | Self::DisplayTextField(_)
            | Self::DeferredBuiltin(_) => true,
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
    UnsupportedNativeFunction(&'static str),
    UnsupportedDeferredBuiltin(Text),
    StackUnderflow,
    InvalidObject(ObjectId),
    InvalidAttachedClip(AttachedClipId),
    InvalidAttachedTextField(AttachedTextFieldId),
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
    /// Active interpreter target, independent of `this` and code provenance.
    /// A direct native Vm::call has no script Environment and supplies None.
    pub context: Option<ObjectId>,
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
#[derive(Debug, Clone)]
pub enum HostResult {
    Value(Value),
    /// A fresh native Object construction, followed by ordinary writes in order.
    /// Current profile: unique ASCII identifiers and valid NUL-free String/Bool
    /// values. The host owns reflected-field selection and its input limits.
    ObjectFields(Vec<(Text, Value)>),
}
/// Exact typed receiver and current state for an admitted named `_visible`
/// write. The host must prepare every fallible operation before returning its
/// one-use commit. Source/parent associations are immutable after adoption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedVisibilityRequest {
    pub root: ObjectId,
    pub context: ObjectId,
    pub receiver: AttachedClipId,
    pub owner: ObjectId,
    pub target: DisplayTargetId,
    pub expected_local: bool,
    pub visible: bool,
    pub origin: Option<CodeOrigin>,
}
/// Native zero-argument Mouse.hide host event. The callback ignores `this`;
/// the active Environment target selects the root/MovieImpl and its handler.
/// Event type is0x16, byte+4 and DWORD+8 are0, and `index` is DWORD+c.
/// The admitted zero-argument callback always supplies index0. A host must
/// validate its explicit context/handler association and implement the actual
/// event effects; acceptance is not a generic cursor-visibility stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseHideRequest {
    pub origin: Option<CodeOrigin>,
    pub context: ObjectId,
    pub index: u32,
}
pub trait Host {
    fn call(&mut self, call: HostCall<'_>) -> std::result::Result<Value, String>;
    /// Existing hosts retain their direct-value behavior. The VM materializes
    /// ObjectFields only after this callback returns, permitting setter reentry.
    fn call_result(&mut self, call: HostCall<'_>) -> std::result::Result<HostResult, String> {
        self.call(call).map(HostResult::Value)
    }
    /// Deliver the admitted event to the explicitly bound current movie's
    /// user-event handler. No handler, cursor state or successful delivery is
    /// assumed by default. This interface itself does not install Mouse.
    fn mouse_hide(&mut self, _request: MouseHideRequest) -> std::result::Result<(), String> {
        Err("Mouse.hide user-event host is unbound".into())
    }
    /// Preparation may reject but must not publish changed display state. The
    /// borrowed closure publishes the admitted result once after the VM commits
    /// its raw local bit, without callbacks, reentry or fallible work.
    fn prepare_attached_visibility(
        &mut self,
        _request: AttachedVisibilityRequest,
    ) -> std::result::Result<Box<dyn FnOnce() + '_>, String> {
        Err("attached visibility host is unbound".into())
    }
    fn get_text_field_member(
        &mut self,
        request: TextFieldMemberRequest,
    ) -> std::result::Result<Value, String> {
        Err(format!("unbound TextField getter: {:?}", request.member))
    }
    fn set_text_field_member(
        &mut self,
        request: TextFieldMemberRequest,
        _value: Value,
    ) -> std::result::Result<(), String> {
        Err(format!("unbound TextField setter: {:?}", request.member))
    }
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
    UnsupportedNative(&'static str),
    ObjectConstructor,
    ArrayConstructor,
    FunctionConversion,
    MovieClipConstructor,
    ExternalInterfaceConstructor,
    ExternalInterfacePrototype,
    PointConstructor,
    PointObject(point::PointRole),
    RectangleConstructor,
    RectangleObject(rectangle::RectangleRole),
    StringConstructor,
    StringObject(string::StringState),
    StringMethod(string::StringMethod),
    MovieClip(MovieClipRole),
    AttachedOwner(AttachedClipId),
    BoundStorage(AttachedClipId),
    AttachedTextFieldOwner(AttachedTextFieldId),
    BoundTextFieldStorage(AttachedTextFieldId),
    AddProperty,
    RegisterClass,
    SetPropertyFlags,
    FunctionMethod(FunctionMethod),
    FunctionPrototype,
    ConstructorSuper {
        constructor: ObjectId,
        receiver: ObjectId,
    },
    AttachedSuper {
        constructor: ObjectId,
        call: AttachedSuperCall,
    },
}
impl Kind {
    fn function_value(&self) -> bool {
        matches!(
            self,
            Self::Native(_)
                | Self::UnsupportedNative(_)
                | Self::Script(_)
                | Self::ObjectConstructor
                | Self::ArrayConstructor
                | Self::FunctionConversion
                | Self::MovieClipConstructor
                | Self::ExternalInterfaceConstructor
                | Self::PointConstructor
                | Self::RectangleConstructor
                | Self::StringConstructor
                | Self::StringMethod(_)
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
    NativeMarker(NativeMarker),
    Accessor {
        getter: Value,
        setter: Option<Value>,
    },
}
#[derive(Debug, Clone, Copy)]
enum NativeMarker {
    ExternalAvailable,
    StringLength,
}
#[derive(Debug, Clone, Copy)]
struct ClassBase {
    constructor: ObjectId,
    prototype: ObjectId,
    parent_prototype: ObjectId,
}
#[derive(Debug, Clone, Copy)]
enum InvocationContext {
    Ordinary,
    Construction(ObjectId),
    Attached(AttachedSuperCall),
}
#[derive(Debug, Clone)]
struct Object {
    kind: Kind,
    properties: BTreeMap<Text, Property>,
    flags: BTreeMap<Text, u8>,
    prototype: Option<ObjectId>,
    insertion_keys: Vec<Text>,
    native_hash_profile: bool,
}
struct Frame {
    store: Rc<RefCell<ExecutionStore>>,
    registers: RegisterBank,
    shared_calls: Option<SharedCallContext>,
    scopes: Vec<ObjectId>,
    pool: Arc<Vec<Value>>,
    target: ObjectId,
    this: Value,
    this_visible: bool,
    locals: ObjectId,
    kind: FrameKind,
}
enum FrameKind {
    Ordinary,
    Saved {
        action: saved_state::SavedActionState,
    },
}
impl Frame {
    fn is_saved(&self) -> bool {
        matches!(self.kind, FrameKind::Saved { .. })
    }
    fn is_shared_script(&self) -> bool {
        !self.is_saved() && self.shared_calls.is_some()
    }
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
    builtin_registry: BTreeMap<Text, deferred::BuiltinEntry>,
    builtin_targets: BTreeMap<ObjectId, ()>,
    string_objects: Option<StringObjects>,
    point_objects: Option<PointObjects>,
    rectangle_objects: Option<RectangleObjects>,
    geom_namespace: Option<GeomNamespace>,
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
    object_prototype: Option<ObjectId>,
    function_prototype: Option<ObjectId>,
    function_conversion: Option<ObjectId>,
    movie_clip_constructor: Option<ObjectId>,
    movie_clip_prototype: Option<ObjectId>,
    movie_clip_parent: Option<ObjectId>,
    movie_clip_event_names: BTreeMap<ObjectId, bool>,
    array_constructor: Option<ObjectId>,
    array_prototype: Option<ObjectId>,
    class_bases: BTreeMap<ObjectId, ClassBase>,
    attached: Vec<attached::AttachedClip>,
    attached_name_profile: Option<Arc<NativeNameProfile>>,
    attached_visibility_policy: Option<AttachedVisibilityPolicy>,
    attached_targets: BTreeMap<DisplayTargetId, AttachedTarget>,
    text_fields: Vec<attached_textfield::AttachedTextField>,
    attached_child_entries: usize,
    saved_actions: Vec<Option<saved_runtime::InstalledAction>>,
    saved_environments: Vec<Option<saved_runtime::InstalledEnvironment>>,
    saved_environment_owners: BTreeMap<AttachedClipId, SavedEnvironmentId>,
    saved_active: bool,
}
impl Vm {
    /// This execution profile supports SWF 7+; legacy coercions are not guessed.
    pub fn new(version: u8, limits: Limits) -> Result<Self> {
        let mut vm = Self {
            limits,
            movie_version: version,
            loaded_movie_targets: BTreeMap::new(),
            builtin_registry: BTreeMap::new(),
            builtin_targets: BTreeMap::new(),
            string_objects: None,
            point_objects: None,
            rectangle_objects: None,
            geom_namespace: None,
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
            object_prototype: None,
            function_prototype: None,
            function_conversion: None,
            movie_clip_constructor: None,
            movie_clip_prototype: None,
            movie_clip_parent: None,
            movie_clip_event_names: BTreeMap::new(),
            array_constructor: None,
            array_prototype: None,
            class_bases: BTreeMap::new(),
            attached: Vec::new(),
            attached_name_profile: None,
            attached_visibility_policy: None,
            attached_targets: BTreeMap::new(),
            text_fields: Vec::new(),
            attached_child_entries: 0,
            saved_actions: Vec::new(),
            saved_environments: Vec::new(),
            saved_environment_owners: BTreeMap::new(),
            saved_active: false,
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
    /// Controlled Plain targets share the VM profile. Attached targets already
    /// have an immutable MovieDef/version established by their adoption route.
    pub fn bind_loaded_movie_target(&mut self, target: ObjectId, header_version: u8) -> Result<()> {
        self.check_object(target)?;
        if let Kind::AttachedOwner(id) = self.objects[target.0].kind {
            if header_version != self.attached_info(id)?.environment_header_version {
                return self.fail(ErrorKind::Unsupported(
                    "attached MovieDef version reassignment",
                ));
            }
            return Ok(());
        }
        if !matches!(self.objects[target.0].kind, Kind::Plain) {
            return self.fail(ErrorKind::Unsupported(
                "loaded movie target identity profile",
            ));
        }
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
            insertion_keys: Vec::new(),
            native_hash_profile: false,
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
        let add_property = self.alloc(Kind::AddProperty, None)?;
        self.raw_member(
            prototype,
            text("addProperty"),
            Property::Data(Value::Object(add_property)),
            1,
        )?;
        // Complete native Object.prototype inventory: preserve function identity and
        // flags without pretending these callbacks are implemented.
        for (key, name) in [
            ("hasOwnProperty", "Object.prototype.hasOwnProperty"),
            ("watch", "Object.prototype.watch"),
            ("unwatch", "Object.prototype.unwatch"),
            (
                "isPropertyEnumerable",
                "Object.prototype.isPropertyEnumerable",
            ),
            ("isPrototypeOf", "Object.prototype.isPrototypeOf"),
            ("toString", "Object.prototype.toString"),
            ("valueOf", "Object.prototype.valueOf"),
        ] {
            let method = self.alloc(Kind::UnsupportedNative(name), None)?;
            self.raw_member(
                prototype,
                text(key),
                Property::Data(Value::Object(method)),
                1,
            )?;
        }
        let register_class = self.alloc(Kind::RegisterClass, None)?;
        self.set(constructor, "registerClass", Value::Object(register_class))?;
        self.set(self.global, "Object", Value::Object(constructor))?;
        self.object_constructor = Some(constructor);
        self.object_prototype = Some(prototype);
        self.objects[prototype.0].native_hash_profile = true;
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
        if self.attached_visibility_policy.is_some()
            && id == self.global
            && key.as_ref() == text("noInvisibleAdvance").as_ref()
        {
            return self.fail(ErrorKind::Unsupported(
                "bound native movie visibility policy cannot use ordinary globals",
            ));
        }
        if matches!(
            self.objects[id.0].kind,
            Kind::AttachedOwner(_)
                | Kind::BoundStorage(_)
                | Kind::AttachedTextFieldOwner(_)
                | Kind::BoundTextFieldStorage(_)
        ) {
            return self.fail(ErrorKind::Unsupported(
                "attached owner/storage requires typed member route",
            ));
        }
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
        if matches!(
            self.objects[id.0].kind,
            Kind::ConstructorSuper { .. } | Kind::AttachedSuper { .. }
        ) {
            return self.fail(ErrorKind::Unsupported("super member lookup/assignment"));
        }
        Ok(())
    }
    fn is_geometry_native(&self, id: ObjectId) -> bool {
        self.objects.get(id.0).is_some_and(|object| {
            matches!(
                object.kind,
                Kind::PointConstructor
                    | Kind::PointObject(_)
                    | Kind::RectangleConstructor
                    | Kind::RectangleObject(_)
            )
        })
    }
    fn reject_geometry_mutation_chain(&self, mut id: ObjectId) -> Result<()> {
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            if self.is_geometry_native(id) {
                return self.fail(ErrorKind::Unsupported("geometry native mutation profile"));
            }
            match self.objects[id.0].prototype {
                Some(parent) => id = parent,
                None => return Ok(()),
            }
        }
        self.fail(ErrorKind::Budget("geometry mutation prototype depth"))
    }
    fn is_string_native(&self, id: ObjectId) -> bool {
        self.objects.get(id.0).is_some_and(|object| {
            matches!(
                object.kind,
                Kind::StringConstructor | Kind::StringObject(_) | Kind::StringMethod(_)
            )
        }) || self
            .string_objects
            .is_some_and(|graph| graph.from_char_code == id)
    }
    fn reject_string_mutation_chain(&self, mut id: ObjectId) -> Result<()> {
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            if self.is_string_native(id) {
                return self.fail(ErrorKind::Unsupported("String native mutation profile"));
            }
            match self.objects[id.0].prototype {
                Some(parent) => id = parent,
                None => return Ok(()),
            }
        }
        self.fail(ErrorKind::Budget("String mutation prototype depth"))
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
        let selected = self.materialize_value(selected)?;
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
            if matches!(
                self.objects[selected.0].kind,
                Kind::ExternalInterfaceConstructor
            ) {
                return self.fail(ErrorKind::Unsupported(
                    "ExternalInterface allocator profile",
                ));
            }
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
    fn object_literal(&mut self) -> Result<ObjectId> {
        let (Some(constructor), Some(prototype)) = (self.object_constructor, self.object_prototype)
        else {
            return self.fail(ErrorKind::Unsupported(
                "InitObject requires installed native Object",
            ));
        };
        let bindings = [
            (self.global, "Object", constructor),
            (constructor, "prototype", prototype),
            (prototype, "constructor", constructor),
        ];
        if bindings.iter().any(|(owner, name, expected)| {
            !matches!(self.objects[owner.0].properties.get(&text(name)),
                Some(Property::Data(Value::Object(actual))) if actual == expected)
        }) || self.objects[prototype.0].prototype.is_some()
        {
            return self.fail(ErrorKind::Unsupported(
                "InitObject modified Object binding/prototype",
            ));
        }
        // Exact unchanged-native-Object zero-argument New route: allocator links
        // __proto__, shared New writes hidden ctor, callback writes public ctor.
        // This is not the metadata contract for every ordinary_object caller.
        let object = self.alloc(Kind::Plain, Some(prototype))?;
        for name in ["__constructor__", "constructor"] {
            self.raw_member(
                object,
                text(name),
                Property::Data(Value::Object(constructor)),
                3,
            )?;
        }
        self.objects[object.0].native_hash_profile = true;
        Ok(object)
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
        if self.is_geometry_native(id) || self.is_geom_namespace(id) {
            return self.fail(ErrorKind::Unsupported(
                "partial native own-member inspection",
            ));
        }
        if self.is_string_native(id) {
            return self.fail(ErrorKind::Unsupported(
                "partial String native own-member inspection",
            ));
        }
        if matches!(
            self.objects[id.0].kind,
            Kind::AttachedTextFieldOwner(_) | Kind::BoundTextFieldStorage(_)
        ) {
            return self.fail(ErrorKind::Unsupported(
                "TextField raw own-member inspection",
            ));
        }
        let key = text(key);
        Ok(self.objects[id.0]
            .properties
            .contains_key(&key)
            .then(|| self.own_flags(id, &key)))
    }
    /// Inspection order is lexical, not a claim about native hash enumeration.
    pub fn own_enumerable_names(&self, id: ObjectId) -> Result<Vec<Text>> {
        self.check_object(id)?;
        if self.is_geometry_native(id) || self.is_geom_namespace(id) {
            return self.fail(ErrorKind::Unsupported(
                "partial native own-member inspection",
            ));
        }
        if self.is_string_native(id) {
            return self.fail(ErrorKind::Unsupported(
                "partial String native own-member inspection",
            ));
        }
        if matches!(
            self.objects[id.0].kind,
            Kind::AttachedTextFieldOwner(_) | Kind::BoundTextFieldStorage(_)
        ) {
            return self.fail(ErrorKind::Unsupported(
                "TextField raw own-member inspection",
            ));
        }
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
        let attached = matches!(args[0], Value::DisplayClip(_));
        let target = if let Value::DisplayClip(clip) = args[0] {
            let [_, Value::String(key), Value::Number(mask)] = args else {
                return self.fail(ErrorKind::Unsupported(
                    "attached ASSetPropFlags requires one name and set mask one",
                ));
            };
            if *mask != 1.0 {
                return self.fail(ErrorKind::Unsupported(
                    "attached ASSetPropFlags set mask profile",
                ));
            }
            if key.is_empty()
                || !key.iter().all(|unit| {
                    u8::try_from(*unit).is_ok_and(|unit| {
                        unit.is_ascii_alphanumeric() || matches!(unit, b'_' | b'$')
                    })
                })
            {
                return self.fail(ErrorKind::Unsupported(
                    "attached ASSetPropFlags exact ASCII name",
                ));
            }
            self.attached_property_flags_storage(clip, key)?
        } else {
            self.object_id(args[0].clone())?
        };
        self.reject_geometry_mutation_chain(target)?;
        self.reject_geom_mutation_chain(target)?;
        let prototype = matches!(
            self.objects[target.0].kind,
            Kind::MovieClip(MovieClipRole::UnboundPrototype)
        );
        if !matches!(self.objects[target.0].kind, Kind::Plain) && !prototype && !attached {
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
            if self.is_geom_namespace(id) || !matches!(self.objects[id.0].kind, Kind::Plain) {
                return self.fail(ErrorKind::Unsupported(
                    "enumeration on non-generic or partial native class",
                ));
            }
            self.visit_members(self.objects[id.0].properties.len())?;
            let visible_count = self.objects[id.0]
                .properties
                .keys()
                .filter(|key| self.own_flags(id, key) & 1 == 0)
                .count();
            let keys: Vec<Text> = if visible_count > 1 {
                if !self.objects[id.0].native_hash_profile {
                    return self.fail(ErrorKind::Unsupported(
                        "native hash order after unproved creation/deletion history",
                    ));
                }
                let (indices, work) = native_hash_order(
                    &self.objects[id.0].insertion_keys,
                    self.limits.member_visits.saturating_sub(self.member_visits),
                )
                .map_err(|kind| self.error(kind))?;
                self.member_visits += work;
                indices
                    .into_iter()
                    .map(|index| self.objects[id.0].insertion_keys[index].clone())
                    .collect()
            } else {
                self.objects[id.0].properties.keys().cloned().collect()
            };
            for key in &keys {
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
        self.check_member_target(id, key)?;
        self.reject_string_mutation_chain(id)?;
        self.reject_geometry_mutation_chain(id)?;
        self.reject_geom_mutation_chain(id)?;
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
        self.objects[id.0].native_hash_profile = false;
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
        match value {
            Value::Object(id) => {
                self.check_object(*id)?;
                if matches!(
                    self.objects[id.0].kind,
                    Kind::AttachedOwner(_)
                        | Kind::BoundStorage(_)
                        | Kind::AttachedTextFieldOwner(_)
                        | Kind::BoundTextFieldStorage(_)
                ) {
                    return self.fail(ErrorKind::Unsupported(
                        "attached owner/storage is not an ordinary object value",
                    ));
                }
                Ok(())
            }
            Value::DisplayClip(id) => self.check_attached(*id),
            Value::DisplayTextField(id) => self.attached_text_field_info(*id).map(|_| ()),
            _ => Ok(()),
        }
    }
    fn target_value(&self, id: ObjectId) -> Value {
        match self.objects[id.0].kind {
            Kind::AttachedOwner(clip) => Value::DisplayClip(clip),
            Kind::AttachedTextFieldOwner(field) => Value::DisplayTextField(field),
            _ => Value::Object(id),
        }
    }
    /// Explicit member access may use a display owner's interface; generic
    /// object coercion deliberately does not manufacture an Object value.
    fn materialize_value(&mut self, value: Value) -> Result<Value> {
        match value {
            Value::DeferredBuiltin(name) => self.resolve_deferred_builtin(&name).map(Value::Object),
            value => Ok(value),
        }
    }
    fn member_owner(&mut self, value: Value) -> Result<ObjectId> {
        let value = self.materialize_value(value)?;
        match value {
            Value::DisplayClip(id) => self.attached_owner(id),
            Value::DisplayTextField(id) => self.attached_text_field_owner(id),
            value => self.object_id(value),
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
        self.check_object(id)?;
        if let Kind::AttachedOwner(clip) = self.objects[id.0].kind {
            return Ok(match self.attached_lookup(clip, key)? {
                AttachedMember::Missing => Value::Undefined,
                AttachedMember::Present(value) => value,
            });
        }
        let mut native_member_context = false;
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            if let Some(value) = self.point_read(id, key)? {
                return Ok(value);
            }
            if let Some(value) = self.rectangle_read(id, key)? {
                return Ok(value);
            }
            if let Some(value) = self.geom_namespace_member(id, key)? {
                return Ok(value);
            }
            if self.is_string_native(id) {
                return self.string_member(id, key);
            }
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
                    Property::NativeMarker(NativeMarker::ExternalAvailable) => self.fail(
                        ErrorKind::Unsupported("ExternalInterface.available active handler query"),
                    ),
                    Property::NativeMarker(NativeMarker::StringLength) => self.fail(
                        ErrorKind::Unsupported("String length marker requires typed receiver"),
                    ),
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
        self.check_object(id)?;
        if let Kind::AttachedOwner(clip) = self.objects[id.0].kind {
            return Ok(matches!(
                self.attached_lookup(clip, key)?,
                AttachedMember::Present(_)
            ));
        }
        let mut native_member_context = false;
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            if self.point_read(id, key)?.is_some()
                || self.rectangle_read(id, key)?.is_some()
                || self.geom_namespace_member(id, key)?.is_some()
            {
                return Ok(true);
            }
            if self.is_string_native(id) {
                self.string_member(id, key)?;
                return Ok(true);
            }
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
            if let Some(value) = self.point_read(id, key)? {
                return Ok(Some(Property::Data(value)));
            }
            if let Some(value) = self.rectangle_read(id, key)? {
                return Ok(Some(Property::Data(value)));
            }
            if let Some(value) = self.geom_namespace_member(id, key)? {
                return Ok(Some(Property::Data(value)));
            }
            if self.is_string_native(id) {
                return Ok(Some(Property::Data(self.string_member(id, key)?)));
            }
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
            self.objects
                .get(id.0)
                .is_some_and(|object| object.kind.function_value())
        } else {
            false
        }
    }
    /// Inspect the existing Value/kind without coercion, native calls or lazy
    /// factory resolution. Invalid or disguised owner/storage handles reject.
    pub fn classify_callability(&self, value: &Value) -> Result<ValueCallability> {
        self.check_value(value)?;
        Ok(if matches!(value, Value::DeferredBuiltin(_)) {
            ValueCallability::DeferredBuiltin
        } else if self.callable(value) {
            ValueCallability::Callable
        } else {
            ValueCallability::NonCallable
        })
    }
    fn read_member(&mut self, id: ObjectId, key: &Text, host: &mut impl Host) -> Result<Value> {
        self.check_object(id)?;
        if let Kind::AttachedTextFieldOwner(field) = self.objects[id.0].kind {
            return self.attached_text_field_read(field, key, host);
        }
        if matches!(self.objects[id.0].kind, Kind::AttachedOwner(_)) {
            return self.get_key(id, key);
        }
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
        self.check_object(id)?;
        self.reject_string_mutation_chain(id)?;
        self.reject_geometry_mutation_chain(id)?;
        self.reject_geom_mutation_chain(id)?;
        if let Kind::AttachedTextFieldOwner(field) = self.objects[id.0].kind {
            return self.attached_text_field_write(field, key, value, host);
        }
        if matches!(
            self.objects[id.0].kind,
            Kind::ExternalInterfaceConstructor | Kind::ExternalInterfacePrototype
        ) {
            return self.fail(ErrorKind::Unsupported(
                "ExternalInterface member mutation profile",
            ));
        }
        if let Kind::AttachedOwner(clip) = self.objects[id.0].kind {
            if key.as_ref() == text("_visible").as_ref() {
                self.attached_visibility_write(clip, &key, value, host)?;
                return Ok(());
            }
            return self.set_key(id, key, value);
        }
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
            self.objects[id.0].insertion_keys.push(key.clone());
        }
        Ok(())
    }
    fn set_key(&mut self, id: ObjectId, key: Text, value: Value) -> Result<()> {
        self.check_object(id)?;
        if self.is_geometry_native(id) {
            if self.point_write(id, &key, value.clone())?
                || self.rectangle_write(id, &key, value.clone())?
            {
                return Ok(());
            }
        }
        self.reject_string_mutation_chain(id)?;
        self.reject_geometry_mutation_chain(id)?;
        self.reject_geom_mutation_chain(id)?;
        if matches!(
            self.objects[id.0].kind,
            Kind::ExternalInterfaceConstructor | Kind::ExternalInterfacePrototype
        ) {
            return self.fail(ErrorKind::Unsupported(
                "ExternalInterface member mutation profile",
            ));
        }
        if let Kind::AttachedOwner(clip) = self.objects[id.0].kind {
            self.check_value(&value)?;
            if key.as_ref() == text("__proto__").as_ref() {
                let prototype = match value {
                    Value::Object(id) => Some(id),
                    Value::Null => None,
                    _ => {
                        return self
                            .fail(ErrorKind::Unsupported("attached prototype value profile"));
                    }
                };
                return self.set_attached_prototype(clip, prototype);
            }
            self.attached_write(clip, key, value)?;
            return Ok(());
        }
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
        if matches!(self.descriptor(id, &key)?, Some(Property::NativeMarker(_))) {
            return self.fail(ErrorKind::Unsupported("native special-property assignment"));
        }
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
    fn store_error(&self, error: StoreError) -> Error {
        self.error(match error {
            StoreError::Underflow => ErrorKind::StackUnderflow,
            StoreError::StackLimit => ErrorKind::Budget("operand stack"),
            StoreError::RegisterLimit => ErrorKind::Budget("dynamic registers"),
            StoreError::InvalidRegister(index) => ErrorKind::InvalidRegister(index),
            StoreError::InvalidRegisterFrame => ErrorKind::Unsupported("register frame ownership"),
        })
    }
    fn isolated_store(
        &self,
        register_count: Option<u8>,
    ) -> Result<(Rc<RefCell<ExecutionStore>>, RegisterBank)> {
        let mut store = ExecutionStore::new(
            Vec::new(),
            std::array::from_fn(|_| Value::Undefined),
            self.limits.stack_values,
        )
        .map_err(|error| self.store_error(error))?;
        let bank = match register_count {
            Some(count) => store
                .enter_function2(count)
                .map_err(|error| self.store_error(error))?,
            None => RegisterBank::Fixed,
        };
        Ok((Rc::new(RefCell::new(store)), bank))
    }
    fn push(&self, frame: &mut Frame, value: Value) -> Result<()> {
        frame
            .store
            .borrow_mut()
            .push(value)
            .map_err(|error| self.store_error(error))
    }
    fn pop(&self, frame: &mut Frame) -> Result<Value> {
        frame
            .store
            .borrow_mut()
            .pop()
            .map_err(|error| self.store_error(error))
    }
    fn peek(&self, frame: &Frame, depth: usize) -> Result<Value> {
        frame
            .store
            .borrow()
            .peek_from_top(depth)
            .cloned()
            .map_err(|error| self.store_error(error))
    }
    fn object_id(&self, value: Value) -> Result<ObjectId> {
        if let Value::Object(id) = value {
            self.check_value(&Value::Object(id))?;
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
            Value::DeferredBuiltin(_) => Ok(0.0),
            Value::Bool(v) => Ok(if *v { 1.0 } else { 0.0 }),
            Value::Undefined => Ok(f64::NAN),
            Value::Null => self.fail(ErrorKind::Unsupported(
                "unverified AVM1 null-to-number conversion",
            )),
            Value::Object(_) | Value::DisplayClip(_) | Value::DisplayTextField(_) => self.fail(
                ErrorKind::Unsupported("object primitive coercion requires valueOf/toString"),
            ),
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
            Value::DeferredBuiltin(_) => {
                if self.current_target.is_none() {
                    return self.fail(ErrorKind::Unsupported(
                        "deferred builtin string conversion requires active Environment",
                    ));
                }
                Ok(text("[type Function]"))
            }
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
            Value::Object(_) | Value::DisplayClip(_) | Value::DisplayTextField(_) => self.fail(
                ErrorKind::Unsupported("object primitive coercion requires valueOf/toString"),
            ),
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
            return Ok(self.target_value(self.root));
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
    // This is an ordinary namespace profile in the modeled target/global scope.
    // An attached native display target has separate child/prototype resolution.
    fn get_variable(&mut self, frame: &Frame, key: &Text, host: &mut impl Host) -> Result<Value> {
        if !key.iter().any(|v| matches!(*v, 46 | 47 | 58)) {
            return self.variable(frame, key, host);
        }
        if frame.scopes.as_slice() != [frame.target, self.global] || frame.locals != frame.target {
            return self.fail(ErrorKind::Unsupported(
                "qualified variable activation/With scope",
            ));
        }
        for id in &frame.scopes {
            self.check_object(*id)?;
            if *id == frame.target && matches!(self.objects[id.0].kind, Kind::AttachedOwner(_)) {
                // Explicit attached lookup performs storage/children/prototype
                // admission; the historical Plain-root profile stays separate.
                continue;
            }
            if !matches!(self.objects[id.0].kind, Kind::Plain)
                || self.objects[id.0].prototype.is_some()
            {
                return self.fail(ErrorKind::Unsupported(
                    "qualified variable modeled root/global profile",
                ));
            }
        }
        if key.split(|v| *v == 46).take(4).count() != 3 {
            return self.fail(ErrorKind::Unsupported(
                "qualified variable three-segment profile",
            ));
        }
        if self.limits.prototype_depth < 3 {
            return self.fail(ErrorKind::Budget("qualified variable path depth"));
        }
        let parts: Vec<_> = key.split(|v| *v == 46).collect();
        for part in &parts {
            if !part
                .first()
                .is_some_and(|v| matches!(*v, 65..=90 | 97..=122))
                || part
                    .iter()
                    .any(|v| !matches!(*v, 48..=57 | 65..=90 | 95 | 97..=122))
            {
                return self.fail(ErrorKind::Unsupported(
                    "qualified variable ASCII identifier profile",
                ));
            }
        }
        let first = self.allocate_text(parts[0].to_vec())?;
        if ["this", "super", "arguments", "NaN", "Infinity"]
            .iter()
            .any(|v| first.as_ref() == text(v).as_ref())
        {
            return self.fail(ErrorKind::Unsupported(
                "qualified variable reserved first segment",
            ));
        }
        for id in &frame.scopes {
            if self.has_key(*id, &first)? {
                if !matches!(self.objects[id.0].kind, Kind::AttachedOwner(_))
                    && !matches!(
                        self.objects[id.0].properties.get(&first),
                        Some(Property::Data(_))
                    )
                {
                    return self.fail(ErrorKind::Unsupported(
                        "qualified variable first-segment accessor",
                    ));
                }
                break;
            }
        }
        // Preserve the normal first-segment target-before-global precedence.
        let mut value = self.variable(frame, &first, host)?;
        for part in &parts[1..] {
            if self.member_visits >= self.limits.member_visits {
                return self.fail(ErrorKind::Budget("qualified variable member visits"));
            }
            self.member_visits += 1;
            value = self.materialize_value(value)?;
            let Value::Object(owner) = value else {
                return self.fail(ErrorKind::Unsupported(
                    "qualified variable primitive/missing segment",
                ));
            };
            self.check_object(owner)?;
            if !matches!(self.objects[owner.0].kind, Kind::Plain) {
                return self.fail(ErrorKind::Unsupported(
                    "qualified variable namespace object kind",
                ));
            }
            let name = self.allocate_text(part.to_vec())?;
            value = match self.objects[owner.0].properties.get(&name) {
                Some(Property::Data(value)) => value.clone(),
                _ => {
                    return self.fail(ErrorKind::Unsupported(
                        "qualified variable own data member profile",
                    ));
                }
            };
        }
        if matches!(value, Value::DeferredBuiltin(_)) {
            // Terminal ordinary member copies retain the token unchanged.
            return Ok(value);
        }
        if let Value::Object(id) = value {
            if matches!(self.objects[id.0].kind, Kind::Script(_)) {
                // Ordinary member copy preserves this function reference. This is
                // deliberately distinct from Function() wrapper conversion.
                return Ok(value);
            }
        }
        self.fail(ErrorKind::Unsupported(
            "qualified variable final ScriptFunction profile",
        ))
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
    fn saved_set_variable(&mut self, frame: &mut Frame, host: &mut impl Host) -> Result<()> {
        let len = frame.store.borrow().len();
        if len < 2 {
            return self.fail(ErrorKind::StackUnderflow);
        }
        let Value::String(key) = self.peek(frame, 1)? else {
            return self.fail(ErrorKind::Unsupported(
                "saved SetVariable primitive String name profile",
            ));
        };
        if key.is_empty() || key.iter().any(|unit| *unit == 0 || *unit > 127) {
            return self.fail(ErrorKind::Unsupported(
                "saved SetVariable simple ASCII name profile",
            ));
        }
        self.simple_name(&key)?;

        // Native routing forwards the original Value without RHS coercion.
        // The selected member setter retains its own type/identity guards.
        let value = self.peek(frame, 0)?;
        // Empty captured/With scope and the saved null activation fence route
        // directly to the current target. Reads' global fallback does not apply.
        // Keep both operands during normal setter dispatch; unsupported writes
        // stop before native normal-return cleanup would consume them.
        self.write_member(frame.target, key, value, host)?;
        frame.store.borrow_mut().stack.truncate(len - 2);
        Ok(())
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
        if !self.saved_environments.is_empty() {
            return self.fail(ErrorKind::Unsupported(
                "diagnostic execution after saved Environment adoption",
            ));
        }
        self.reset_execution();
        self.check_object(target)?;
        if matches!(
            self.objects[target.0].kind,
            Kind::BoundStorage(_)
                | Kind::BoundTextFieldStorage(_)
                | Kind::AttachedTextFieldOwner(_)
        ) {
            return self.fail(ErrorKind::Unsupported(
                "bound storage/TextField owner is not an execution Environment target",
            ));
        }
        let program = self
            .code
            .get(code.0)
            .cloned()
            .ok_or_else(|| self.error(ErrorKind::InvalidCode(code)))?;
        let (store, registers) = self.isolated_store(None)?;
        let mut frame = Frame {
            store,
            registers,
            shared_calls: None,
            scopes: vec![target, self.global],
            pool: Arc::new(Vec::new()),
            target,
            this: self.target_value(target),
            this_visible: true,
            locals: target,
            kind: FrameKind::Ordinary,
        };
        let value = match self.run(code, &program, &mut frame, host)? {
            Flow::Return(v) => v,
            Flow::End => Value::Undefined,
            Flow::Jump(p) => return self.fail(ErrorKind::InvalidBranch(p as i128)),
        };
        Ok(Execution {
            value,
            stack: Rc::try_unwrap(frame.store)
                .expect("isolated execution owns its store")
                .into_inner()
                .stack,
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
    fn materialize_host_result(
        &mut self,
        result: HostResult,
        host: &mut impl Host,
    ) -> Result<Value> {
        let fields = match result {
            HostResult::Value(value) => {
                self.check_value(&value)?;
                return Ok(value);
            }
            HostResult::ObjectFields(fields) => fields,
        };
        // Charge a conservative complete input/storage bound before adopting
        // host-owned strings or allocating the fresh object. Setter effects have
        // their own ordinary budgets; a later failure is not transactional.
        if fields.len().saturating_add(3) > self.limits.properties.saturating_sub(self.properties) {
            return self.fail(ErrorKind::Budget("host result object fields"));
        }
        self.visit_members(fields.len())?;
        let remaining_units = self.limits.string_units.saturating_sub(self.string_units);
        let metadata_units = ["__proto__", "__constructor__", "constructor"]
            .iter()
            .map(|name| name.len())
            .sum::<usize>();
        let mut input_units = 0usize;
        let mut stored_name_units = 0usize;
        for (key, value) in &fields {
            let value_units = match value {
                Value::String(value) => value.len(),
                Value::Bool(_) => 0,
                _ => return self.fail(ErrorKind::Unsupported("host result object scalar profile")),
            };
            input_units = input_units
                .saturating_add(key.len())
                .saturating_add(value_units);
            stored_name_units = stored_name_units.saturating_add(key.len());
            if input_units
                .saturating_add(stored_name_units)
                .saturating_add(metadata_units)
                > remaining_units
            {
                return self.fail(ErrorKind::Budget("host result object string units"));
            }
        }
        let mut names = std::collections::BTreeSet::new();
        for (key, value) in &fields {
            if !key
                .first()
                .is_some_and(|v| matches!(*v, 65..=90 | 97..=122))
                || key
                    .iter()
                    .any(|v| !matches!(*v, 48..=57 | 65..=90 | 95 | 97..=122))
                || key.as_ref() == text("constructor").as_ref()
                || !names.insert(key.clone())
            {
                return self.fail(ErrorKind::Unsupported(
                    "host result object field name profile",
                ));
            }
            if let Value::String(value) = value {
                if value.contains(&0)
                    || char::decode_utf16(value.iter().copied()).any(|unit| unit.is_err())
                {
                    return self.fail(ErrorKind::Unsupported("host result object UTF-16 profile"));
                }
            }
        }
        let object = self.object_literal()?;
        self.string_units += input_units;
        for (key, value) in fields {
            self.write_member(object, key, value, host)?;
        }
        Ok(Value::Object(object))
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
        self.invoke_in_context(callable, this, args, host, InvocationContext::Ordinary)
    }
    fn invoke_in_context(
        &mut self,
        callable: Value,
        this: Value,
        args: &[Value],
        host: &mut impl Host,
        context: InvocationContext,
    ) -> Result<Value> {
        if self.depth >= self.limits.call_depth {
            return self.fail(ErrorKind::Budget("function call depth"));
        }
        let caller = (self.current, self.current_code, self.current_target);
        self.depth += 1;
        let result = self.invoke_inner(callable, this, args, host, context);
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
        context: InvocationContext,
    ) -> Result<Value> {
        if args.len() > self.limits.arguments {
            return self.fail(ErrorKind::Budget("arguments"));
        }
        self.check_value(&this)?;
        for arg in args {
            self.check_value(arg)?;
        }
        let callable = self.materialize_value(callable)?;
        let Value::Object(id) = callable else {
            return self.fail(ErrorKind::UnboundCall);
        };
        self.check_object(id)?;
        let constructor_context = match context {
            InvocationContext::Construction(constructor) => Some(constructor),
            _ => None,
        };
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
                    InvocationContext::Construction(constructor),
                );
            }
            Kind::AttachedSuper { constructor, call } => {
                if this != Value::Object(id) || !args.is_empty() {
                    return self.fail(ErrorKind::Unsupported(
                        "attached super requires unnamed zero-argument call",
                    ));
                }
                let captured = match &self.objects[constructor.0].kind {
                    Kind::Script(function) => function.target,
                    Kind::MovieClipConstructor => self.current_target.ok_or_else(|| {
                        self.error(ErrorKind::Unsupported(
                            "attached super requires active Environment",
                        ))
                    })?,
                    _ => {
                        return self
                            .fail(ErrorKind::Unsupported("attached super callable category"));
                    }
                };
                self.attached_invocation_context(call.receiver, captured)?;
                return self.invoke_in_context(
                    Value::Object(constructor),
                    Value::DisplayClip(call.receiver),
                    args,
                    host,
                    InvocationContext::Attached(call),
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
                    .call_result(HostCall {
                        origin: self.origin(),
                        context: self.current_target,
                        binding,
                        this,
                        args,
                        source_offset: self.current.map(|v| v.0),
                    })
                    .map_err(|s| self.error(ErrorKind::Host(s)))?;
                return self.materialize_host_result(value, host);
            }
            Kind::UnsupportedNative(name) => {
                return self.fail(ErrorKind::UnsupportedNativeFunction(name));
            }
            Kind::Script(_) if !self.saved_environments.is_empty() => {
                // Ordinary invocation currently owns a separate operand vector.
                // Saved-handler calls require the native shared-Environment
                // frame/cleanup contract before that representation is admitted.
                return self.fail(ErrorKind::Unsupported(
                    "script call after saved Environment adoption requires shared state profile",
                ));
            }
            Kind::Script(function) => function,
            Kind::PointConstructor => {
                return self.fail(ErrorKind::Unsupported("direct Point invocation profile"));
            }
            Kind::RectangleConstructor => {
                return self.fail(ErrorKind::Unsupported(
                    "direct Rectangle invocation profile",
                ));
            }
            Kind::StringConstructor => {
                return self.fail(ErrorKind::Unsupported("direct String invocation profile"));
            }
            Kind::StringMethod(method) => {
                return self.invoke_string_method(method, this, args);
            }
            Kind::FunctionMethod(method) => {
                if !self.callable(&this) {
                    return self.fail(ErrorKind::UnboundCall);
                }
                let receiver = match args.first() {
                    Some(Value::Object(receiver)) => Value::Object(*receiver),
                    Some(Value::DisplayClip(receiver)) => {
                        self.attached_apply_receiver(method, &this, *receiver, args)?;
                        Value::DisplayClip(*receiver)
                    }
                    _ => {
                        return self.fail(ErrorKind::Unsupported(
                            "Function method unsupported thisObject category",
                        ));
                    }
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
                if let Value::DisplayClip(clip) = receiver {
                    let call = AttachedSuperCall {
                        receiver: clip,
                        prototype: self.attached_info(clip)?.prototype,
                    };
                    return self.invoke_in_context(
                        this,
                        receiver,
                        &forwarded,
                        host,
                        InvocationContext::Attached(call),
                    );
                }
                return self.invoke(this, receiver, &forwarded, host);
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
                if matches!(this, Value::DisplayClip(_) | Value::DisplayTextField(_)) {
                    return self.fail(ErrorKind::Unsupported(
                        "attached display addProperty profile",
                    ));
                }
                if args
                    .iter()
                    .skip(1)
                    .any(|value| matches!(value, Value::DeferredBuiltin(_)))
                {
                    return self.fail(ErrorKind::Unsupported(
                        "addProperty deferred callable conversion",
                    ));
                }
                if args.iter().skip(1).any(|value| {
                    matches!(value, Value::Object(id)
                    if matches!(self.objects[id.0].kind, Kind::FunctionPrototype | Kind::ConstructorSuper { .. } | Kind::AttachedSuper { .. }))
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
                self.reject_string_mutation_chain(*receiver)?;
                self.reject_geometry_mutation_chain(*receiver)?;
                self.reject_geom_mutation_chain(*receiver)?;
                if matches!(self.objects[receiver.0].kind, Kind::MovieClip(_)) {
                    return self.fail(ErrorKind::Unsupported("MovieClip addProperty profile"));
                }
                if matches!(
                    self.objects[receiver.0].kind,
                    Kind::ExternalInterfaceConstructor | Kind::ExternalInterfacePrototype
                ) {
                    return self.fail(ErrorKind::Unsupported(
                        "ExternalInterface addProperty profile",
                    ));
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
                    // Applied-super unwrapping retains the attached owner,
                    // whose category2 takes the native callback's Undefined
                    // branch. It does not substitute category11 lazy storage.
                    Value::DisplayClip(_) => Value::Undefined,
                    _ => Value::Undefined,
                });
            }
            Kind::ExternalInterfaceConstructor => {
                if constructor_context.is_some() {
                    return self.fail(ErrorKind::Unsupported(
                        "ExternalInterface construction profile",
                    ));
                }
                // Verified native callback18c560 writes Undefined; no host call,
                // allocation or receiver-dependent effects are performed here.
                return Ok(Value::Undefined);
            }
            Kind::FunctionConversion => {
                if constructor_context.is_some() || args.len() != 1 {
                    return self.fail(ErrorKind::Unsupported(
                        "Function conversion arity/construction profile",
                    ));
                }
                if matches!(args[0], Value::DeferredBuiltin(_)) {
                    return self.fail(ErrorKind::Unsupported(
                        "Function conversion deferred builtin profile",
                    ));
                }
                if let Value::Object(argument) = args[0] {
                    if matches!(
                        self.objects[argument.0].kind,
                        Kind::ConstructorSuper { .. } | Kind::AttachedSuper { .. }
                    ) {
                        return self.fail(ErrorKind::Unsupported(
                            "Function conversion of super reference",
                        ));
                    }
                    if let Kind::Script(function) = &self.objects[argument.0].kind {
                        // Actual type1/type2 native getters preserve the function
                        // object but clear the reference's captured activation.
                        // Our object identity is safe only without that wrapper state.
                        if function.scopes.as_slice() != [function.target, self.global] {
                            return self.fail(ErrorKind::Unsupported(
                                "Function conversion captured activation/With profile",
                            ));
                        }
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
        if let Kind::AttachedOwner(clip) = self.objects[function.target.0].kind {
            let flags = self.attached_event_state(clip)?.target_flags;
            if flags & 0x80 == 0 || flags & 0x10 != 0 {
                return self.fail(ErrorKind::Unsupported(
                    "captured attached Environment requires live eligible target and bit4 clear",
                ));
            }
        }
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
        let super_reference = if let InvocationContext::Attached(call) = context
            && flags & 0x20 == 0
        {
            if this != Value::DisplayClip(call.receiver) {
                return self.fail(ErrorKind::Unsupported("attached super receiver identity"));
            }
            Some(self.prepare_attached_super(id, call)?)
        } else if constructor_context == Some(id)
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
        let (store, registers) = self.isolated_store(definition.register_count)?;
        let mut frame = Frame {
            store,
            registers,
            shared_calls: None,
            scopes,
            pool: function.pool.clone(),
            target,
            this: this.clone(),
            this_visible: flags & 2 == 0,
            locals: activation,
            kind: FrameKind::Ordinary,
        };
        for (index, param) in definition.parameters.iter().enumerate() {
            if let Some(register) = param.register.filter(|r| *r != 0) {
                // Native setup writes only supplied register parameters. An
                // absent duplicate destination must not erase an earlier arg.
                if let Some(value) = args.get(index) {
                    self.write_register(&mut frame, register, value.clone())?;
                }
            } else {
                let value = args.get(index).cloned().unwrap_or(Value::Undefined);
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
            (0x40, self.target_value(self.root)),
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
    fn attached_apply_receiver(
        &self,
        method: FunctionMethod,
        callable: &Value,
        receiver: AttachedClipId,
        args: &[Value],
    ) -> Result<()> {
        // This is the native live case-3 handle/interface bridge, not generic
        // object coercion. Normal apply remains an ordinary function invocation.
        if !matches!(method, FunctionMethod::Apply)
            || !matches!(
                args,
                [Value::DisplayClip(_), Value::Null | Value::Undefined]
            )
        {
            return self.fail(ErrorKind::Unsupported(
                "attached Function.apply requires explicit null/undefined argument list",
            ));
        }
        let Value::Object(callable) = callable else {
            return self.fail(ErrorKind::UnboundCall);
        };
        let Kind::Script(function) = &self.objects[callable.0].kind else {
            return self.fail(ErrorKind::Unsupported(
                "attached Function.apply requires ScriptFunction2",
            ));
        };
        if function.scopes.as_slice() != [function.target, self.global] {
            return self.fail(ErrorKind::Unsupported(
                "attached Function.apply captured activation/With profile",
            ));
        }
        let instruction = self.code[function.code.0].instruction_at(function.offset);
        if !matches!(instruction.map(|i| &i.operands),
            Some(Operands::Function(definition)) if definition.flags.is_some())
        {
            return self.fail(ErrorKind::Unsupported(
                "attached Function.apply requires ScriptFunction2",
            ));
        }
        self.attached_invocation_context(receiver, function.target)
    }
    fn attached_invocation_context(
        &self,
        receiver: AttachedClipId,
        captured: ObjectId,
    ) -> Result<()> {
        let Some(caller) = self.current_target else {
            return self.fail(ErrorKind::Unsupported(
                "attached invocation requires active attached Environment",
            ));
        };
        let (Kind::AttachedOwner(root), Kind::AttachedOwner(caller), Kind::AttachedOwner(captured)) = (
            &self.objects[self.root.0].kind,
            &self.objects[caller.0].kind,
            &self.objects[captured.0].kind,
        ) else {
            return self.fail(ErrorKind::Unsupported(
                "attached invocation requires one adopted attached root context",
            ));
        };
        for clip in [receiver, *caller, *captured] {
            self.check_attached(clip)?;
            if self.attached_event_state(clip)?.target_flags & 0x80 == 0 {
                return self.fail(ErrorKind::Unsupported(
                    "attached invocation requires live eligible targets",
                ));
            }
            let mut next = Some(clip);
            let mut found = false;
            for _ in 0..self.limits.prototype_depth {
                let Some(id) = next else { break };
                if id == *root {
                    found = true;
                    break;
                }
                next = self.attached_info(id)?.parent;
            }
            if !found {
                return self.fail(ErrorKind::Unsupported(
                    "attached invocation crosses adopted root context",
                ));
            }
        }
        Ok(())
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
        if frame.is_saved() && index > 3 {
            return Ok(());
        }
        frame
            .store
            .borrow_mut()
            .write_register(frame.registers, index, value)
            .map_err(|error| self.store_error(error))
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
        if count > frame.store.borrow().len() {
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
        if frame.is_saved() {
            return self.fail(ErrorKind::Unsupported(
                "saved action function definition/capture profile",
            ));
        }
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
    fn truth_value(&self, value: &Value) -> Result<bool> {
        if matches!(value, Value::DisplayClip(_) | Value::DisplayTextField(_)) {
            return self.fail(ErrorKind::Unsupported(
                "attached display Boolean conversion",
            ));
        }
        Ok(value.truthy())
    }
    fn strict_equal(&self, a: &Value, b: &Value) -> bool {
        if matches!(a, Value::DeferredBuiltin(_)) || matches!(b, Value::DeferredBuiltin(_)) {
            return matches!(
                (a, b),
                (Value::DeferredBuiltin(_), Value::DeferredBuiltin(_))
            );
        }
        a == b
    }
    fn display_null_pair(a: &Value, b: &Value) -> bool {
        matches!(
            (a, b),
            (
                Value::DisplayClip(_) | Value::DisplayTextField(_),
                Value::Undefined | Value::Null
            ) | (
                Value::Undefined | Value::Null,
                Value::DisplayClip(_) | Value::DisplayTextField(_)
            )
        )
    }
    fn equal(&mut self, a: &Value, b: &Value) -> Result<bool> {
        if matches!(a, Value::DeferredBuiltin(_)) {
            return Ok(matches!(b, Value::DeferredBuiltin(_)));
        }
        if let Value::DeferredBuiltin(name) = b {
            if matches!(a, Value::Bool(_) | Value::Number(_) | Value::String(_)) {
                // Native primitive conversion may create/publish the callable,
                // but yields a nonprimitive and comparison is still false.
                self.resolve_deferred_builtin(name)?;
            }
            return Ok(false);
        }
        // Native null-like classification returns before display coercion or
        // resolution for tag7 versus tag0/1. Ordinary Value cleanup is separate.
        if Self::display_null_pair(a, b) {
            return Ok(false);
        }
        if matches!(a, Value::DisplayClip(_) | Value::DisplayTextField(_))
            || matches!(b, Value::DisplayClip(_) | Value::DisplayTextField(_))
        {
            return self.fail(ErrorKind::Unsupported(
                "attached display equality semantics",
            ));
        }
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
            if frame.is_shared_script() {
                self.shared_script_instruction(frame, instruction)?;
            }
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
                            Literal::Register(r) if frame.is_saved() && *r > 3 => Value::Undefined,
                            Literal::Register(r) => frame
                                .store
                                .borrow()
                                .read_register(frame.registers, *r)
                                .map_err(|error| self.store_error(error))?,
                            Literal::Constant8(i) if frame.is_saved() => frame
                                .pool
                                .get(usize::from(*i))
                                .cloned()
                                .unwrap_or(Value::Undefined),
                            Literal::Constant8(i) => {
                                frame.pool.get(usize::from(*i)).cloned().ok_or_else(|| {
                                    self.error(ErrorKind::InvalidConstant(u16::from(*i)))
                                })?
                            }
                            Literal::Constant16(i) if frame.is_saved() => frame
                                .pool
                                .get(usize::from(*i))
                                .cloned()
                                .unwrap_or(Value::Undefined),
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
                    if let FrameKind::Saved { action, .. } = &mut frame.kind {
                        let origin = CodeOrigin {
                            code,
                            source_offset: instruction.span.offset,
                        };
                        action
                            .visit_constant_pool(origin, || {
                                pool.iter().map(|value| self.decode_text(value)).collect()
                            })
                            .map_err(|error| match error {
                                saved_state::PoolError::Decode(error) => error,
                                saved_state::PoolError::State(_) => {
                                    self.error(ErrorKind::Unsupported(
                                        "saved constant-pool source/state profile",
                                    ))
                                }
                            })?;
                        frame.pool = Arc::clone(action.pool());
                    } else {
                        let mut values = Vec::new();
                        for value in pool {
                            values.push(Value::String(self.decode_text(value)?));
                        }
                        frame.pool = Arc::new(values);
                    }
                }
                Operands::StoreRegister(r) => {
                    // Native modes3/4 ignore out-of-bank destinations entirely.
                    if frame.is_saved() && *r > 3 {
                        pc += 1;
                        continue;
                    }
                    let value = self.peek(frame, 0)?;
                    self.write_register(frame, *r, value)?;
                }
                Operands::Function(definition) => {
                    self.define(code, instruction.span.offset, definition, frame, host)?
                }
                Operands::Branch(branch) => {
                    if instruction.opcode == 0x99 || self.truth_value(&self.pop(frame)?)? {
                        jump =
                            Some(usize::try_from(branch.target).map_err(|_| {
                                self.error(ErrorKind::InvalidBranch(branch.target))
                            })?);
                    }
                }
                Operands::With(body) => {
                    if frame.is_saved() {
                        return self.fail(ErrorKind::Unsupported("saved action With scope"));
                    }
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
                        frame.store.borrow_mut().stack.pop();
                    }
                    0x4c => {
                        let value = self.peek(frame, 0)?;
                        self.push(frame, value)?;
                    }
                    0x4d => {
                        let a = self.pop(frame)?;
                        let b = self.pop(frame)?;
                        self.push(frame, a)?;
                        self.push(frame, b)?;
                    }
                    0x12 => {
                        let value = !self.truth_value(&self.pop(frame)?)?;
                        self.push(frame, Value::Bool(value))?;
                    }
                    0x10 | 0x11 => {
                        let a = self.truth_value(&self.pop(frame)?)?;
                        let b = self.truth_value(&self.pop(frame)?)?;
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
                        let value = self.get_variable(frame, &key, host)?;
                        self.push(frame, value)?;
                    }
                    0x1d | 0x3c => {
                        if frame.is_saved() && instruction.opcode == 0x1d {
                            self.saved_set_variable(frame, host)?;
                        } else {
                            let value = self.pop(frame)?;
                            let key = self.pop(frame)?;
                            let key = self.to_text(&key)?;
                            if frame.is_saved() {
                                self.simple_name(&key)?;
                                self.write_member(frame.target, key, value, host)?;
                            } else {
                                self.set_variable(
                                    frame,
                                    key,
                                    value,
                                    instruction.opcode == 0x3c,
                                    host,
                                )?;
                            }
                        }
                    }
                    0x41 => {
                        let key = self.pop(frame)?;
                        let key = self.to_text(&key)?;
                        if !frame.is_saved() {
                            self.set_variable(frame, key, Value::Undefined, true, host)?;
                        }
                    }
                    0x4e => {
                        let key = self.pop(frame)?;
                        let key = self.to_text(&key)?;
                        let object = self.member_owner(self.pop(frame)?)?;
                        let value = self.read_member(object, &key, host)?;
                        self.push(frame, value)?;
                    }
                    0x4f => {
                        let value = self.pop(frame)?;
                        let key = self.pop(frame)?;
                        let key = self.to_text(&key)?;
                        let object = self.member_owner(self.pop(frame)?)?;
                        self.write_member(object, key, value, host)?;
                    }
                    0x42 => {
                        let args = self.args(frame)?;
                        let array = self.array(&args)?;
                        self.push(frame, Value::Object(array))?;
                    }
                    0x43 => {
                        let Value::Number(count) = self.pop(frame)? else {
                            return self.fail(ErrorKind::Unsupported("InitObject count profile"));
                        };
                        if !count.is_finite() || count < 0.0 || count.fract() != 0.0 {
                            return self.fail(ErrorKind::InvalidArgumentCount);
                        }
                        if count > self.limits.arguments as f64 {
                            return self.fail(ErrorKind::Budget("object literal properties"));
                        }
                        let object = self.object_literal()?;
                        for _ in 0..count as usize {
                            let value = self.pop(frame)?;
                            let Value::String(key) = self.pop(frame)? else {
                                return self
                                    .fail(ErrorKind::Unsupported("InitObject name type profile"));
                            };
                            if key.iter().any(|unit| *unit == 0 || *unit > 127) {
                                return self
                                    .fail(ErrorKind::Unsupported("InitObject ASCII name profile"));
                            }
                            self.write_member(object, key, value, host)?;
                        }
                        self.push(frame, Value::Object(object))?;
                    }
                    0x3d => {
                        let name = self.pop(frame)?;
                        let key = if matches!(name, Value::DeferredBuiltin(_)) {
                            None
                        } else {
                            Some(self.to_text(&name)?)
                        };
                        let args = self.args(frame)?;
                        let callable = match key {
                            Some(key) => self.variable(frame, &key, host)?,
                            None => name,
                        };
                        let value =
                            self.invoke(callable, self.target_value(frame.target), &args, host)?;
                        self.push(frame, value)?;
                    }
                    0x52 if frame.is_saved() || frame.shared_calls.is_some() => {
                        self.shared_call_method(frame, host)?;
                    }
                    0x52 => {
                        let method = self.pop(frame)?;
                        let mut receiver = self.pop(frame)?;
                        let args = self.args(frame)?;
                        let mut primitive_string = false;
                        let callable = if matches!(method, Value::Undefined)
                            || matches!(&method,Value::String(s) if s.is_empty())
                        {
                            if !matches!(method, Value::Undefined)
                                && matches!(&receiver, Value::Object(id)
                                    if matches!(self.objects[id.0].kind, Kind::AttachedSuper { .. }))
                            {
                                return self.fail(ErrorKind::Unsupported(
                                    "attached super requires Undefined method name",
                                ));
                            }
                            if matches!(receiver, Value::DeferredBuiltin(_)) {
                                return self.fail(ErrorKind::Unsupported(
                                    "unnamed CallMethod deferred this binding",
                                ));
                            }
                            receiver.clone()
                        } else {
                            receiver = self.materialize_value(receiver)?;
                            let key = self.to_text(&method)?;
                            if let Value::String(input) = &receiver {
                                let object = self.box_string_for_named_call(input.clone())?;
                                receiver = Value::Object(object);
                                primitive_string = true;
                            }
                            let object = self.member_owner(receiver.clone())?;
                            self.read_member(object, &key, host)?
                        };
                        // This native receiver branch accepts an actual tag8
                        // method only. Generic invoke would otherwise resolve a
                        // returned deferred tag11, which is a different route.
                        if primitive_string
                            && !matches!(&callable, Value::Object(id)
                            if self.objects[id.0].kind.function_value())
                        {
                            return self.fail(ErrorKind::Unsupported(
                                "primitive String method requires exact function value",
                            ));
                        }
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
                                let id = self.member_owner(receiver)?;
                                self.read_member(id, &key, host)?
                            }
                        };
                        let args = self.args(frame)?;
                        let callable = self.materialize_value(callable)?;
                        let constructor = self.object_id(callable.clone())?;
                        if matches!(self.objects[constructor.0].kind, Kind::PointConstructor) {
                            if instruction.opcode != 0x53 || !args.is_empty() {
                                return self.fail(ErrorKind::Unsupported(
                                    "Point construction requires zero-argument NewMethod",
                                ));
                            }
                            let object = self.construct_point_zero(constructor)?;
                            self.push(frame, Value::Object(object))?;
                            pc += 1;
                            continue;
                        }
                        if matches!(self.objects[constructor.0].kind, Kind::RectangleConstructor) {
                            if instruction.opcode != 0x53 || !args.is_empty() {
                                return self.fail(ErrorKind::Unsupported(
                                    "Rectangle construction requires zero-argument NewMethod",
                                ));
                            }
                            let object = self.construct_rectangle_zero(constructor)?;
                            self.push(frame, Value::Object(object))?;
                            pc += 1;
                            continue;
                        }
                        if matches!(self.objects[constructor.0].kind, Kind::StringConstructor) {
                            return self.fail(ErrorKind::Unsupported(
                                "explicit String construction profile",
                            ));
                        }
                        if matches!(self.objects[constructor.0].kind, Kind::FunctionConversion) {
                            return self.fail(ErrorKind::Unsupported(
                                "Function dynamic construction profile",
                            ));
                        }
                        if matches!(
                            self.objects[constructor.0].kind,
                            Kind::ExternalInterfaceConstructor
                        ) {
                            return self.fail(ErrorKind::Unsupported(
                                "ExternalInterface construction profile",
                            ));
                        }
                        let prototype = self.get(constructor, "prototype")?;
                        let prototype = match self.materialize_value(prototype)? {
                            Value::Object(id) => Some(id),
                            Value::Undefined | Value::Null => None,
                            _ => {
                                return self.fail(ErrorKind::Unsupported(
                                    "primitive constructor prototype",
                                ));
                            }
                        };
                        if matches!(self.objects[constructor.0].kind, Kind::ObjectConstructor) {
                            if instruction.opcode != 0x40 || !args.is_empty() {
                                return self.fail(ErrorKind::Unsupported(
                                    "Object construction argument/method profile",
                                ));
                            }
                            // Raw NewObject argc0 joins the same original native
                            // allocation/callback metadata path as InitObject.
                            // Direct Object() calls retain their separate profile.
                            let object = self.object_literal()?;
                            self.push(frame, Value::Object(object))?;
                        } else if matches!(self.objects[constructor.0].kind, Kind::ArrayConstructor)
                        {
                            // Native standard constructors select their constructed object;
                            // this is distinct from a script constructor's return value.
                            let object = self.invoke(
                                callable,
                                self.target_value(frame.target),
                                &args,
                                host,
                            )?;
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
                                InvocationContext::Construction(constructor),
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
                                    | Kind::UnsupportedNative(_)
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
                            > self
                                .limits
                                .stack_values
                                .saturating_sub(frame.store.borrow().len())
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
                        if self.attached_visibility_policy.is_some()
                            && property == MovieProperty::Visible
                            && match &target {
                                Value::DisplayClip(_) => true,
                                Value::Object(id) => self.objects.get(id.0).is_some_and(|object| {
                                    matches!(object.kind, Kind::AttachedOwner(_))
                                }),
                                Value::String(path) if path.is_empty() => matches!(
                                    self.objects[frame.target.0].kind,
                                    Kind::AttachedOwner(_)
                                ),
                                _ => false,
                            }
                        {
                            return self.fail(ErrorKind::Unsupported(
                                "indexed visibility bypass of bound attached state",
                            ));
                        }
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
                            if !matches!(a, Value::DeferredBuiltin(_))
                                && !matches!(b, Value::DeferredBuiltin(_))
                                && !Self::display_null_pair(&a, &b)
                                && (matches!(a, Value::DisplayClip(_) | Value::DisplayTextField(_))
                                    || matches!(
                                        b,
                                        Value::DisplayClip(_) | Value::DisplayTextField(_)
                                    ))
                            {
                                return self.fail(ErrorKind::Unsupported(
                                    "attached display strict equality semantics",
                                ));
                            }
                            self.strict_equal(&a, &b)
                        } else {
                            // Native left is the lower operand, right is top;
                            // token materialization makes this order observable.
                            self.equal(&b, &a)?
                        };
                        self.push(frame, Value::Bool(value))?;
                    }
                    0x47 => {
                        let a = self.pop(frame)?;
                        let b = self.pop(frame)?;
                        if matches!(a, Value::DeferredBuiltin(_))
                            || matches!(b, Value::DeferredBuiltin(_))
                        {
                            return self.fail(ErrorKind::Unsupported(
                                "Add2 deferred primitive conversion order",
                            ));
                        }
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
                        if matches!(a, Value::DeferredBuiltin(_))
                            || matches!(b, Value::DeferredBuiltin(_))
                        {
                            return self.fail(ErrorKind::Unsupported(
                                "comparison deferred primitive conversion order",
                            ));
                        }
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
                            Value::DeferredBuiltin(_) => "undefined",
                            Value::Undefined => "undefined",
                            Value::Null => "null",
                            Value::Bool(_) => "boolean",
                            Value::Number(_) => "number",
                            Value::String(_) => "string",
                            Value::DisplayClip(_) | Value::DisplayTextField(_) => {
                                return self.fail(ErrorKind::Unsupported(
                                    "attached display typeof semantics",
                                ));
                            }
                            Value::Object(id) => {
                                self.check_object(id)?;
                                if matches!(
                                    self.objects[id.0].kind,
                                    Kind::FunctionPrototype
                                        | Kind::ConstructorSuper { .. }
                                        | Kind::AttachedSuper { .. }
                                        | Kind::MovieClip(_)
                                        | Kind::StringObject(_)
                                        | Kind::PointObject(_)
                                        | Kind::RectangleObject(_)
                                ) {
                                    return self.fail(ErrorKind::Unsupported(
                                        "special prototype/super typeof semantics",
                                    ));
                                }
                                if matches!(
                                    self.objects[id.0].kind,
                                    Kind::Script(_)
                                        | Kind::Native(_)
                                        | Kind::UnsupportedNative(_)
                                        | Kind::ObjectConstructor
                                        | Kind::ArrayConstructor
                                        | Kind::FunctionConversion
                                        | Kind::MovieClipConstructor
                                        | Kind::ExternalInterfaceConstructor
                                        | Kind::PointConstructor
                                        | Kind::RectangleConstructor
                                        | Kind::StringConstructor
                                        | Kind::StringMethod(_)
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

#[derive(Clone, Copy)]
struct HashSlot {
    key: usize,
    next: Option<usize>,
}
fn hash_work(work: &mut usize, limit: usize, count: usize) -> std::result::Result<(), ErrorKind> {
    if count > limit.saturating_sub(*work) {
        return Err(ErrorKind::Budget("property hash reconstruction work"));
    }
    *work += count;
    Ok(())
}
fn ascii_property_hash(key: &Text) -> std::result::Result<usize, ErrorKind> {
    let mut hash = 5381u64;
    for unit in key.iter().rev() {
        let byte = u8::try_from(*unit)
            .ok()
            .filter(|b| b.is_ascii())
            .ok_or(ErrorKind::Unsupported("native non-ASCII property hash"))?;
        hash = hash.wrapping_mul(33) ^ u64::from(byte.to_ascii_lowercase());
    }
    Ok((hash & 0x00ff_ffff) as usize)
}
fn place_hash_key(
    table: &mut [Option<HashSlot>],
    hashes: &[usize],
    key: usize,
    work: &mut usize,
    limit: usize,
) -> std::result::Result<(), ErrorKind> {
    let mask = table.len() - 1;
    let home = hashes[key] & mask;
    hash_work(work, limit, 1)?;
    let Some(old) = table[home] else {
        table[home] = Some(HashSlot { key, next: None });
        return Ok(());
    };
    let mut free = (home + 1) & mask;
    while table[free].is_some() {
        hash_work(work, limit, 1)?;
        free = (free + 1) & mask;
        if free == home {
            return Err(ErrorKind::Unsupported("full reconstructed property hash"));
        }
    }
    table[free] = Some(old);
    let old_home = hashes[old.key] & mask;
    let next = if old_home == home {
        Some(free)
    } else {
        let mut prior = old_home;
        loop {
            hash_work(work, limit, 1)?;
            let Some(entry) = table[prior] else {
                return Err(ErrorKind::Unsupported(
                    "invalid reconstructed property hash chain",
                ));
            };
            if entry.next == Some(home) {
                table[prior].as_mut().unwrap().next = Some(free);
                break;
            }
            prior = entry.next.ok_or(ErrorKind::Unsupported(
                "missing reconstructed property hash link",
            ))?;
        }
        None
    };
    table[home] = Some(HashSlot { key, next });
    Ok(())
}
/// A fresh native ASCII table reconstructed from complete insertion history.
/// Values/flags may change without moving slots. Deletion histories stay gated.
fn native_hash_order(
    keys: &[Text],
    budget: usize,
) -> std::result::Result<(Vec<usize>, usize), ErrorKind> {
    let mut work = 0;
    let mut hashes = Vec::with_capacity(keys.len());
    for key in keys {
        hash_work(&mut work, budget, key.len().saturating_add(1))?;
        hashes.push(ascii_property_hash(key)?);
    }
    let mut table: Vec<Option<HashSlot>> = Vec::new();
    for key in 0..keys.len() {
        if table.is_empty() || (key as u128) * 5 > (table.len() as u128) * 4 {
            let capacity = if table.is_empty() {
                8
            } else {
                table
                    .len()
                    .checked_mul(2)
                    .ok_or(ErrorKind::Budget("property hash capacity"))?
            };
            hash_work(&mut work, budget, capacity)?;
            let mut grown = vec![None; capacity];
            for entry in &table {
                hash_work(&mut work, budget, 1)?;
                if let Some(entry) = entry {
                    place_hash_key(&mut grown, &hashes, entry.key, &mut work, budget)?;
                }
            }
            table = grown;
        }
        place_hash_key(&mut table, &hashes, key, &mut work, budget)?;
    }
    hash_work(&mut work, budget, table.len())?;
    Ok((
        table.into_iter().flatten().map(|entry| entry.key).collect(),
        work,
    ))
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
    pub(super) fn push(values: &[Value]) -> Vec<u8> {
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
    fn object_native_inventory_has_exact_flags_and_explicit_callback_gates() {
        let mut vm = setup();
        let constructor = vm.install_object_constructor().unwrap();
        let Value::Object(prototype) = vm.get(constructor, "prototype").unwrap() else {
            panic!()
        };
        assert_eq!(vm.objects[prototype.0].properties.len(), 9);
        assert_eq!(vm.get(prototype, "__proto__").unwrap(), Value::Null);
        assert_eq!(vm.own_property_flags(prototype, "__proto__").unwrap(), None);
        assert_eq!(
            vm.own_property_flags(prototype, "constructor").unwrap(),
            Some(3)
        );
        assert_eq!(
            vm.own_property_flags(prototype, "addProperty").unwrap(),
            Some(1)
        );
        assert!(vm.enumerate_names(prototype).unwrap().is_empty());
        let instance = vm.ordinary_object().unwrap();
        assert!(vm.enumerate_names(instance).unwrap().is_empty());
        for key in [
            "hasOwnProperty",
            "watch",
            "unwatch",
            "isPropertyEnumerable",
            "isPrototypeOf",
            "toString",
            "valueOf",
        ] {
            let method = vm.get(instance, key).unwrap();
            assert!(vm.callable(&method));
            assert_eq!(vm.own_property_flags(prototype, key).unwrap(), Some(1));
            assert!(vm.has_key(instance, &text(key)).unwrap());
            assert!(matches!(
                vm.call(method, Value::Object(instance), &[], &mut RejectHost)
                    .unwrap_err()
                    .kind,
                ErrorKind::UnsupportedNativeFunction(_)
            ));
            assert_eq!(vm.host_calls, 0);
        }
    }
    #[test]
    fn init_object_has_three_hidden_metadata_entries_and_guards_native_binding() {
        let mut vm = setup();
        let constructor = vm.install_object_constructor().unwrap();
        let mut bytes = push(&[
            Value::string("a"),
            Value::Number(1.0),
            Value::string("b"),
            Value::Number(2.0),
            Value::Number(2.0),
        ]);
        bytes.push(0x43);
        let literal = execute(&mut vm, &bytes).unwrap();
        let Value::Object(literal) = literal.stack[0] else {
            panic!()
        };
        assert_eq!(vm.objects[literal.0].properties.len(), 5);
        for key in ["__proto__", "__constructor__", "constructor"] {
            assert_eq!(vm.own_property_flags(literal, key).unwrap(), Some(3));
        }
        for key in ["__constructor__", "constructor"] {
            assert_eq!(vm.get(literal, key).unwrap(), Value::Object(constructor));
        }
        assert_eq!(vm.get(literal, "a").unwrap(), Value::Number(1.0));
        assert_eq!(vm.own_property_flags(literal, "b").unwrap(), Some(0));
        let mut overridden = push(&[
            Value::string("constructor"),
            Value::Number(7.0),
            Value::Number(1.0),
        ]);
        overridden.push(0x43);
        let value = execute(&mut vm, &overridden).unwrap();
        let Value::Object(overridden) = value.stack[0] else {
            panic!()
        };
        assert_eq!(
            vm.get(overridden, "constructor").unwrap(),
            Value::Number(7.0)
        );
        assert_eq!(
            vm.own_property_flags(overridden, "constructor").unwrap(),
            Some(3)
        );
        let replacement = vm.native(42).unwrap();
        vm.set(vm.global(), "Object", Value::Object(replacement))
            .unwrap();
        let count = vm.object_count();
        assert!(matches!(
            execute(&mut vm, &bytes).unwrap_err().kind,
            ErrorKind::Unsupported(_)
        ));
        assert_eq!(vm.object_count(), count);
    }
    #[test]
    fn init_object_uses_inherited_setter_and_ignores_its_return_value() {
        let mut vm = setup();
        let object = vm.install_object_constructor().unwrap();
        let Value::Object(prototype) = vm.get(object, "prototype").unwrap() else {
            panic!()
        };
        let mut getter = push(&[Value::Number(0.0)]);
        getter.push(0x3e);
        let mut setter = push(&[Value::string("this")]);
        setter.push(0x1c);
        setter.extend(push(&[Value::string("seen"), Value::string("arguments")]));
        setter.push(0x1c);
        setter.extend(push(&[Value::string("0")]));
        setter.extend([0x4e, 0x4f]);
        setter.extend(push(&[Value::Bool(false)]));
        setter.push(0x3e);
        let mut definitions = function("getter", &getter);
        definitions.extend(function("setter", &setter));
        execute(&mut vm, &definitions).unwrap();
        let getter = vm.get(vm.root(), "getter").unwrap();
        let setter = vm.get(vm.root(), "setter").unwrap();
        let add = vm.get(prototype, "addProperty").unwrap();
        vm.call(
            add,
            Value::Object(prototype),
            &[Value::string("watched"), getter.clone(), setter.clone()],
            &mut RejectHost,
        )
        .unwrap();
        let mut literal = push(&[
            Value::string("watched"),
            Value::Number(7.0),
            Value::Number(1.0),
        ]);
        literal.push(0x43);
        let result = execute(&mut vm, &literal).unwrap();
        assert_eq!(result.host_calls, 0);
        let Value::Object(instance) = result.stack[0] else {
            panic!()
        };
        assert_eq!(vm.get(instance, "seen").unwrap(), Value::Number(7.0));
        assert_eq!(vm.own_property_flags(instance, "watched").unwrap(), None);
        vm.raw_member(
            prototype,
            text("watched"),
            Property::Accessor {
                getter,
                setter: Some(setter),
            },
            4,
        )
        .unwrap();
        assert!(matches!(
            execute(&mut vm, &literal).unwrap_err().kind,
            ErrorKind::Unsupported("flagged accessor assignment native ordering")
        ));
    }
    #[test]
    fn native_ascii_hash_and_literal_enumeration_match_static_retail_derivation() {
        let names = [
            "__proto__",
            "__constructor__",
            "constructor",
            "preferredWidth",
            "preferredHeight",
            "maxWidth",
            "maxHeight",
            "minWidth",
            "minHeight",
            "visible",
        ];
        let keys: Vec<_> = names.iter().map(|name| text(name)).collect();
        assert_eq!(ascii_property_hash(&text("__proto__")).unwrap(), 0x8db693);
        assert_eq!(ascii_property_hash(&text("VISIBLE")).unwrap(), 0x149c0b);
        let (order, _) = native_hash_order(&keys, 10_000).unwrap();
        assert_eq!(order, vec![8, 1, 0, 3, 5, 7, 9, 2, 6, 4]);
        assert!(matches!(
            native_hash_order(&keys, 4).unwrap_err(),
            ErrorKind::Budget(_)
        ));
        assert!(native_hash_order(&[text("é")], 100).is_err());
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        let mut values = Vec::new();
        for name in names[3..].iter().rev() {
            values.extend([Value::string(name), Value::Number(1.0)]);
        }
        values.push(Value::Number(7.0));
        let mut code = push(&values);
        code.push(0x43);
        let made = execute(&mut vm, &code).unwrap();
        let Value::Object(object) = made.stack[0] else {
            panic!()
        };
        assert_eq!(vm.objects[object.0].insertion_keys, keys);
        let expected: Vec<_> = [
            "minHeight",
            "preferredWidth",
            "maxWidth",
            "minWidth",
            "visible",
            "maxHeight",
            "preferredHeight",
        ]
        .iter()
        .map(|name| text(name))
        .collect();
        assert_eq!(vm.enumerate_names(object).unwrap(), expected);
        vm.set(object, "visible", Value::Number(2.0)).unwrap();
        assert_eq!(vm.enumerate_names(object).unwrap(), expected);
        assert_eq!(vm.objects[object.0].insertion_keys.len(), 10);
        assert!(vm.delete_member(object, &text("visible")).unwrap());
        assert!(matches!(
            vm.enumerate_names(object).unwrap_err().kind,
            ErrorKind::Unsupported(_)
        ));
    }
    #[test]
    fn readonly_callability_and_builtin_context_membership_do_not_resolve_or_mutate() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        let clip = vm
            .create_attached_movie_clip(AttachedClipSpec {
                target: DisplayTargetId(912),
                source: AttachedClipSource {
                    movie: LoadedMovieSourceId(2),
                    sprite_id: 3,
                    header_version: 15,
                },
                parent: None,
                event_state: AttachedEventState {
                    target_flags: 0x5200_0180,
                    load_flags: 0,
                    source_flags: 0,
                },
            })
            .unwrap();
        let owner = vm.attached_owner(clip).unwrap();
        let plain = vm.object().unwrap();
        let native = vm.native(97).unwrap();
        let unsupported = vm
            .alloc(Kind::UnsupportedNative("unimplemented test method"), None)
            .unwrap();
        let token = vm
            .register_deferred_builtin(vm.global(), "DeferredTest", BuiltinFactory::Unsupported, 1)
            .unwrap();
        let before = (
            vm.object_count(),
            vm.properties,
            vm.string_units,
            vm.host_calls,
            vm.builtin_targets.len(),
        );
        for value in [
            Value::Object(native),
            Value::Object(unsupported),
            Value::Object(vm.object_constructor.unwrap()),
        ] {
            assert_eq!(
                vm.classify_callability(&value).unwrap(),
                ValueCallability::Callable
            );
        }
        for value in [
            Value::Object(plain),
            Value::DisplayClip(clip),
            Value::Undefined,
            Value::Null,
            Value::Bool(true),
            Value::Number(1.0),
            Value::string("function"),
        ] {
            assert_eq!(
                vm.classify_callability(&value).unwrap(),
                ValueCallability::NonCallable
            );
        }
        assert_eq!(
            vm.classify_callability(&token).unwrap(),
            ValueCallability::DeferredBuiltin
        );
        assert_eq!(vm.get(vm.global(), "DeferredTest").unwrap(), token);
        for value in [
            Value::Object(ObjectId(usize::MAX)),
            Value::DisplayClip(AttachedClipId(usize::MAX)),
            Value::Object(owner),
        ] {
            assert!(vm.classify_callability(&value).is_err());
        }
        assert!(!vm.is_bound_builtin_context_target(owner).unwrap());
        assert!(!vm.is_bound_builtin_context_target(plain).unwrap());
        assert!(vm.is_bound_builtin_context_target(native).is_err());
        assert!(
            vm.is_bound_builtin_context_target(ObjectId(usize::MAX))
                .is_err()
        );
        assert_eq!(
            (
                vm.object_count(),
                vm.properties,
                vm.string_units,
                vm.host_calls,
                vm.builtin_targets.len()
            ),
            before
        );
        vm.bind_builtin_context_target(owner).unwrap();
        vm.bind_builtin_context_target(plain).unwrap();
        assert!(vm.is_bound_builtin_context_target(owner).unwrap());
        assert!(vm.is_bound_builtin_context_target(plain).unwrap());
        assert!(!vm.is_bound_builtin_context_target(vm.global()).unwrap());
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
    struct ObjectResultHost {
        fields: Vec<(Text, Value)>,
        calls: Vec<(u32, Value, Vec<Value>)>,
    }
    #[derive(Default)]
    struct ContextHost {
        calls: Vec<(u32, Option<ObjectId>, Value, Option<CodeOrigin>)>,
        fail_binding: Option<u32>,
        fields_binding: Option<u32>,
    }
    impl Host for ContextHost {
        fn command(&mut self, _: ObjectId, _: HostCommand) -> std::result::Result<(), String> {
            Err("unsupported test command".into())
        }
        fn call(&mut self, call: HostCall<'_>) -> std::result::Result<Value, String> {
            self.calls
                .push((call.binding, call.context, call.this, call.origin));
            if self.fail_binding == Some(call.binding) {
                Err("controlled native failure".into())
            } else {
                Ok(Value::Undefined)
            }
        }
        fn call_result(&mut self, call: HostCall<'_>) -> std::result::Result<HostResult, String> {
            let fields = self.fields_binding == Some(call.binding);
            self.call(call).map(|value| {
                if fields {
                    HostResult::ObjectFields(vec![(text("field"), Value::string("payload"))])
                } else {
                    HostResult::Value(value)
                }
            })
        }
    }
    fn method_call(receiver: &str, method: &str) -> Vec<u8> {
        let mut bytes = push(&[Value::Number(0.0), Value::string(receiver)]);
        bytes.push(0x1c);
        bytes.extend(push(&[Value::string(method)]));
        bytes.push(0x52);
        bytes
    }
    fn variable_bytes(name: &str) -> Vec<u8> {
        let mut bytes = push(&[Value::string(name)]);
        bytes.push(0x1c);
        bytes
    }
    fn typed_field_fixture() -> (Vm, AttachedClipId, AttachedClipId, AttachedTextFieldId) {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        let mut make_clip = |target, parent, marked| {
            vm.create_attached_movie_clip(AttachedClipSpec {
                target: DisplayTargetId(target),
                source: AttachedClipSource {
                    movie: LoadedMovieSourceId(1),
                    sprite_id: 1,
                    header_version: 15,
                },
                parent,
                event_state: AttachedEventState {
                    target_flags: 0x5200_0180 | if marked { 0x400 } else { 0 },
                    load_flags: 0,
                    source_flags: 0,
                },
            })
            .unwrap()
        };
        let root = make_clip(1, None, true);
        let caller = make_clip(2, Some(root), false);
        vm.adopt_attached_root(root).unwrap();
        // Entirely synthetic same-version routing fixture, not UISource11.
        let field = vm
            .adopt_attached_text_field(AttachedTextFieldSpec {
                target: DisplayTargetId(3),
                source: AttachedTextFieldSource {
                    movie: LoadedMovieSourceId(2),
                    edit_text_id: 1,
                    header_version: 15,
                },
                parent: root,
                environment_owner: root,
                target_flags_6a: 0x5200_0180,
                prototype: TextFieldPrototypeId(1),
                watch_state: TextFieldWatchState::Absent,
            })
            .unwrap();
        vm.set_attached_children(
            root,
            vec![
                AttachedChild::Clip {
                    name: text("gfx"),
                    clip: caller,
                },
                AttachedChild::TextField {
                    name: text("localizedText_tf"),
                    field,
                },
            ],
        )
        .unwrap();
        vm.set_attached_children(caller, vec![]).unwrap();
        vm.set(
            vm.global(),
            "fixtureField",
            vm.attached_text_field_value(field).unwrap(),
        )
        .unwrap();
        (vm, root, caller, field)
    }

    #[test]
    fn typed_text_field_opcodes_preserve_receiver_environment_and_shadowing() {
        #[derive(Default)]
        struct FieldHost(Vec<(TextFieldMemberRequest, Option<Value>)>);
        impl Host for FieldHost {
            fn call(&mut self, _: HostCall<'_>) -> std::result::Result<Value, String> {
                Err("unexpected native call".into())
            }
            fn command(&mut self, _: ObjectId, _: HostCommand) -> std::result::Result<(), String> {
                Err("unexpected command".into())
            }
            fn get_text_field_member(
                &mut self,
                request: TextFieldMemberRequest,
            ) -> std::result::Result<Value, String> {
                self.0.push((request, None));
                Ok(Value::string("controlled read"))
            }
            fn set_text_field_member(
                &mut self,
                request: TextFieldMemberRequest,
                value: Value,
            ) -> std::result::Result<(), String> {
                self.0.push((request, Some(value)));
                Ok(())
            }
        }
        let (mut vm, root, caller, field) = typed_field_fixture();
        assert_eq!(
            vm.read_attached_member(root, "localizedText_tf").unwrap(),
            AttachedMember::Present(Value::DisplayTextField(field))
        );
        let get_field = || {
            let mut bytes = variable_bytes("_global");
            bytes.extend(push(&[Value::string("fixtureField")]));
            bytes.push(0x4e);
            bytes
        };
        let mut bytes = get_field();
        bytes.extend(push(&[Value::string("html"), Value::Bool(true)]));
        bytes.push(0x4f);
        bytes.extend(get_field());
        bytes.extend(push(&[Value::string("text")]));
        bytes.push(0x4e);
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        let caller_owner = vm.attached_owner(caller).unwrap();
        let mut host = FieldHost::default();
        let result = vm.execute(code, caller_owner, &mut host).unwrap();
        assert_eq!(result.stack, vec![Value::string("controlled read")]);
        assert_eq!(result.host_calls, 2);
        let info = vm.attached_text_field_info(field).unwrap();
        assert!(info.storage.is_some());
        for (request, _) in &host.0 {
            assert_eq!(request.field, field);
            assert_eq!(request.owner, info.owner);
            assert_eq!(request.target, info.target);
            assert_eq!(request.source, info.source);
            assert_eq!(request.context, Some(caller_owner));
            assert_eq!(request.environment_owner, vm.attached_owner(root).unwrap());
            assert_ne!(request.context, Some(request.environment_owner));
        }
        assert_eq!(host.0[0].0.member, TextFieldMember::Html);
        assert_eq!(host.0[0].1, Some(Value::Bool(true)));
        assert_eq!(host.0[1].0.member, TextFieldMember::Text);
        assert!(vm.get(info.owner, "text").is_err());
        assert!(vm.set(info.owner, "text", Value::string("x")).is_err());
        assert!(
            vm.set(vm.global(), "forged", Value::Object(info.owner))
                .is_err()
        );
        assert!(vm.execute(code, info.owner, &mut host).is_err());
        vm.write_attached_member(root, "localizedText_tf", Value::Undefined)
            .unwrap();
        assert_eq!(
            vm.read_attached_member(root, "localizedText_tf").unwrap(),
            AttachedMember::Present(Value::Undefined)
        );
        assert_eq!(
            vm.attached_text_field_info(field).unwrap().storage,
            info.storage
        );
        assert_eq!(host.0.len(), 2);
    }

    #[test]
    fn typed_display_null_comparisons_do_not_coerce_or_dispatch() {
        let (mut vm, root, caller, _) = typed_field_fixture();
        vm.set(vm.global(), "fixtureClip", vm.attached_value(root).unwrap())
            .unwrap();
        let caller_owner = vm.attached_owner(caller).unwrap();
        for alias in ["fixtureField", "fixtureClip"] {
            for opcode in [0x49, 0x66] {
                for null in [Value::Undefined, Value::Null] {
                    for reverse in [false, true] {
                        let mut load = variable_bytes("_global");
                        load.extend(push(&[Value::string(alias)]));
                        load.push(0x4e);
                        let mut bytes = if reverse {
                            push(&[null.clone()])
                        } else {
                            vec![]
                        };
                        bytes.extend(load);
                        if !reverse {
                            bytes.extend(push(&[null.clone()]));
                        }
                        bytes.push(opcode);
                        let code = vm
                            .register(Arc::new(Program::parse(&bytes).unwrap()))
                            .unwrap();
                        let result = vm.execute(code, caller_owner, &mut RejectHost).unwrap();
                        assert_eq!(result.stack, vec![Value::Bool(false)]);
                        assert_eq!(result.host_calls, 0);
                    }
                }
            }
        }
        let mut bytes = variable_bytes("_global");
        bytes.extend(push(&[Value::string("fixtureField")]));
        bytes.extend([0x4e, 0x4c, 0x49]); // General display/display still gated.
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        assert!(vm.execute(code, caller_owner, &mut RejectHost).is_err());
    }

    #[test]
    fn imported_receiver_does_not_replace_captured_script_environment() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        let make = |target, movie, version, parent| AttachedClipSpec {
            target: DisplayTargetId(target),
            source: AttachedClipSource {
                movie: LoadedMovieSourceId(movie),
                sprite_id: 1,
                header_version: version,
            },
            parent,
            event_state: AttachedEventState {
                target_flags: 0x5200_0180,
                load_flags: 0,
                source_flags: 0,
            },
        };
        let root = vm
            .create_attached_movie_clip(make(1, 15, 15, None))
            .unwrap();
        vm.adopt_attached_root(root).unwrap();
        let imported = vm
            .create_attached_movie_clip_with_binding(
                make(2, 11, 11, Some(root)),
                AttachedMovieBinding::LocalSourceOrImportedLocalExport,
            )
            .unwrap();
        vm.set_attached_children(
            root,
            vec![AttachedChild::Clip {
                name: text("button"),
                clip: imported,
            }],
        )
        .unwrap();
        vm.set_attached_children(imported, vec![]).unwrap();
        let observe = vm.native(97).unwrap();
        vm.set(vm.global(), "observe", Value::Object(observe))
            .unwrap();
        let mut body = method_call("_global", "observe");
        body.push(0x3e);
        let bytes = function("", &body);
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        let root_owner = vm.attached_owner(root).unwrap();
        let callable = vm
            .execute(code, root_owner, &mut RejectHost)
            .unwrap()
            .stack
            .pop()
            .unwrap();
        let receiver = vm.attached_value(imported).unwrap();
        let mut host = ContextHost::default();
        vm.call(callable.clone(), receiver.clone(), &[], &mut host)
            .unwrap();
        assert_eq!(host.calls.len(), 1);
        assert_eq!(host.calls[0].1, Some(root_owner));
        assert_eq!(
            vm.attached_info(imported)
                .unwrap()
                .environment_header_version,
            11
        );
        assert_eq!(
            vm.attached_info(root).unwrap().environment_header_version,
            15
        );
        let mut state = vm.attached_event_state(root).unwrap();
        state.target_flags |= 0x10;
        vm.update_attached_event_state(root, state).unwrap();
        assert!(vm.call(callable, receiver, &[], &mut host).is_err());
        assert_eq!(host.calls.len(), 1);
        assert!(vm.current_target.is_none());
    }

    #[test]
    fn focus_rect_opcode_records_origin_without_storage_or_host_dispatch() {
        let (mut vm, root, caller, _) = typed_field_fixture();
        vm.bind_attached_focus_rect(
            root,
            AttachedFocusRectState {
                target_flags_b8: 0x1234_0180,
            },
        )
        .unwrap();
        let mut bytes = variable_bytes("_root");
        bytes.extend(push(&[Value::string("_focusrect"), Value::Bool(false)]));
        let setter_offset = bytes.len();
        bytes.push(0x4f);
        bytes.extend(variable_bytes("_root"));
        bytes.extend(push(&[Value::string("_focusrect")]));
        bytes.push(0x4e);
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        let result = vm
            .execute(code, vm.attached_owner(caller).unwrap(), &mut RejectHost)
            .unwrap();
        assert_eq!(result.stack, vec![Value::Bool(false)]);
        assert_eq!(result.host_calls, 0);
        let snapshot = vm.attached_focus_rect(root).unwrap().unwrap();
        assert_eq!(snapshot.state.target_flags_b8, 0x1234_0100);
        assert_eq!(
            snapshot.last_write,
            Some(CodeOrigin {
                code,
                source_offset: setter_offset
            })
        );
        assert!(vm.attached_info(root).unwrap().storage.is_none());
        assert!(vm.attached_focus_rect(caller).unwrap().is_none());
        // Public host writes carry no stale completed-interpreter provenance.
        vm.write_attached_member(root, "_focusrect", Value::Bool(false))
            .unwrap();
        assert!(
            vm.attached_focus_rect(root)
                .unwrap()
                .unwrap()
                .last_write
                .is_none()
        );
    }
    #[test]
    fn tab_enabled_opcode_keeps_receiver_status_and_focus_provenance_separate() {
        let (mut vm, root, caller, _) = typed_field_fixture();
        vm.bind_attached_target_status(root, 0x1234_01e8).unwrap();
        vm.bind_attached_target_status(caller, 0x5678_01e8).unwrap();
        let mut bytes = variable_bytes("_root");
        bytes.extend(push(&[Value::string("_focusrect"), Value::Bool(false)]));
        let focus_offset = bytes.len();
        bytes.push(0x4f);
        bytes.extend(variable_bytes("_root"));
        bytes.extend(push(&[Value::string("tabEnabled"), Value::Bool(false)]));
        let tab_offset = bytes.len();
        bytes.push(0x4f);
        bytes.extend(variable_bytes("_root"));
        bytes.extend(push(&[Value::string("tabEnabled")]));
        bytes.push(0x4e);
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        let root_event = vm.attached_event_state(root).unwrap();
        let count = vm.object_count();
        let result = vm
            .execute(code, vm.attached_owner(caller).unwrap(), &mut RejectHost)
            .unwrap();
        assert_eq!(result.stack, vec![Value::Bool(false)]);
        assert_eq!(result.host_calls, 0);
        assert_eq!(vm.object_count(), count);
        assert!(vm.attached_info(root).unwrap().storage.is_none());
        assert_eq!(vm.attached_target_status(root).unwrap(), Some(0x1234_0148));
        assert_eq!(
            vm.attached_target_status(caller).unwrap(),
            Some(0x5678_01e8)
        );
        assert_eq!(vm.attached_event_state(root).unwrap(), root_event);
        assert_eq!(
            vm.attached_focus_rect(root).unwrap().unwrap().last_write,
            Some(CodeOrigin {
                code,
                source_offset: focus_offset
            })
        );
        assert_eq!(
            vm.attached_tab_enabled(root).unwrap().unwrap().last_write,
            Some(CodeOrigin {
                code,
                source_offset: tab_offset
            })
        );
        vm.write_attached_member(root, "tabEnabled", Value::Bool(false))
            .unwrap();
        assert!(
            vm.attached_tab_enabled(root)
                .unwrap()
                .unwrap()
                .last_write
                .is_none()
        );
        assert_eq!(
            vm.attached_focus_rect(root).unwrap().unwrap().last_write,
            Some(CodeOrigin {
                code,
                source_offset: focus_offset
            })
        );
    }
    #[test]
    fn deferred_scalars_and_token_equality_do_not_materialize() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        let a = vm
            .register_deferred_builtin(vm.global(), "Stage", BuiltinFactory::Unsupported, 1)
            .unwrap();
        let b = vm
            .register_deferred_builtin(vm.global(), "Mouse", BuiltinFactory::Unsupported, 1)
            .unwrap();
        assert_ne!(a, b); // Stored-name inspection is not ActionStrictEquals.
        let before = vm.object_count();
        for (opcode, expected) in [
            (0x44, Value::string("undefined")),
            (0x4a, Value::Number(0.0)),
            (0x4b, Value::string("[type Function]")),
            (0x12, Value::Bool(false)),
        ] {
            let mut bytes = variable_bytes("Stage");
            bytes.push(opcode);
            let result = execute(&mut vm, &bytes).unwrap();
            assert_eq!(result.stack, vec![expected]);
            assert_eq!(result.host_calls, 0);
        }
        let mut branch = variable_bytes("Stage");
        let mut false_branch = push(&[Value::Bool(false)]);
        false_branch.push(0x3e);
        branch.extend(action(0x9d, &(false_branch.len() as i16).to_le_bytes()));
        branch.extend(false_branch);
        branch.extend(push(&[Value::Bool(true)]));
        branch.push(0x3e);
        assert_eq!(execute(&mut vm, &branch).unwrap().value, Value::Bool(true));
        for opcode in [0x49, 0x66] {
            let mut bytes = variable_bytes("Stage");
            bytes.extend(variable_bytes("Mouse"));
            bytes.push(opcode);
            assert_eq!(
                execute(&mut vm, &bytes).unwrap().stack,
                vec![Value::Bool(true)]
            );
            let mut bytes = variable_bytes("Stage");
            bytes.extend(push(&[Value::Number(0.0)]));
            bytes.push(opcode);
            assert_eq!(
                execute(&mut vm, &bytes).unwrap().stack,
                vec![Value::Bool(false)]
            );
        }
        assert_eq!(vm.object_count(), before);
        assert_eq!(vm.get(vm.global(), "Stage").unwrap(), a);
        assert_eq!(vm.get(vm.global(), "Mouse").unwrap(), b);
        vm.bind_builtin_context_target(vm.root()).unwrap();
        let mut reversed = push(&[Value::Number(0.0)]);
        reversed.extend(variable_bytes("Stage"));
        reversed.push(0x49);
        assert!(matches!(
            execute(&mut vm, &reversed).unwrap_err().kind,
            ErrorKind::UnsupportedDeferredBuiltin(_)
        ));
        // Strict comparison of the same operands has no factory side effect.
        *reversed.last_mut().unwrap() = 0x66;
        assert_eq!(
            execute(&mut vm, &reversed).unwrap().stack,
            vec![Value::Bool(false)]
        );
    }

    #[test]
    fn primitive_string_call_uses_deferred_graph_and_fresh_typed_boxes() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        vm.bind_builtin_context_target(vm.root()).unwrap();
        let token = vm
            .register_deferred_builtin(vm.global(), "String", BuiltinFactory::String, 1)
            .unwrap();
        vm.set(vm.global(), "retainedStringToken", token.clone())
            .unwrap();
        let count = vm.object_count();
        let mut bytes = push(&[
            Value::Number(0.0),
            Value::string("aBc! 019"),
            Value::string("toUpperCase"),
        ]);
        bytes.push(0x52);
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        for iteration in 0..2 {
            let result = vm.execute(code, vm.root(), &mut RejectHost).unwrap();
            assert_eq!(result.stack, vec![Value::string("ABC! 019")]);
            assert_eq!(result.host_calls, 0);
            assert_eq!(vm.object_count(), count + 5 + iteration);
            assert_eq!(vm.get(vm.global(), "retainedStringToken").unwrap(), token);
            assert_eq!(
                vm.own_property_flags(vm.global(), "String").unwrap(),
                Some(1)
            );
        }
        let graph = vm.string_objects.unwrap();
        assert_eq!(
            vm.get(vm.global(), "String").unwrap(),
            Value::Object(graph.constructor)
        );
        assert!(
            vm.call(
                Value::Object(graph.constructor),
                Value::Undefined,
                &[Value::string("a")],
                &mut RejectHost
            )
            .is_err()
        );
        let mut new = push(&[Value::Number(0.0), Value::string("String")]);
        new.push(0x40);
        assert!(execute(&mut vm, &new).is_err());
        let mut member = push(&[Value::string("a"), Value::string("toUpperCase")]);
        member.push(0x4e);
        assert!(execute(&mut vm, &member).is_err());
    }

    #[test]
    fn string_native_roles_gate_mutation_unknowns_and_ancestor_fallback() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        let graph = vm.build_string_graph().unwrap();
        for id in [
            graph.constructor,
            graph.prototype,
            graph.to_upper_case,
            graph.from_char_code,
        ] {
            assert!(vm.get(id, "unmodeled").is_err());
            assert!(vm.has_key(id, &text("unmodeled")).is_err());
            assert!(vm.descriptor(id, &text("unmodeled")).is_err());
            assert!(vm.read_property(id, "unmodeled", &mut RejectHost).is_err());
            assert!(vm.set(id, "x", Value::Undefined).is_err());
            assert!(
                vm.write_property(id, "x", Value::Undefined, &mut RejectHost)
                    .is_err()
            );
            assert!(vm.own_property_flags(id, "unmodeled").is_err());
            assert!(vm.own_enumerable_names(id).is_err());
        }
        let inherited = vm.alloc(Kind::Plain, Some(graph.prototype)).unwrap();
        assert_eq!(
            vm.get(inherited, "toUpperCase").unwrap(),
            Value::Object(graph.to_upper_case)
        );
        assert!(
            vm.read_property(inherited, "unmodeled", &mut RejectHost)
                .is_err()
        );
        assert!(vm.set(inherited, "x", Value::Undefined).is_err());
        assert!(vm.enumerate_names(inherited).is_err());
        assert!(vm.delete_member(inherited, &text("x")).is_err());
        let add_property = vm.get(vm.object_prototype.unwrap(), "addProperty").unwrap();
        assert!(
            vm.call(
                add_property,
                Value::Object(inherited),
                &[
                    Value::string("x"),
                    Value::Object(graph.to_upper_case),
                    Value::Null
                ],
                &mut RejectHost
            )
            .is_err()
        );
    }

    #[test]
    fn primitive_string_call_rejects_method_tokens_without_factory_effects() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        vm.bind_builtin_context_target(vm.root()).unwrap();
        let graph = vm.build_string_graph().unwrap();
        vm.set(vm.global(), "String", Value::Object(graph.constructor))
            .unwrap();
        let token = vm
            .register_deferred_builtin(
                vm.global(),
                "UnsupportedMethod",
                BuiltinFactory::Unsupported,
                1,
            )
            .unwrap();
        vm.raw_member(
            graph.prototype,
            text("toUpperCase"),
            Property::Data(token),
            1,
        )
        .unwrap();
        let count = vm.object_count();
        let mut bytes = push(&[
            Value::Number(0.0),
            Value::string("abc"),
            Value::string("toUpperCase"),
        ]);
        bytes.push(0x52);
        let error = execute(&mut vm, &bytes).unwrap_err();
        // Unsupported graph mutation is detected before any box or alternate
        // factory is invoked. It is not reported as native mutation behavior.
        assert!(matches!(
            error.kind,
            ErrorKind::Unsupported("String canonical graph metadata")
        ));
        assert_eq!(vm.object_count(), count);
        assert_eq!(vm.host_calls, 0);
    }
    #[test]
    fn deferred_receiver_resolution_caches_without_rewriting_retained_tokens() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        let namespace = vm.object().unwrap();
        let token = vm
            .register_deferred_builtin(
                namespace,
                "ExternalInterface",
                BuiltinFactory::ExternalInterface { call_binding: 17 },
                0,
            )
            .unwrap();
        vm.set(vm.global(), "receiver", token.clone()).unwrap();
        let mut bytes = push(&[Value::Number(7.0), Value::Number(1.0)]);
        bytes.extend(variable_bytes("receiver"));
        bytes.extend(push(&[Value::string("call")]));
        bytes.push(0x52);
        assert!(matches!(
            execute(&mut vm, &bytes).unwrap_err().kind,
            ErrorKind::Unsupported(_)
        ));
        assert_eq!(vm.get(namespace, "ExternalInterface").unwrap(), token);
        vm.bind_builtin_context_target(vm.root()).unwrap();
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        let before = vm.object_count();
        let mut host = BoundHost { calls: 0 };
        assert_eq!(
            vm.execute(code, vm.root(), &mut host).unwrap().stack,
            vec![Value::Number(42.0)]
        );
        let resolved = vm.get(namespace, "ExternalInterface").unwrap();
        assert!(matches!(resolved, Value::Object(_)));
        assert_eq!(vm.get(vm.global(), "receiver").unwrap(), token);
        assert_eq!(vm.object_count(), before + 4);
        vm.execute(code, vm.root(), &mut host).unwrap();
        assert_eq!(vm.object_count(), before + 4);
        assert_eq!(host.calls, 2);
        assert!(!vm.strict_equal(&token, &resolved));
        // Direct native API calls do not invent an Environment from `this`.
        assert!(
            vm.call(token.clone(), Value::Object(vm.root()), &[], &mut host)
                .is_err()
        );
        let mut direct = push(&[Value::Number(0.0)]);
        direct.extend(variable_bytes("receiver"));
        direct.push(0x3d);
        assert_eq!(
            execute(&mut vm, &direct).unwrap().stack,
            vec![Value::Undefined]
        );
        assert_eq!(vm.object_count(), before + 4);
    }
    #[test]
    fn deferred_primitive_left_equality_materializes_before_returning_false() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        let namespace = vm.object().unwrap();
        let token = vm
            .register_deferred_builtin(
                namespace,
                "ExternalInterface",
                BuiltinFactory::ExternalInterface { call_binding: 17 },
                0,
            )
            .unwrap();
        vm.set(vm.global(), "token", token.clone()).unwrap();
        vm.bind_builtin_context_target(vm.root()).unwrap();
        let before = vm.object_count();
        let mut bytes = variable_bytes("token");
        bytes.extend(push(&[Value::Number(0.0)]));
        bytes.push(0x49);
        assert_eq!(
            execute(&mut vm, &bytes).unwrap().stack,
            vec![Value::Bool(false)]
        );
        assert_eq!(vm.object_count(), before);
        let mut bytes = push(&[Value::Number(0.0)]);
        bytes.extend(variable_bytes("token"));
        bytes.push(0x49);
        assert_eq!(
            execute(&mut vm, &bytes).unwrap().stack,
            vec![Value::Bool(false)]
        );
        assert_eq!(vm.object_count(), before + 4);
        assert!(matches!(
            vm.get(namespace, "ExternalInterface").unwrap(),
            Value::Object(_)
        ));
        assert_eq!(vm.get(vm.global(), "token").unwrap(), token);
    }
    #[test]
    fn host_context_tracks_active_target_not_receiver_or_definition_origin() {
        let mut vm = setup();
        let a = vm.root();
        let b = vm.object().unwrap();
        let receiver = vm.object().unwrap();
        let native = vm.native(10).unwrap();
        vm.set(receiver, "bound", Value::Object(native)).unwrap();
        vm.set(vm.global(), "receiver", Value::Object(receiver))
            .unwrap();
        let mut body = method_call("receiver", "bound");
        body.push(0x3e);
        let definition = vm
            .register(Arc::new(Program::parse(&function("f", &body)).unwrap()))
            .unwrap();
        vm.execute(definition, b, &mut RejectHost).unwrap();
        let f = vm.get(b, "f").unwrap();
        vm.set(a, "f", f.clone()).unwrap();
        let mut bytes = push(&[Value::Number(0.0), Value::string("f")]);
        bytes.extend([0x3d, 0x17]);
        bytes.extend(method_call("receiver", "bound"));
        let caller = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        let mut host = ContextHost::default();
        vm.execute(caller, a, &mut host).unwrap();
        assert_eq!(host.calls.len(), 2);
        assert_eq!(
            (host.calls[0].1, &host.calls[0].2),
            (Some(b), &Value::Object(receiver))
        );
        assert_eq!(host.calls[0].3.unwrap().code, definition);
        assert_eq!(
            (host.calls[1].1, &host.calls[1].2),
            (Some(a), &Value::Object(receiver))
        );
        assert_eq!(host.calls[1].3.unwrap().code, caller);
        // Failure inside a cross-target script call restores its caller context.
        host.fail_binding = Some(10);
        vm.current_target = Some(a);
        assert!(
            vm.invoke(f, Value::Object(receiver), &[], &mut host)
                .is_err()
        );
        assert_eq!(vm.current_target, Some(a));
        host.fail_binding = None;
        // A subsequent direct native call resets execution; neither receiver nor
        // a previously active movie is invented as a native Environment target.
        vm.call(
            Value::Object(native),
            Value::Object(receiver),
            &[],
            &mut host,
        )
        .unwrap();
        assert_eq!(host.calls.last().unwrap().1, None);
        assert_eq!(host.calls.last().unwrap().3, None);
    }
    #[test]
    fn new_object_zero_arguments_keeps_native_metadata_and_current_global_guard() {
        let mut vm = setup();
        let object = vm.install_object_constructor().unwrap();
        let mut bytes = push(&[Value::Number(0.0), Value::string("Object")]);
        bytes.push(0x40);
        let result = execute(&mut vm, &bytes).unwrap();
        let [Value::Object(instance)] = result.stack.as_slice() else {
            panic!()
        };
        assert_eq!(
            vm.objects[instance.0].insertion_keys,
            [
                text("__proto__"),
                text("__constructor__"),
                text("constructor")
            ]
        );
        for key in ["__proto__", "__constructor__", "constructor"] {
            assert_eq!(vm.own_property_flags(*instance, key).unwrap(), Some(3));
        }
        assert_eq!(
            vm.get(*instance, "constructor").unwrap(),
            Value::Object(object)
        );
        assert_eq!(
            vm.get(*instance, "__constructor__").unwrap(),
            Value::Object(object)
        );
        assert_eq!(
            vm.get(*instance, "__proto__").unwrap(),
            vm.get(object, "prototype").unwrap()
        );
        let mut argument = push(&[Value::Null, Value::Number(1.0), Value::string("Object")]);
        argument.push(0x40);
        assert_eq!(
            execute(&mut vm, &argument).unwrap_err().kind,
            ErrorKind::Unsupported("Object construction argument/method profile")
        );
        vm.set(vm.root(), "Object", Value::Object(object)).unwrap();
        vm.set(vm.global(), "Object", Value::Null).unwrap();
        assert!(matches!(
            execute(&mut vm, &bytes).unwrap_err().kind,
            ErrorKind::Unsupported(_)
        ));
    }
    #[test]
    fn external_interface_execution_keeps_native_boundaries_explicit() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        let graph = vm.build_external_interface(17).unwrap();
        assert!(vm.callable(&Value::Object(graph.constructor)));
        assert!(!vm.callable(&Value::Object(graph.prototype)));
        let result = vm
            .call(
                Value::Object(graph.constructor),
                Value::Null,
                &[Value::string("ignored by native callback")],
                &mut RejectHost,
            )
            .unwrap();
        assert_eq!(result.value, Value::Undefined);
        assert_eq!(result.host_calls, 0);
        assert!(matches!(
            vm.call(
                Value::Object(graph.add_callback),
                Value::Object(graph.constructor),
                &[],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::UnsupportedNativeFunction(_)
        ));
        assert_eq!(vm.host_calls, 0);
        for target in [graph.constructor, graph.prototype] {
            assert!(vm.set(target, "example", Value::Null).is_err());
            assert!(
                vm.write_property(target, "example", Value::Null, &mut RejectHost)
                    .is_err()
            );
            assert!(vm.delete_member(target, &text("constructor")).is_err());
        }
        assert_eq!(
            vm.get(graph.constructor, "available").unwrap_err().kind,
            ErrorKind::Unsupported("ExternalInterface.available active handler query")
        );
        assert!(
            vm.read_property(graph.constructor, "available", &mut RejectHost)
                .is_err()
        );
        let derived = vm.alloc(Kind::Plain, Some(graph.constructor)).unwrap();
        assert!(vm.set(derived, "available", Value::Bool(false)).is_err());
        vm.set(vm.global(), "EI", Value::Object(graph.constructor))
            .unwrap();
        let mut bytes = push(&[Value::Number(0.0), Value::string("EI")]);
        bytes.push(0x40);
        let before = vm.object_count();
        assert_eq!(
            execute(&mut vm, &bytes).unwrap_err().kind,
            ErrorKind::Unsupported("ExternalInterface construction profile")
        );
        assert_eq!(vm.object_count(), before);
        // The installed method remains an explicit host binding. No handler is
        // installed by building the fresh graph, and default hosts reject it.
        assert!(matches!(
            vm.call(
                Value::Object(graph.call),
                Value::Object(graph.constructor),
                &[],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::Host(_)
        ));
        let mut host = BoundHost { calls: 0 };
        assert_eq!(
            vm.call(
                Value::Object(graph.call),
                Value::Object(graph.constructor),
                &[Value::Number(7.0)],
                &mut host
            )
            .unwrap()
            .value,
            Value::Number(42.0)
        );
        assert_eq!(host.calls, 1);
    }
    #[test]
    fn host_context_survives_object_result_setter_reentry_and_error() {
        let mut vm = setup();
        let object = vm.install_object_constructor().unwrap();
        let Value::Object(prototype) = vm.get(object, "prototype").unwrap() else {
            panic!()
        };
        let a = vm.root();
        let b = vm.object().unwrap();
        let receiver = vm.object().unwrap();
        for (name, binding) in [("materialize", 10), ("setter", 20), ("after", 30)] {
            let native = vm.native(binding).unwrap();
            vm.set(receiver, name, Value::Object(native)).unwrap();
        }
        vm.set(vm.global(), "receiver", Value::Object(receiver))
            .unwrap();
        let mut body = method_call("receiver", "setter");
        body.push(0x3e);
        let definition = vm
            .register(Arc::new(
                Program::parse(&function("setter", &body)).unwrap(),
            ))
            .unwrap();
        vm.execute(definition, b, &mut RejectHost).unwrap();
        let setter = vm.get(b, "setter").unwrap();
        vm.raw_member(
            prototype,
            text("field"),
            Property::Accessor {
                getter: setter.clone(),
                setter: Some(setter),
            },
            0,
        )
        .unwrap();
        let mut bytes = method_call("receiver", "materialize");
        bytes.push(0x17);
        bytes.extend(method_call("receiver", "after"));
        let caller = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        let mut host = ContextHost {
            fields_binding: Some(10),
            ..ContextHost::default()
        };
        vm.execute(caller, a, &mut host).unwrap();
        assert_eq!(
            host.calls
                .iter()
                .map(|call| (call.0, call.1))
                .collect::<Vec<_>>(),
            vec![(10, Some(a)), (20, Some(b)), (30, Some(a))]
        );
        host.calls.clear();
        host.fail_binding = Some(20);
        assert!(vm.execute(caller, a, &mut host).is_err());
        assert_eq!(
            host.calls
                .iter()
                .map(|call| (call.0, call.1))
                .collect::<Vec<_>>(),
            vec![(10, Some(a)), (20, Some(b))]
        );
        assert_eq!(vm.current_target, Some(a));
    }
    impl Host for ObjectResultHost {
        fn call(&mut self, call: HostCall<'_>) -> std::result::Result<Value, String> {
            self.calls
                .push((call.binding, call.this, call.args.to_vec()));
            if matches!(call.binding, 20 | 21) {
                Ok(Value::Bool(false))
            } else {
                Err("unsupported test binding".into())
            }
        }
        fn call_result(&mut self, call: HostCall<'_>) -> std::result::Result<HostResult, String> {
            if call.binding == 10 {
                self.calls
                    .push((call.binding, call.this, call.args.to_vec()));
                Ok(HostResult::ObjectFields(self.fields.clone()))
            } else {
                self.call(call).map(HostResult::Value)
            }
        }
        fn command(&mut self, _: ObjectId, _: HostCommand) -> std::result::Result<(), String> {
            Err("unsupported test command".into())
        }
    }
    #[test]
    fn host_object_fields_are_fresh_ordered_and_keep_native_metadata() {
        let mut vm = setup();
        let object = vm.install_object_constructor().unwrap();
        let native = vm.native(10).unwrap();
        let fields = vec![
            (text("localizedPrototypeIdStr"), Value::string("123")),
            (text("localizedText"), Value::string("Hello 🦀")),
            (text("localizedTooltipText"), Value::string("")),
            (text("hasTooltip"), Value::Bool(false)),
        ];
        let mut host = ObjectResultHost {
            fields: fields.clone(),
            calls: vec![],
        };
        let mut ids = vec![];
        for _ in 0..2 {
            let result = vm
                .call(
                    Value::Object(native),
                    Value::Object(vm.root()),
                    &[],
                    &mut host,
                )
                .unwrap();
            assert_eq!(result.host_calls, 1);
            let Value::Object(id) = result.value else {
                panic!()
            };
            ids.push(id);
            for name in ["__proto__", "__constructor__", "constructor"] {
                assert_eq!(vm.own_property_flags(id, name).unwrap(), Some(3));
            }
            assert_eq!(
                vm.get(id, "__proto__").unwrap(),
                vm.get(object, "prototype").unwrap()
            );
            assert_eq!(
                vm.get(id, "__constructor__").unwrap(),
                Value::Object(object)
            );
            assert_eq!(vm.get(id, "constructor").unwrap(), Value::Object(object));
            assert_eq!(
                &vm.objects[id.0].insertion_keys[3..],
                fields
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect::<Vec<_>>()
            );
            for (name, value) in &fields {
                assert_eq!(vm.get_key(id, name).unwrap(), *value);
                assert_eq!(vm.own_flags(id, name), 0);
            }
        }
        assert_ne!(ids[0], ids[1]);
        vm.set(ids[0], "localizedText", Value::string("Changed"))
            .unwrap();
        assert_eq!(
            vm.get(ids[1], "localizedText").unwrap(),
            Value::string("Hello 🦀")
        );
    }
    #[test]
    fn host_object_fields_use_ordered_setters_with_host_reentry() {
        let mut vm = setup();
        let object = vm.install_object_constructor().unwrap();
        let Value::Object(prototype) = vm.get(object, "prototype").unwrap() else {
            panic!()
        };
        for (name, binding) in [("first", 20), ("second", 21)] {
            let setter = vm.native(binding).unwrap();
            vm.raw_member(
                prototype,
                text(name),
                Property::Accessor {
                    getter: Value::Object(setter),
                    setter: Some(Value::Object(setter)),
                },
                0,
            )
            .unwrap();
        }
        let native = vm.native(10).unwrap();
        let mut host = ObjectResultHost {
            fields: vec![
                (text("first"), Value::string("A")),
                (text("second"), Value::Bool(true)),
            ],
            calls: vec![],
        };
        let result = vm
            .call(
                Value::Object(native),
                Value::Object(vm.root()),
                &[],
                &mut host,
            )
            .unwrap();
        let Value::Object(id) = result.value else {
            panic!()
        };
        assert_eq!(result.host_calls, 3);
        assert_eq!(
            host.calls[1],
            (20, Value::Object(id), vec![Value::string("A")])
        );
        assert_eq!(
            host.calls[2],
            (21, Value::Object(id), vec![Value::Bool(true)])
        );
        assert_eq!(vm.own_property_flags(id, "first").unwrap(), None);
        assert_eq!(vm.own_property_flags(id, "second").unwrap(), None);
        // False native setter returns do not discard the fresh result object.
        assert_eq!(vm.get(id, "constructor").unwrap(), Value::Object(object));
        vm.limits.host_calls = 1;
        host.calls.clear();
        let error = vm
            .call(
                Value::Object(native),
                Value::Object(vm.root()),
                &[],
                &mut host,
            )
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Budget("host calls"));
        assert_eq!(host.calls.len(), 1);
    }
    #[test]
    fn host_object_fields_reject_invalid_payloads_and_modified_bindings() {
        let mut vm = setup();
        vm.install_object_constructor().unwrap();
        let native = vm.native(10).unwrap();
        let mut host = ObjectResultHost {
            fields: vec![],
            calls: vec![],
        };
        for fields in [
            vec![(text("x"), Value::Number(1.0))],
            vec![(text("x"), Value::String(Arc::from([0xd800])))],
            vec![(text("x"), Value::string("embedded\0nul"))],
            vec![(text("constructor"), Value::Bool(true))],
            vec![(text("a.b"), Value::Bool(true))],
            vec![
                (text("x"), Value::Bool(true)),
                (text("x"), Value::Bool(false)),
            ],
        ] {
            host.fields = fields;
            let before = vm.object_count();
            assert!(matches!(
                vm.call(
                    Value::Object(native),
                    Value::Object(vm.root()),
                    &[],
                    &mut host
                )
                .unwrap_err()
                .kind,
                ErrorKind::Unsupported(_)
            ));
            assert_eq!(vm.object_count(), before);
        }
        host.fields = vec![(text("value"), Value::string("payload"))];
        let before = vm.object_count();
        vm.limits.string_units = vm.string_units + 2;
        assert_eq!(
            vm.call(
                Value::Object(native),
                Value::Object(vm.root()),
                &[],
                &mut host
            )
            .unwrap_err()
            .kind,
            ErrorKind::Budget("host result object string units")
        );
        assert_eq!(vm.object_count(), before);
        vm.limits.string_units = Limits::default().string_units;
        vm.limits.properties = vm.properties + 3;
        assert_eq!(
            vm.call(
                Value::Object(native),
                Value::Object(vm.root()),
                &[],
                &mut host
            )
            .unwrap_err()
            .kind,
            ErrorKind::Budget("host result object fields")
        );
        assert_eq!(vm.object_count(), before);
        vm.limits.properties = Limits::default().properties;
        let original = vm.get(vm.global(), "Object").unwrap();
        let inherited = vm.object().unwrap();
        vm.set(inherited, "Object", original).unwrap();
        vm.set(vm.global(), "__proto__", Value::Object(inherited))
            .unwrap();
        vm.objects[vm.global.0].properties.remove(&text("Object"));
        let before = vm.object_count();
        assert_eq!(
            vm.call(
                Value::Object(native),
                Value::Object(vm.root()),
                &[],
                &mut host
            )
            .unwrap_err()
            .kind,
            ErrorKind::Unsupported("InitObject modified Object binding/prototype")
        );
        assert_eq!(vm.object_count(), before);
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
            payload.extend([0, 0, 0, 3, 0x19, 0]);
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
        payload.extend([0, 0, 3, 0x19, 0]);
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
    fn qualified_get_variable_preserves_modeled_scope_and_function_identity() {
        let mut vm = setup();
        execute(&mut vm, &function("class", &[0])).unwrap();
        let class = vm.get(vm.root(), "class").unwrap();
        let global_namespace = vm.object().unwrap();
        let global_controls = vm.object().unwrap();
        vm.set(vm.global(), "gfx", Value::Object(global_namespace))
            .unwrap();
        vm.set(global_namespace, "controls", Value::Object(global_controls))
            .unwrap();
        vm.set(global_controls, "Widget", class.clone()).unwrap();
        let mut lookup = push(&[Value::string("gfx.controls.Widget")]);
        lookup.push(0x1c);
        let result = execute(&mut vm, &lookup).unwrap();
        assert_eq!(result.stack, [class.clone()]);
        assert_eq!(result.host_calls, 0);
        // A target binding wins, including a missing inner member: no global restart.
        let target_namespace = vm.object().unwrap();
        let target_controls = vm.object().unwrap();
        vm.set(vm.root(), "gfx", Value::Object(target_namespace))
            .unwrap();
        vm.set(target_namespace, "controls", Value::Object(target_controls))
            .unwrap();
        assert!(matches!(
            execute(&mut vm, &lookup).unwrap_err().kind,
            ErrorKind::Unsupported("qualified variable own data member profile")
        ));
        vm.set(target_controls, "Widget", class.clone()).unwrap();
        assert_eq!(execute(&mut vm, &lookup).unwrap().stack, [class]);
        vm.set(vm.root(), "gfx", Value::Undefined).unwrap();
        assert!(matches!(
            execute(&mut vm, &lookup).unwrap_err().kind,
            ErrorKind::Unsupported("qualified variable primitive/missing segment")
        ));
    }
    #[test]
    fn qualified_get_variable_rejects_unproved_routes_without_getters() {
        let mut vm = setup();
        let getter = vm.native(99).unwrap();
        vm.raw_member(
            vm.global(),
            text("gfx"),
            Property::Accessor {
                getter: Value::Object(getter),
                setter: None,
            },
            0,
        )
        .unwrap();
        let mut lookup = push(&[Value::string("gfx.controls.Widget")]);
        lookup.push(0x1c);
        let error = execute(&mut vm, &lookup).unwrap_err();
        assert_eq!(
            error.kind,
            ErrorKind::Unsupported("qualified variable first-segment accessor")
        );
        assert_eq!(error.opcode, Some(0x1c));
        assert_eq!(vm.host_calls, 0);
        for path in [
            "gfx..Widget",
            "gfx/Widget",
            "gfx:Widget",
            "_global.gfx.Widget",
            "gfx.É.Widget",
            "this.Widget",
        ] {
            let mut code = push(&[Value::string(path)]);
            code.push(0x1c);
            assert!(matches!(
                execute(&mut vm, &code).unwrap_err().kind,
                ErrorKind::Unsupported(_)
            ));
        }
        let (store, registers) = vm.isolated_store(None).unwrap();
        let frame = Frame {
            store,
            registers,
            shared_calls: None,
            scopes: vec![vm.root(), vm.global()],
            pool: Arc::new(vec![]),
            target: vm.root(),
            this: Value::Object(vm.root()),
            this_visible: true,
            locals: vm.object().unwrap(),
            kind: FrameKind::Ordinary,
        };
        assert!(matches!(
            vm.get_variable(&frame, &text("gfx.controls.Widget"), &mut RejectHost)
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported("qualified variable activation/With scope")
        ));
        // Qualified writes and CallFunction paths remain outside this read-only subset.
        assert!(vm.simple_name(&text("gfx.controls.Widget")).is_err());
        let parent = vm.object().unwrap();
        vm.set(vm.root(), "__proto__", Value::Object(parent))
            .unwrap();
        assert!(matches!(
            execute(&mut vm, &lookup).unwrap_err().kind,
            ErrorKind::Unsupported("qualified variable modeled root/global profile")
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
        let (store, registers) = vm.isolated_store(None).unwrap();
        let frame = Frame {
            store,
            registers,
            shared_calls: None,
            scopes: vec![inner, vm.global()],
            pool: Arc::new(Vec::new()),
            target: vm.root(),
            this: Value::Object(inner),
            this_visible: true,
            locals: inner,
            kind: FrameKind::Ordinary,
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
    fn attached_apply_program(method: &str, list: Value) -> Vec<u8> {
        let mut bytes = push(&[list]);
        bytes.extend(variable_bytes("_global"));
        bytes.extend(push(&[Value::string("fixtureReceiver")]));
        bytes.push(0x4e);
        bytes.extend(push(&[Value::Number(2.0)]));
        bytes.extend(variable_bytes("_global"));
        bytes.extend(push(&[Value::string("fixtureFunction")]));
        bytes.push(0x4e);
        bytes.extend(push(&[Value::string(method)]));
        bytes.push(0x52);
        bytes
    }
    fn anonymous_function2(flags: u16, body: &[u8]) -> Vec<u8> {
        // The super fixtures preload indices1 and2; native allocation needs
        // N=3 for both to be inside this function's isolated register range.
        let mut payload = vec![0, 0, 0, 3];
        payload.extend(flags.to_le_bytes());
        payload.extend((body.len() as u16).to_le_bytes());
        let mut bytes = action(0x8e, &payload);
        bytes.extend(body);
        bytes
    }
    #[test]
    fn function2_missing_register_argument_preserves_prior_parameter_and_preloads_win() {
        let mut vm = setup();
        let mut body = action(0x96, &[4, 1]);
        body.push(0x3e);
        for flags in [0x2au16, 0x29] {
            let mut payload = vec![0, 2, 0, 2];
            payload.extend(flags.to_le_bytes());
            payload.extend(b"\x01first\0\x01second\0");
            payload.extend((body.len() as u16).to_le_bytes());
            let mut bytes = action(0x8e, &payload);
            bytes.extend(&body);
            let code = vm
                .register(Arc::new(Program::parse(&bytes).unwrap()))
                .unwrap();
            let function = vm
                .execute(code, vm.root(), &mut RejectHost)
                .unwrap()
                .stack
                .pop()
                .unwrap();
            assert_eq!(
                vm.classify_callability(&function).unwrap(),
                ValueCallability::Callable
            );
            let receiver = Value::Object(vm.root());
            for args in [
                vec![],
                vec![Value::Number(7.0)],
                vec![Value::Number(7.0), Value::Number(9.0)],
            ] {
                let expected = if flags & 1 != 0 {
                    receiver.clone()
                } else {
                    args.last().cloned().unwrap_or(Value::Undefined)
                };
                assert_eq!(
                    vm.call(function.clone(), receiver.clone(), &args, &mut RejectHost)
                        .unwrap()
                        .value,
                    expected
                );
            }
        }
    }
    #[test]
    fn function2_register_count_does_not_grant_an_extra_or_outer_slot() {
        fn invoke_count(n: u8, flags: u16, body: &[u8]) -> Result<Execution> {
            let mut vm = setup();
            let mut definition = anonymous_function2(flags, body);
            definition[6] = n; // Serialized register_count byte in this fixture.
            let code = vm
                .register(Arc::new(Program::parse(&definition).unwrap()))
                .unwrap();
            let function = vm
                .execute(code, vm.root(), &mut RejectHost)
                .unwrap()
                .stack
                .pop()
                .unwrap();
            vm.call(function, Value::Object(vm.root()), &[], &mut RejectHost)
        }
        let mut body = action(0x96, &[4, 1]);
        body.push(0x3e);
        assert!(matches!(
            invoke_count(2, 0x29, &body).unwrap().value,
            Value::Object(_)
        ));
        assert_eq!(
            invoke_count(1, 0x29, &body).unwrap_err().kind,
            ErrorKind::InvalidRegister(1)
        );
        let mut outside = action(0x96, &[4, 2]);
        outside.push(0x3e);
        assert_eq!(
            invoke_count(2, 0x29, &outside).unwrap_err().kind,
            ErrorKind::InvalidRegister(2)
        );
        let mut store = push(&[Value::Number(9.0)]);
        store.extend(action(0x87, &[2]));
        assert_eq!(
            invoke_count(2, 0x29, &store).unwrap_err().kind,
            ErrorKind::InvalidRegister(2)
        );
        let mut no_registers = push(&[Value::Number(7.0)]);
        no_registers.push(0x3e);
        assert_eq!(
            invoke_count(0, 0x2a, &no_registers).unwrap().value,
            Value::Number(7.0)
        );
        assert_eq!(
            invoke_count(0, 0x2a, &action(0x96, &[4, 0]))
                .unwrap_err()
                .kind,
            ErrorKind::InvalidRegister(0)
        );
    }
    #[test]
    fn attached_apply_preserves_display_this_and_captured_environment() {
        for flags in [0x28, 0x29] {
            let (mut vm, root, caller, _) = typed_field_fixture();
            let root_owner = vm.attached_owner(root).unwrap();
            let caller_owner = vm.attached_owner(caller).unwrap();
            let receiver = vm.attached_value(caller).unwrap();
            let observe = vm.native(97).unwrap();
            vm.set(vm.global(), "observe", Value::Object(observe))
                .unwrap();
            let mut body = method_call("_global", "observe");
            body.push(0x17);
            body.extend(if flags & 1 != 0 {
                action(0x96, &[4, 1])
            } else {
                variable_bytes("this")
            });
            body.push(0x3e);
            let definition = anonymous_function2(flags, &body);
            let code = vm
                .register(Arc::new(Program::parse(&definition).unwrap()))
                .unwrap();
            let callable = vm
                .execute(code, root_owner, &mut RejectHost)
                .unwrap()
                .stack
                .pop()
                .unwrap();
            vm.set(vm.global(), "fixtureFunction", callable).unwrap();
            vm.set(vm.global(), "fixtureReceiver", receiver.clone())
                .unwrap();
            for list in [Value::Null, Value::Undefined] {
                let mut bytes = attached_apply_program("apply", list);
                // Observe caller context after the nested script returns.
                bytes.extend(method_call("_global", "observe"));
                bytes.push(0x17);
                let code = vm
                    .register(Arc::new(Program::parse_at(&bytes, 400).unwrap()))
                    .unwrap();
                let mut host = ContextHost::default();
                let result = vm.execute(code, caller_owner, &mut host).unwrap();
                assert_eq!(result.stack, vec![receiver.clone()]);
                assert_eq!(host.calls.len(), 2);
                assert_eq!(host.calls[0].1, Some(root_owner));
                assert_eq!(host.calls[1].1, Some(caller_owner));
                assert_eq!(vm.current_target, Some(caller_owner));
                assert!(vm.attached_info(root).unwrap().storage.is_none());
                assert!(vm.attached_info(caller).unwrap().storage.is_none());
            }
        }
    }
    #[test]
    fn attached_applied_super_advances_prototype_without_constructing_receiver() {
        #[derive(Default)]
        struct SuperHost(Vec<(u32, Vec<Value>, Option<ObjectId>)>);
        impl Host for SuperHost {
            fn call(&mut self, call: HostCall<'_>) -> std::result::Result<Value, String> {
                self.0
                    .push((call.binding, call.args.to_vec(), call.context));
                Ok(Value::Undefined)
            }
            fn command(&mut self, _: ObjectId, _: HostCommand) -> std::result::Result<(), String> {
                Err("unexpected command".into())
            }
        }
        let (mut vm, root, receiver, _) = typed_field_fixture();
        let root_owner = vm.attached_owner(root).unwrap();
        let receiver_value = vm.attached_value(receiver).unwrap();
        for (name, audit, binding) in [("Base", "auditBase", 701), ("Derived", "auditDerived", 702)]
        {
            let callback = vm.native(binding).unwrap();
            vm.set(vm.global(), audit, Value::Object(callback)).unwrap();
            let mut body = push(&[Value::Number(0.0)]);
            body.extend(action(0x96, &[4, 2, 3])); // r2, Undefined: unnamed super().
            body.push(0x52);
            // Preserve the returned superclass result as the second audit argument.
            body.extend(action(0x96, &[4, 1]));
            body.extend(push(&[Value::Number(2.0)]));
            body.extend(variable_bytes("_global"));
            body.extend(push(&[Value::string(audit)]));
            body.extend([0x52, 0x17]);
            body.extend(action(0x96, &[4, 1]));
            body.push(0x3e);
            let definition = anonymous_function2(0x19, &body);
            let code = vm
                .register(Arc::new(Program::parse(&definition).unwrap()))
                .unwrap();
            let class = vm
                .execute(code, root_owner, &mut RejectHost)
                .unwrap()
                .stack
                .pop()
                .unwrap();
            vm.set(vm.global(), name, class).unwrap();
        }
        for (child, parent) in [("Base", "MovieClip"), ("Derived", "Base")] {
            let mut bytes = vec![];
            for name in [child, parent] {
                bytes.extend(variable_bytes("_global"));
                bytes.extend(push(&[Value::string(name)]));
                bytes.push(0x4e);
            }
            bytes.push(0x69);
            let code = vm
                .register(Arc::new(Program::parse(&bytes).unwrap()))
                .unwrap();
            vm.execute(code, root_owner, &mut RejectHost).unwrap();
        }
        let Value::Object(derived) = vm.get(vm.global(), "Derived").unwrap() else {
            panic!()
        };
        let Value::Object(prototype) = vm.get(derived, "prototype").unwrap() else {
            panic!()
        };
        vm.set_attached_prototype(receiver, Some(prototype))
            .unwrap();
        let storage = vm.attached_info(receiver).unwrap().storage;
        vm.set(vm.global(), "fixtureReceiver", receiver_value.clone())
            .unwrap();
        vm.set(vm.global(), "fixtureFunction", Value::Object(derived))
            .unwrap();
        let bytes = attached_apply_program("apply", Value::Null);
        let code = vm
            .register(Arc::new(Program::parse_at(&bytes, 600).unwrap()))
            .unwrap();
        let attached_count = vm.attached.len();
        let wrappers_before = vm
            .objects
            .iter()
            .filter(|o| matches!(o.kind, Kind::AttachedSuper { .. }))
            .count();
        let mut host = SuperHost::default();
        let result = vm.execute(code, root_owner, &mut host).unwrap();
        assert_eq!(result.stack, vec![receiver_value.clone()]);
        assert_eq!(
            host.0,
            vec![
                (
                    701,
                    vec![receiver_value.clone(), Value::Undefined],
                    Some(root_owner)
                ),
                (
                    702,
                    vec![receiver_value.clone(), receiver_value],
                    Some(root_owner)
                ),
            ]
        );
        assert_eq!(vm.attached.len(), attached_count);
        assert_eq!(
            vm.attached_info(receiver).unwrap().prototype,
            Some(prototype)
        );
        assert_eq!(vm.attached_info(receiver).unwrap().storage, storage);
        assert_eq!(
            vm.objects
                .iter()
                .filter(|o| matches!(o.kind, Kind::AttachedSuper { .. }))
                .count(),
            wrappers_before + 2
        );
        assert_eq!(vm.current_target, Some(root_owner));
    }
    #[test]
    fn attached_apply_gates_unproved_profiles_and_keeps_super_separate() {
        let (mut vm, root, caller, field) = typed_field_fixture();
        let root_owner = vm.attached_owner(root).unwrap();
        let receiver = vm.attached_value(caller).unwrap();
        let definition = anonymous_function2(0x29, &[0]);
        let code = vm
            .register(Arc::new(Program::parse(&definition).unwrap()))
            .unwrap();
        let callable = vm
            .execute(code, root_owner, &mut RejectHost)
            .unwrap()
            .stack
            .pop()
            .unwrap();
        vm.set(vm.global(), "fixtureFunction", callable.clone())
            .unwrap();
        vm.set(vm.global(), "fixtureReceiver", receiver.clone())
            .unwrap();
        let Value::Object(function_id) = callable.clone() else {
            panic!()
        };
        let apply = vm.get(function_id, "apply").unwrap();
        let before = vm.object_count();
        assert!(
            vm.call(
                apply,
                callable,
                &[receiver.clone(), Value::Null],
                &mut RejectHost
            )
            .is_err()
        );
        assert_eq!(vm.object_count(), before);
        for (method, list) in [("call", Value::Null), ("apply", Value::Number(0.0))] {
            let bytes = attached_apply_program(method, list);
            let code = vm
                .register(Arc::new(Program::parse(&bytes).unwrap()))
                .unwrap();
            assert!(vm.execute(code, root_owner, &mut RejectHost).is_err());
            assert_eq!(vm.object_count(), before);
        }
        let bytes = attached_apply_program("apply", Value::Null);
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        vm.set(
            vm.global(),
            "fixtureReceiver",
            vm.attached_text_field_value(field).unwrap(),
        )
        .unwrap();
        assert!(vm.execute(code, root_owner, &mut RejectHost).is_err());
        assert_eq!(vm.object_count(), before);
        vm.set(vm.global(), "fixtureReceiver", receiver).unwrap();
        let plain = vm.object().unwrap();
        assert!(vm.execute(code, plain, &mut RejectHost).is_err());
        let old = function("", &[0]);
        let old_code = vm
            .register(Arc::new(Program::parse(&old).unwrap()))
            .unwrap();
        let old_function = vm
            .execute(old_code, root_owner, &mut RejectHost)
            .unwrap()
            .stack
            .pop()
            .unwrap();
        vm.set(vm.global(), "fixtureFunction", old_function)
            .unwrap();
        assert!(vm.execute(code, root_owner, &mut RejectHost).is_err());
        let mut closure_body = anonymous_function2(0x29, &[0]);
        closure_body.push(0x3e);
        let closure_definition = anonymous_function2(0x29, &closure_body);
        let closure_code = vm
            .register(Arc::new(Program::parse(&closure_definition).unwrap()))
            .unwrap();
        let outer = vm
            .execute(closure_code, root_owner, &mut RejectHost)
            .unwrap()
            .stack
            .pop()
            .unwrap();
        let closure = vm
            .call(
                outer,
                vm.attached_value(root).unwrap(),
                &[],
                &mut RejectHost,
            )
            .unwrap()
            .value;
        vm.set(vm.global(), "fixtureFunction", closure).unwrap();
        assert_eq!(
            vm.execute(code, root_owner, &mut RejectHost)
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported("attached Function.apply captured activation/With profile")
        );
        let super_definition = anonymous_function2(0x19, &[0]);
        let super_code = vm
            .register(Arc::new(Program::parse(&super_definition).unwrap()))
            .unwrap();
        let super_function = vm
            .execute(super_code, root_owner, &mut RejectHost)
            .unwrap()
            .stack
            .pop()
            .unwrap();
        vm.set(vm.global(), "fixtureFunction", super_function)
            .unwrap();
        let error = vm.execute(code, root_owner, &mut RejectHost).unwrap_err();
        assert!(matches!(error.kind, ErrorKind::Unsupported(_)));
        assert_eq!(error.origin.unwrap().code, code);
        assert_eq!(vm.host_calls, 0);
        assert!(vm.attached_info(caller).unwrap().storage.is_none());
    }
    #[test]
    fn function_conversion_preserves_top_level_identity_and_gates_captured_frames() {
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
        assert_eq!(
            vm.call(
                Value::Object(conversion),
                Value::Object(root),
                &[f.clone()],
                &mut RejectHost
            )
            .unwrap()
            .value,
            f
        );
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
    fn attached_flags_program(name: &str) -> Vec<u8> {
        let mut bytes = push(&[Value::Number(1.0), Value::string(name)]);
        bytes.extend(variable_bytes("_global"));
        bytes.extend(push(&[Value::string("fixtureFlagsTarget")]));
        bytes.push(0x4e);
        bytes.extend(push(&[Value::Number(3.0)]));
        bytes.extend(variable_bytes("_global"));
        bytes.extend(push(&[Value::string("ASSetPropFlags")]));
        bytes.push(0x52);
        bytes
    }
    #[test]
    fn attached_property_flags_change_only_existing_own_storage_flags() {
        let (mut vm, root, _, _) = typed_field_fixture();
        vm.install_property_flags().unwrap();
        let owner = vm.attached_owner(root).unwrap();
        let prototype = vm.object().unwrap();
        let getter = vm.native(99).unwrap();
        vm.raw_member(
            prototype,
            text("dispatchQueue"),
            Property::Accessor {
                getter: Value::Object(getter),
                setter: None,
            },
            3,
        )
        .unwrap();
        vm.set_attached_prototype(root, Some(prototype)).unwrap();
        let storage = ObjectId(vm.attached_info(root).unwrap().storage.unwrap().0);
        vm.raw_member(
            storage,
            text("dispatchQueue"),
            Property::Data(Value::Undefined),
            4,
        )
        .unwrap();
        vm.set(
            vm.global(),
            "fixtureFlagsTarget",
            vm.attached_value(root).unwrap(),
        )
        .unwrap();
        let bytes = attached_flags_program("dispatchQueue");
        let code = vm
            .register(Arc::new(Program::parse_at(&bytes, 800).unwrap()))
            .unwrap();
        let before_objects = vm.object_count();
        let before_flags = vm.attached_event_state(root).unwrap();
        let result = vm.execute(code, owner, &mut RejectHost).unwrap();
        assert_eq!(result.stack, vec![Value::Undefined]);
        assert_eq!(result.host_calls, 0);
        assert_eq!(vm.own_flags(storage, &text("dispatchQueue")), 5);
        assert!(matches!(
            vm.objects[storage.0].properties.get(&text("dispatchQueue")),
            Some(Property::Data(Value::Undefined))
        ));
        assert_eq!(vm.own_flags(prototype, &text("dispatchQueue")), 3);
        assert!(matches!(
            vm.objects[prototype.0]
                .properties
                .get(&text("dispatchQueue")),
            Some(Property::Accessor { .. })
        ));
        assert_eq!(vm.object_count(), before_objects);
        assert_eq!(
            vm.attached_info(root).unwrap().storage.unwrap().0,
            storage.0
        );
        assert_eq!(vm.attached_event_state(root).unwrap(), before_flags);
    }
    #[test]
    fn attached_property_flags_gate_missing_storage_inherited_and_accessor_profiles() {
        let (mut vm, root, caller, field) = typed_field_fixture();
        let intrinsic = vm.install_property_flags().unwrap();
        let owner = vm.attached_owner(root).unwrap();
        vm.set(
            vm.global(),
            "fixtureFlagsTarget",
            vm.attached_value(caller).unwrap(),
        )
        .unwrap();
        let bytes = attached_flags_program("dispatchQueue");
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        let before_objects = vm.object_count();
        assert!(vm.execute(code, owner, &mut RejectHost).is_err());
        assert!(vm.attached_info(caller).unwrap().storage.is_none());
        assert_eq!(vm.object_count(), before_objects);
        let prototype = vm.object().unwrap();
        vm.raw_member(
            prototype,
            text("dispatchQueue"),
            Property::Data(Value::Number(8.0)),
            4,
        )
        .unwrap();
        vm.set_attached_prototype(caller, Some(prototype)).unwrap();
        let storage = ObjectId(vm.attached_info(caller).unwrap().storage.unwrap().0);
        assert!(vm.execute(code, owner, &mut RejectHost).is_err());
        assert_eq!(vm.own_flags(prototype, &text("dispatchQueue")), 4);
        vm.raw_member(
            storage,
            text("dispatchQueue"),
            Property::Accessor {
                getter: Value::Object(intrinsic),
                setter: None,
            },
            0,
        )
        .unwrap();
        assert!(vm.execute(code, owner, &mut RejectHost).is_err());
        assert_eq!(vm.own_flags(storage, &text("dispatchQueue")), 0);
        vm.raw_member(
            storage,
            text("dispatchQueue"),
            Property::Data(Value::Bool(true)),
            0,
        )
        .unwrap();
        for name in [
            "dispatchQueue,other",
            "",
            "dispatch.Queue",
            "__proto__",
            "__constructor__",
            "constructor",
        ] {
            let bytes = attached_flags_program(name);
            let code = vm
                .register(Arc::new(Program::parse(&bytes).unwrap()))
                .unwrap();
            assert!(vm.execute(code, owner, &mut RejectHost).is_err());
            assert_eq!(vm.own_flags(storage, &text("dispatchQueue")), 0);
        }
        assert!(
            vm.call(
                Value::Object(intrinsic),
                Value::Object(vm.global()),
                &[
                    vm.attached_value(caller).unwrap(),
                    Value::string("dispatchQueue"),
                    Value::Number(1.0),
                ],
                &mut RejectHost
            )
            .is_err()
        );
        vm.set(
            vm.global(),
            "fixtureFlagsTarget",
            vm.attached_text_field_value(field).unwrap(),
        )
        .unwrap();
        assert!(vm.execute(code, owner, &mut RejectHost).is_err());
        assert_eq!(vm.host_calls, 0);
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
