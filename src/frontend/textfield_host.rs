//! Explicit attached TextField state, with checked scene/VM associations.
//! Native attachment, prototype discovery and effective styles are caller inputs.

use super::{display_state::DisplayState, textfield_state as state};
use crate::{
    avm1::vm::{
        AttachedClipId, AttachedTextFieldId, AttachedTextFieldInfo, CodeOrigin, ObjectId,
        TextFieldMember, TextFieldMemberRequest, TextFieldWatchState, Value, Vm,
    },
    gfx::{CharacterKind, scene::NodeId},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_fields: usize,
    pub max_contexts: usize,
    pub max_parent_depth: usize,
    pub max_calls: usize,
    /// Accounted last-input plus document bytes, not an RSS/allocator bound.
    pub max_retained_text_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_fields: 4096,
            max_contexts: 4096,
            max_parent_depth: 256,
            max_calls: 100_000,
            max_retained_text_bytes: 16 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    /// Admitted attempts, including later failures. Budget rejection is excluded.
    pub calls: usize,
    pub reads: usize,
    pub writes: usize,
    pub replacements: usize,
    pub retained_text_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    ReadString { utf16_units: usize },
    ReadBool(bool),
    SetHtml(state::HtmlFlagChange),
    Replace(state::Replacement),
}
#[derive(Clone, Debug)]
pub struct Trace {
    pub origin: Option<CodeOrigin>,
    pub source_offset: Option<usize>,
    pub field: AttachedTextFieldId,
    pub node: NodeId,
    pub context: ObjectId,
    pub environment_owner: ObjectId,
    pub member: TextFieldMember,
    pub operation: Operation,
}
struct Binding {
    info: AttachedTextFieldInfo,
    environment_owner: ObjectId,
    state: state::TextFieldState,
}

pub struct TextFieldHost {
    root: ObjectId,
    domain: NonZeroU64,
    contexts: BTreeSet<ObjectId>,
    fields: BTreeMap<AttachedTextFieldId, Binding>,
    nodes: BTreeMap<NodeId, AttachedTextFieldId>,
    owners: BTreeMap<ObjectId, AttachedTextFieldId>,
    traces: Vec<Trace>,
    limits: Limits,
    stats: Statistics,
}
impl TextFieldHost {
    /// `domain` belongs to this exact VM/Scene lifetime. The caller must create
    /// a new domain and host if either is replaced; raw numeric IDs are local.
    pub fn new(root: ObjectId, domain: NonZeroU64, limits: Limits) -> Result<Self, String> {
        if limits.max_fields == 0
            || limits.max_fields > 100_000
            || limits.max_contexts == 0
            || limits.max_contexts > 100_000
            || limits.max_parent_depth == 0
            || limits.max_parent_depth > 4096
            || limits.max_calls == 0
            || limits.max_calls > 1_000_000
        {
            return Err("invalid TextField host budgets".into());
        }
        Ok(Self {
            root,
            domain,
            contexts: BTreeSet::from([root]),
            fields: BTreeMap::new(),
            nodes: BTreeMap::new(),
            owners: BTreeMap::new(),
            traces: Vec::new(),
            limits,
            stats: Statistics::default(),
        })
    }
    pub fn root(&self) -> ObjectId {
        self.root
    }
    pub fn domain(&self) -> NonZeroU64 {
        self.domain
    }
    pub fn statistics(&self) -> Statistics {
        self.stats
    }
    pub fn traces(&self) -> &[Trace] {
        &self.traces
    }
    pub fn state(&self, field: AttachedTextFieldId) -> Option<&state::TextFieldState> {
        self.fields.get(&field).map(|b| &b.state)
    }

    fn context_owner(&self, vm: &Vm, clip: AttachedClipId) -> Result<ObjectId, String> {
        if vm.root() != self.root {
            return Err("TextField host VM root changed".into());
        }
        let owner = vm.attached_owner(clip).map_err(|e| e.to_string())?;
        let mut current = clip;
        for _ in 0..self.limits.max_parent_depth {
            let info = vm.attached_info(current).map_err(|e| e.to_string())?;
            if info.owner == self.root {
                if info.parent.is_some() {
                    return Err("TextField host root is not an attachment root".into());
                }
                return Ok(owner);
            }
            current = info
                .parent
                .ok_or("TextField context belongs to another root")?;
        }
        Err("TextField context parent depth exceeded".into())
    }
    pub fn bind_context(&mut self, vm: &Vm, clip: AttachedClipId) -> Result<(), String> {
        let owner = self.context_owner(vm, clip)?;
        if !self.contexts.contains(&owner) && self.contexts.len() >= self.limits.max_contexts {
            return Err("TextField context budget exceeded".into());
        }
        self.contexts.insert(owner);
        Ok(())
    }

    /// Both native snapshot adoption and conditional TextFieldState admission
    /// must have succeeded. Validate exact source/instance/parent associations;
    /// no authored placeholder, format, watch state or environment is invented.
    pub fn bind(
        &mut self,
        vm: &Vm,
        display: &DisplayState,
        field: AttachedTextFieldId,
        state: state::TextFieldState,
    ) -> Result<(), String> {
        let info = vm
            .attached_text_field_info(field)
            .map_err(|e| e.to_string())?;
        let identity = state.identity();
        if identity.domain != self.domain || identity.avm_owner != info.owner.0 {
            return Err("TextField state identity/domain differs from adopted owner".into());
        }
        if display.bindings().get(&self.root) != Some(&display.root()) {
            return Err("TextField scene does not bind the owning root".into());
        }
        self.context_owner(vm, info.parent)?;
        let environment_owner = self.context_owner(vm, info.environment_owner)?;
        if info.watch_state != TextFieldWatchState::Absent {
            return Err("TextField host requires the proved absent-watch profile".into());
        }
        let node = &display
            .node(identity.scene_node)
            .map_err(|e| e.to_string())?
            .source;
        let definition = node
            .kind
            .definition()
            .ok_or("TextField scene node has no definition")?;
        let movie = display
            .movie_for_source(info.source.movie)
            .ok_or("TextField source movie is not bound")?;
        if definition.kind != CharacterKind::Text
            || display.tag_code(movie, definition.tag_index) != Some(37)
            || definition.character_id != info.source.edit_text_id
            || definition.movie_id != movie
            || display.movie_version(movie) != Some(info.source.header_version)
        {
            return Err("TextField definition/source/header differs from scene".into());
        }
        if node.parent != display.node_for_attached(info.parent) {
            return Err("TextField parent differs from checked scene attachment".into());
        }
        if node.parent.is_none() {
            return Err("TextField requires a bound scene parent".into());
        }
        if self.fields.contains_key(&field)
            || self.nodes.contains_key(&identity.scene_node)
            || self.owners.contains_key(&info.owner)
        {
            return Err("TextField identity is already bound".into());
        }
        if self.fields.len() >= self.limits.max_fields {
            return Err("TextField count budget exceeded".into());
        }
        let bytes = retained_bytes(&state)?;
        let total = self
            .stats
            .retained_text_bytes
            .checked_add(bytes)
            .ok_or("TextField retained byte overflow")?;
        if total > self.limits.max_retained_text_bytes {
            return Err("TextField retained text budget exceeded".into());
        }
        self.nodes.insert(identity.scene_node, field);
        self.owners.insert(info.owner, field);
        self.fields.insert(
            field,
            Binding {
                info,
                environment_owner,
                state,
            },
        );
        self.stats.retained_text_bytes = total;
        Ok(())
    }

    fn admit(&mut self, request: &TextFieldMemberRequest) -> Result<ObjectId, String> {
        if self.stats.calls >= self.limits.max_calls {
            return Err("TextField call budget exceeded".into());
        }
        self.stats.calls += 1;
        let context = request
            .context
            .ok_or("TextField request has no active caller target")?;
        if !self.contexts.contains(&context) {
            return Err("TextField caller context is not bound".into());
        }
        let binding = self
            .fields
            .get(&request.field)
            .ok_or("TextField target is not bound")?;
        let source = binding.info.source;
        if request.owner != binding.info.owner
            || request.target != binding.info.target
            || request.environment_owner != binding.environment_owner
            || request.source.movie != source.movie
            || request.source.edit_text_id != source.edit_text_id
            || request.source.header_version != source.header_version
        {
            return Err(
                "TextField request identity/source/environment differs from binding".into(),
            );
        }
        Ok(context)
    }
    fn trace(&mut self, request: &TextFieldMemberRequest, context: ObjectId, operation: Operation) {
        self.traces.push(Trace {
            origin: request.origin,
            source_offset: request.source_offset,
            field: request.field,
            node: self.fields[&request.field].state.identity().scene_node,
            context,
            environment_owner: request.environment_owner,
            member: request.member,
            operation,
        });
    }
    pub fn get(&mut self, request: TextFieldMemberRequest) -> Result<Value, String> {
        let context = self.admit(&request)?;
        let state = &self.fields[&request.field].state;
        let (value, operation) = match request.member {
            TextFieldMember::Html => (Value::Bool(state.html()), Operation::ReadBool(state.html())),
            TextFieldMember::Text | TextFieldMember::HtmlText => {
                let text = if request.member == TextFieldMember::Text {
                    state.text()
                } else {
                    state.html_text().map_err(|e| e.to_string())?
                };
                (
                    Value::string(text),
                    Operation::ReadString {
                        utf16_units: text.encode_utf16().count(),
                    },
                )
            }
        };
        self.stats.reads += 1;
        self.trace(&request, context, operation);
        Ok(value)
    }
    pub fn set(&mut self, request: TextFieldMemberRequest, value: Value) -> Result<(), String> {
        let context = self.admit(&request)?;
        let binding = self.fields.get_mut(&request.field).unwrap();
        let operation = match (request.member, value) {
            (TextFieldMember::Html, Value::Bool(enabled)) => {
                Operation::SetHtml(binding.state.set_html(enabled))
            }
            (TextFieldMember::Text | TextFieldMember::HtmlText, Value::String(units)) => {
                // Bound before conversion; this backend's admitted text is ASCII.
                if units.len() > state::MAX_TEXT_BYTES || units.contains(&0) {
                    return Err("TextField string exceeds its supported input profile".into());
                }
                let input = String::from_utf16(&units).map_err(|e| e.to_string())?;
                let old = retained_bytes(&binding.state)?;
                let unchanged =
                    input == binding.state.snapshot().last_input && !binding.state.force_rebuild();
                let next = if unchanged {
                    old
                } else {
                    input
                        .len()
                        .checked_mul(2)
                        .ok_or("TextField text size overflow")?
                };
                let total = self
                    .stats
                    .retained_text_bytes
                    .checked_sub(old)
                    .and_then(|n| n.checked_add(next))
                    .ok_or("TextField retained text accounting overflow")?;
                if total > self.limits.max_retained_text_bytes {
                    return Err("TextField retained text budget exceeded".into());
                }
                let result = if request.member == TextFieldMember::Text {
                    binding.state.set_text(&input)
                } else {
                    binding.state.set_html_text(&input)
                }
                .map_err(|e| e.to_string())?;
                self.stats.retained_text_bytes = total;
                if matches!(result, state::Replacement::Replaced { .. }) {
                    self.stats.replacements += 1;
                }
                Operation::Replace(result)
            }
            _ => return Err("TextField member requires its actual Boolean or String value".into()),
        };
        self.stats.writes += 1;
        self.trace(&request, context, operation);
        Ok(())
    }
}
fn retained_bytes(state: &state::TextFieldState) -> Result<usize, String> {
    state
        .snapshot()
        .last_input
        .len()
        .checked_add(state.text().len())
        .ok_or_else(|| "TextField retained text size overflow".into())
}

#[cfg(test)]
mod tests {
    use super::Limits;
    use super::*;
    use crate::{
        avm1::{self, vm::*},
        frontend::{display_state, host::FrontendHost},
        gfx::{self, scene},
    };
    use std::sync::Arc;

    fn fixture() -> (Vm, DisplayState, [AttachedTextFieldId; 2]) {
        // Synthetic source metadata, not copied game definitions or runtime defaults.
        let mut movie = gfx::Movie::parse(&[
            b'F', b'W', b'S', 15, 18, 0, 0, 0, 8, 0, 0, 12, 1, 0, 64, 0, 0, 0,
        ])
        .unwrap();
        movie.tags.clear();
        movie.timelines.clear();
        movie.dictionary.insert(
            7,
            gfx::Character {
                id: 7,
                kind: CharacterKind::Text,
                tag_index: 0,
                bounds: None,
            },
        );
        movie.tags.push(gfx::Tag {
            code: 37,
            record_offset: 0,
            body: gfx::Span { offset: 0, len: 0 },
            timeline_id: 0,
            frame_index: 0,
            data: gfx::TagData::Definition {
                id: 7,
                bounds: None,
            },
        });
        for depth in 1..=2 {
            let placement = gfx::Placement {
                depth,
                move_existing: false,
                character_id: Some(7),
                matrix: None,
                color: None,
                ratio: None,
                name: None,
                clip_depth: None,
                clip_actions: Vec::new(),
                has_clip_actions: false,
                filters: None,
                blend_mode: None,
                bitmap_cache: None,
                visible: None,
                opaque_background: None,
                class_name: None,
            };
            movie.tags.push(gfx::Tag {
                code: 26,
                record_offset: 0,
                body: gfx::Span { offset: 0, len: 0 },
                timeline_id: 0,
                frame_index: 0,
                data: gfx::TagData::Place(placement),
            });
        }
        movie.tags.push(gfx::Tag {
            code: 1,
            record_offset: 0,
            body: gfx::Span { offset: 0, len: 0 },
            timeline_id: 0,
            frame_index: 0,
            data: gfx::TagData::ShowFrame,
        });
        movie.timelines.insert(
            0,
            gfx::Timeline {
                id: 0,
                frame_count: 1,
                tags: vec![1, 2, 3],
            },
        );
        let scene = scene::Scene::first_frame(
            scene::MovieRef::new(10, &movie),
            &scene::NoImports,
            scene::Limits::default(),
        )
        .unwrap();
        let mut display =
            DisplayState::from_scene(&scene, display_state::Limits::default()).unwrap();
        let mut vm = Vm::new(15, crate::avm1::vm::Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        let root = vm
            .create_attached_movie_clip(AttachedClipSpec {
                target: DisplayTargetId(100),
                source: AttachedClipSource {
                    movie: LoadedMovieSourceId(1),
                    sprite_id: 0,
                    header_version: 15,
                },
                parent: None,
                event_state: AttachedEventState {
                    target_flags: 0x5200_0580,
                    load_flags: 0,
                    source_flags: 0,
                },
            })
            .unwrap();
        vm.adopt_attached_root(root).unwrap();
        display
            .bind_movie_source(LoadedMovieSourceId(1), 10)
            .unwrap();
        display.bind_attached(&vm, root, 0).unwrap();
        let fields = [200, 201].map(|target| {
            vm.adopt_attached_text_field(AttachedTextFieldSpec {
                target: DisplayTargetId(target),
                source: AttachedTextFieldSource {
                    movie: LoadedMovieSourceId(1),
                    edit_text_id: 7,
                    header_version: 15,
                },
                parent: root,
                environment_owner: root,
                target_flags_6a: 0x5200_0180,
                prototype: TextFieldPrototypeId(1000),
                watch_state: TextFieldWatchState::Absent,
            })
            .unwrap()
        });
        vm.set_attached_children(
            root,
            vec![
                AttachedChild::TextField {
                    name: text("localizedText_tf"),
                    field: fields[0],
                },
                AttachedChild::TextField {
                    name: text("button"),
                    field: fields[1],
                },
            ],
        )
        .unwrap();
        (vm, display, fields)
    }
    fn field_state(
        vm: &Vm,
        field: AttachedTextFieldId,
        node: usize,
        domain: u64,
    ) -> state::TextFieldState {
        let format = state::FormatProfile {
            text: state::TextFormat(NonZeroU64::new(1).unwrap()),
            paragraph: state::ParagraphFormat(NonZeroU64::new(2).unwrap()),
        };
        state::TextFieldState::from_snapshot(
            state::AttachedIdentity {
                domain: NonZeroU64::new(domain).unwrap(),
                scene_node: node,
                avm_owner: vm.attached_text_field_owner(field).unwrap().0,
            },
            state::Profile {
                watches: state::SidePath::Absent,
                translation_service: state::SidePath::Absent,
                stylesheet: state::SidePath::Absent,
                editor: state::SidePath::Absent,
                variable_binding: state::SidePath::Absent,
                image_callbacks: state::SidePath::Absent,
            },
            state::Snapshot {
                raw_target_flags: 0,
                raw_view_flags_149: 0,
                last_input: "seed".into(),
                document: state::Document {
                    text: "seed".into(),
                    run_format: Some(format),
                },
                current_defaults: format,
                definition_html: state::DefinitionHtmlProfile {
                    run_format: format,
                    defaults_after_nonempty: format,
                },
                pending_invalidation: state::Invalidation {
                    view_mask: 0,
                    render_dirty: false,
                },
            },
            state::Limits::default(),
        )
        .unwrap()
    }
    fn request(
        vm: &Vm,
        field: AttachedTextFieldId,
        member: TextFieldMember,
    ) -> TextFieldMemberRequest {
        let info = vm.attached_text_field_info(field).unwrap();
        TextFieldMemberRequest {
            origin: None,
            source_offset: None,
            context: Some(vm.root()),
            environment_owner: vm.attached_owner(info.environment_owner).unwrap(),
            field,
            owner: info.owner,
            target: info.target,
            source: info.source,
            member,
        }
    }
    fn push_string(code: &mut Vec<u8>, value: &str) {
        code.push(0x96);
        code.extend_from_slice(&((value.len() + 2) as u16).to_le_bytes());
        code.push(0);
        code.extend_from_slice(value.as_bytes());
        code.push(0);
    }
    fn receiver(code: &mut Vec<u8>, name: &str) {
        push_string(code, name);
        code.push(0x1c);
    }
    fn execute(
        vm: &mut Vm,
        code: Vec<u8>,
        host: &mut FrontendHost,
    ) -> crate::avm1::vm::Result<Execution> {
        let id = vm
            .register(Arc::new(avm1::Program::parse(&code).unwrap()))
            .unwrap();
        vm.execute(id, vm.root(), host)
    }

    #[test]
    fn typed_vm_writes_change_one_instance_and_failed_backend_keeps_native_storage() {
        let (mut vm, display, fields) = fixture();
        let mut states =
            TextFieldHost::new(vm.root(), NonZeroU64::new(1).unwrap(), Limits::default()).unwrap();
        for (index, field) in fields.into_iter().enumerate() {
            states
                .bind(&vm, &display, field, field_state(&vm, field, index + 1, 1))
                .unwrap();
        }
        let mut host = FrontendHost::install(&mut vm, 15).unwrap();
        host.attach_display(display).unwrap();
        host.attach_text_fields(states).unwrap();
        let mut code = Vec::new();
        receiver(&mut code, "localizedText_tf");
        push_string(&mut code, "html");
        code.extend([0x96, 2, 0, 5, 1, 0x4f]);
        receiver(&mut code, "localizedText_tf");
        push_string(&mut code, "htmlText");
        push_string(&mut code, "Changed");
        code.push(0x4f);
        receiver(&mut code, "localizedText_tf");
        push_string(&mut code, "text");
        code.extend([0x4e, 0x3e, 0]);
        let result = execute(&mut vm, code, &mut host).unwrap();
        assert_eq!(result.value, Value::string("Changed"));
        assert_eq!(result.host_calls, 3);
        let state = host.text_fields().unwrap();
        assert_eq!(state.state(fields[0]).unwrap().text(), "Changed");
        assert!(state.state(fields[0]).unwrap().html());
        assert_eq!(state.state(fields[1]).unwrap().text(), "seed");
        assert_eq!(state.statistics().retained_text_bytes, 22);
        assert_eq!(state.traces().len(), 3);
        assert!(
            state
                .traces()
                .iter()
                .all(|t| t.origin.is_some() && t.source_offset.is_some())
        );
        assert!(
            vm.attached_text_field_info(fields[0])
                .unwrap()
                .storage
                .is_some()
        );
        assert!(
            vm.attached_text_field_info(fields[1])
                .unwrap()
                .storage
                .is_none()
        );
        let mut rejected = Vec::new();
        receiver(&mut rejected, "button");
        push_string(&mut rejected, "htmlText");
        push_string(&mut rejected, "bad<");
        rejected.extend([0x4f, 0]);
        assert!(execute(&mut vm, rejected, &mut host).is_err());
        assert!(
            vm.attached_text_field_info(fields[1])
                .unwrap()
                .storage
                .is_some()
        );
        assert_eq!(
            host.text_fields().unwrap().state(fields[1]).unwrap().text(),
            "seed"
        );
        assert_eq!(
            host.text_fields().unwrap().statistics().retained_text_bytes,
            22
        );
        assert!(
            host.text_fields_mut()
                .unwrap()
                .get(request(&vm, fields[0], TextFieldMember::HtmlText))
                .is_err()
        );
    }

    #[test]
    fn binding_request_and_retention_failures_preserve_the_admitted_state() {
        let (vm, display, fields) = fixture();
        let mut host = TextFieldHost::new(
            vm.root(),
            NonZeroU64::new(1).unwrap(),
            Limits {
                max_retained_text_bytes: 15,
                ..Limits::default()
            },
        )
        .unwrap();
        assert!(
            host.bind(&vm, &display, fields[0], field_state(&vm, fields[0], 1, 2))
                .is_err()
        );
        assert!(
            host.bind(&vm, &display, fields[0], field_state(&vm, fields[0], 0, 1))
                .is_err()
        );
        host.bind(&vm, &display, fields[0], field_state(&vm, fields[0], 1, 1))
            .unwrap();
        assert!(
            host.bind(&vm, &display, fields[1], field_state(&vm, fields[1], 2, 1))
                .is_err()
        );
        assert_eq!(host.statistics().retained_text_bytes, 8);
        let mut bad = request(&vm, fields[0], TextFieldMember::Text);
        bad.context = None;
        assert!(host.get(bad).is_err());
        let mut bad = request(&vm, fields[0], TextFieldMember::Text);
        bad.owner = vm.attached_text_field_owner(fields[1]).unwrap();
        assert!(host.get(bad).is_err());
        let mut bad = request(&vm, fields[0], TextFieldMember::Text);
        bad.environment_owner = bad.owner;
        assert!(host.get(bad).is_err());
        assert!(
            host.set(
                request(&vm, fields[0], TextFieldMember::Text),
                Value::string("12345678")
            )
            .is_err()
        );
        assert_eq!(host.state(fields[0]).unwrap().text(), "seed");
        assert_eq!(host.statistics().retained_text_bytes, 8);
        host.set(
            request(&vm, fields[0], TextFieldMember::Text),
            Value::string("1234567"),
        )
        .unwrap();
        assert_eq!(host.statistics().retained_text_bytes, 14);
        assert_eq!(host.traces().len(), 1);
    }
}
