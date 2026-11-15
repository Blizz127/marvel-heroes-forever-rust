//! Immutable projection of one already-associated frontend host and its fields.
//! No lifecycle, script execution, format defaults or invalidation acknowledgement.

use super::{
    AssetSet, display_state,
    host::FrontendHost,
    library::MovieLibrary,
    runtime_text::{self, FieldBinding, FieldSnapshot, FormatBinding},
    scene_content::{self, PreparedScene},
    textfield_host,
};
use crate::{
    avm1::vm::AttachedTextFieldId,
    gfx::{Movie, scene::MovieId},
};
use std::{collections::BTreeMap, fmt, io, num::NonZeroU64};

#[derive(Clone, Copy, Debug, Default)]
pub struct Limits {
    pub text: runtime_text::Limits,
    pub content: scene_content::Limits,
}

/// The full bound-field projection, including unchanged fields and invalidation.
/// This records caller-supplied format mappings; it does not establish them.
#[derive(Clone, Debug)]
pub struct TextSnapshot {
    pub domain: NonZeroU64,
    pub fields: Vec<FieldSnapshot>,
    pub statistics: runtime_text::Statistics,
    pub host_statistics: textfield_host::Statistics,
}

/// Owned frame preparation. Later host writes do not change this value.
#[derive(Clone, Debug)]
pub struct CurrentScene {
    pub prepared: PreparedScene,
    pub display_revision: u64,
    pub display_statistics: display_state::Statistics,
    /// None means no TextFieldHost; Some(empty) retains an attached empty host.
    pub text: Option<TextSnapshot>,
}

#[derive(Debug)]
pub enum Error {
    MissingDisplay,
    BindingsWithoutTextFieldHost {
        fields: usize,
        formats: usize,
    },
    FieldCoverage {
        bound_fields: usize,
        supplied_fields: usize,
        missing: Option<AttachedTextFieldId>,
    },
    Display(display_state::DisplayError),
    Text(runtime_text::Error),
    Content(io::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Display(e) => write!(f, "current scene: {e}"),
            Self::Text(e) => write!(f, "current scene: {e}"),
            Self::Content(e) => write!(f, "current scene content: {e}"),
            other => write!(f, "current scene: {other:?}"),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Display(e) => Some(e),
            Self::Text(e) => Some(e),
            Self::Content(e) => Some(e),
            _ => None,
        }
    }
}
pub type Result<T> = std::result::Result<T, Error>;

/// Prepare one immutable current frame from the same VM/Scene host association.
/// Source movies must be the corresponding library entries; the display checks
/// their retained byte hashes before scene-content checks library identity.
///
/// Every bound TextField must have exactly one explicit FieldBinding, even when
/// unchanged or hidden. Snapshot/format failures reject the whole preparation;
/// missing current text never falls back to saved text. Unbound text nodes keep
/// the existing saved preparation behavior. Formats, empty-document policy and
/// host domain remain explicit caller inputs, not discovered native state.
pub fn prepare<'movie>(
    host: &FrontendHost,
    movies: BTreeMap<MovieId, &'movie Movie>,
    assets: &AssetSet,
    library: &MovieLibrary,
    fields: &[FieldBinding],
    formats: &[FormatBinding],
    limits: Limits,
) -> Result<CurrentScene> {
    let display = host.display().ok_or(Error::MissingDisplay)?;
    let scene = display.snapshot_scene(movies).map_err(Error::Display)?;
    let display_statistics = display.stats();
    let (prepared, text) = match host.text_fields() {
        None => {
            if !fields.is_empty() || !formats.is_empty() {
                return Err(Error::BindingsWithoutTextFieldHost {
                    fields: fields.len(),
                    formats: formats.len(),
                });
            }
            (
                scene_content::prepare_with_text_overrides(
                    assets,
                    library,
                    &scene,
                    &[],
                    limits.content,
                )
                .map_err(Error::Content)?,
                None,
            )
        }
        Some(text_host) => {
            let bound_fields = text_host.bound_fields().len();
            if bound_fields > limits.text.max_fields {
                return Err(Error::Text(runtime_text::Error::Limit("bound field count")));
            }
            let batch =
                runtime_text::snapshot_overrides(text_host, &scene, fields, formats, limits.text)
                    .map_err(Error::Text)?;
            // snapshot_overrides has already rejected duplicates, unknown fields,
            // wrong identities and invalid formats. Check the complementary set:
            // omission must not reactivate a bound field's authored initial text.
            let supplied: std::collections::BTreeSet<_> =
                batch.snapshots().iter().map(|field| field.field).collect();
            let missing = text_host
                .bound_fields()
                .find(|field| !supplied.contains(field));
            if missing.is_some() || supplied.len() != bound_fields {
                return Err(Error::FieldCoverage {
                    bound_fields,
                    supplied_fields: supplied.len(),
                    missing,
                });
            }
            let prepared = scene_content::prepare_with_text_overrides(
                assets,
                library,
                batch.scene(),
                batch.overrides(),
                limits.content,
            )
            .map_err(Error::Content)?;
            (
                prepared,
                Some(TextSnapshot {
                    domain: text_host.domain(),
                    fields: batch.snapshots().to_vec(),
                    statistics: batch.statistics(),
                    host_statistics: text_host.statistics(),
                }),
            )
        }
    };
    Ok(CurrentScene {
        prepared,
        display_revision: display_statistics.revision,
        display_statistics,
        text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        avm1::vm::{
            self, AttachedClipSource, AttachedClipSpec, AttachedEventState,
            AttachedTextFieldSource, AttachedTextFieldSpec, DisplayTargetId, LoadedMovieSourceId,
            MovieProperty, PropertyRequest, TextFieldMember, TextFieldMemberRequest,
            TextFieldPrototypeId, TextFieldWatchState, Value, Vm,
        },
        frontend::{
            runtime_text::EmptyDocumentStyle,
            scene_content::{RuntimeFont, RuntimeTextEffects, RuntimeTextStyle},
            textfield_host::TextFieldHost,
            textfield_state as state,
        },
        gfx::{
            self,
            glyphs::FontIdentity,
            scene::{MovieRef, Scene},
            text::EditText,
        },
    };

    struct Fixture {
        assets: AssetSet,
        library: MovieLibrary,
        host: FrontendHost,
        vm: Vm,
        fields: [FieldBinding; 2],
        formats: [FormatBinding; 1],
    }
    impl Fixture {
        fn new(with_text_host: bool) -> Self {
            let (assets, library) = synthetic_fixture().unwrap();
            let movie = &library.entry(0).unwrap().asset.movie;
            let scene =
                Scene::first_frame(MovieRef::new(0, movie), &library, Default::default()).unwrap();
            assert_eq!(scene.nodes.len(), 3);
            let definition = scene.nodes[1].kind.definition().unwrap();
            let tag = &movie.tags[definition.tag_index];
            let field =
                EditText::parse(tag.body.bytes(movie.bytes()).unwrap(), tag.body.offset).unwrap();
            let mut display =
                display_state::DisplayState::from_scene(&scene, Default::default()).unwrap();
            let mut vm = Vm::new(15, vm::Limits::default()).unwrap();
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
                        target_flags: 0x5200_4580,
                        load_flags: 0,
                        source_flags: 0,
                    },
                })
                .unwrap();
            vm.adopt_attached_root(root).unwrap();
            display
                .bind_movie_source(LoadedMovieSourceId(1), 0)
                .unwrap();
            display.bind_attached(&vm, root, 0).unwrap();
            let mut host = FrontendHost::install(&mut vm, 15).unwrap();
            let domain = NonZeroU64::new(81).unwrap();
            let profile = state::FormatProfile {
                text: state::TextFormat(NonZeroU64::new(1).unwrap()),
                paragraph: state::ParagraphFormat(NonZeroU64::new(2).unwrap()),
            };
            let mut text_host = TextFieldHost::new(vm.root(), domain, Default::default()).unwrap();
            let fields = [1, 2].map(|node| {
                let id = vm
                    .adopt_attached_text_field(AttachedTextFieldSpec {
                        target: DisplayTargetId(200 + node as u64),
                        source: AttachedTextFieldSource {
                            movie: LoadedMovieSourceId(1),
                            edit_text_id: field.id,
                            header_version: 15,
                        },
                        parent: root,
                        environment_owner: root,
                        target_flags_6a: 0x5200_0180,
                        prototype: TextFieldPrototypeId(1000),
                        watch_state: TextFieldWatchState::Absent,
                    })
                    .unwrap();
                let identity = state::AttachedIdentity {
                    domain,
                    scene_node: node,
                    avm_owner: vm.attached_text_field_owner(id).unwrap().0,
                };
                let state = state::TextFieldState::from_snapshot(
                    identity,
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
                        last_input: "A".into(),
                        document: state::Document {
                            text: "A".into(),
                            run_format: Some(profile),
                        },
                        current_defaults: profile,
                        definition_html: state::DefinitionHtmlProfile {
                            run_format: profile,
                            defaults_after_nonempty: profile,
                        },
                        pending_invalidation: state::Invalidation {
                            view_mask: 0,
                            render_dirty: false,
                        },
                    },
                    Default::default(),
                )
                .unwrap();
                text_host.bind(&vm, &display, id, state).unwrap();
                FieldBinding {
                    field: id,
                    identity,
                    expected_definition: scene.nodes[node].kind.definition().unwrap(),
                    empty_style: EmptyDocumentStyle::Reject,
                }
            });
            let formats = [FormatBinding { domain, profile, style: RuntimeTextStyle {
                flags: field.flags, font: RuntimeFont { movie_id: 0, character_id: 1, expected_resolved: FontIdentity { movie_id: 0, character_id: 1 } },
                height_twips: field.font_height.unwrap(), color_rgba: field.color.unwrap(), layout_twips: field.layout.unwrap(),
                effects: RuntimeTextEffects { kerning: Some(false), letter_spacing_twips: Some(0), underline: Some(false), block_indent_twips: Some(0) },
            }, provenance: "Original synthetic single-font fixture: explicit uniform style and inactive effects".into() }];
            host.attach_display(display).unwrap();
            if with_text_host {
                host.attach_text_fields(text_host).unwrap();
            }
            Self {
                assets,
                library,
                host,
                vm,
                fields,
                formats,
            }
        }
        fn prepare(
            &self,
            fields: &[FieldBinding],
            formats: &[FormatBinding],
            limits: Limits,
        ) -> Result<CurrentScene> {
            prepare(
                &self.host,
                BTreeMap::from([(0, &self.library.entry(0).unwrap().asset.movie)]),
                &self.assets,
                &self.library,
                fields,
                formats,
                limits,
            )
        }
        fn set_text(&mut self, index: usize, value: &str) {
            let field = self.fields[index].field;
            let info = self.vm.attached_text_field_info(field).unwrap();
            self.host
                .text_fields_mut()
                .unwrap()
                .set(
                    TextFieldMemberRequest {
                        origin: None,
                        source_offset: None,
                        context: Some(self.vm.root()),
                        environment_owner: self.vm.attached_owner(info.environment_owner).unwrap(),
                        field,
                        owner: info.owner,
                        target: info.target,
                        source: info.source,
                        member: TextFieldMember::Text,
                    },
                    Value::string(value),
                )
                .unwrap();
        }
        fn observe(&self) -> String {
            format!(
                "{:?}|{}|{:?}|{}|{:?}",
                self.host.display().unwrap().stats(),
                self.host.display().unwrap().traces().len(),
                self.host.text_fields().map(TextFieldHost::statistics),
                self.host.text_fields().map_or(0, |h| h.traces().len()),
                self.host.text_fields().map(|h| self
                    .fields
                    .iter()
                    .map(|f| h.state(f.field).unwrap().snapshot())
                    .collect::<Vec<_>>())
            )
        }
    }
    fn texts(frame: &CurrentScene) -> Vec<&str> {
        frame
            .prepared
            .texts
            .iter()
            .filter_map(|t| t.runtime.as_ref().map(|r| r.text.as_str()))
            .collect()
    }

    #[test]
    fn current_display_and_all_documents_are_frozen_without_acknowledgement() {
        let mut f = Fixture::new(true);
        f.set_text(0, "AA");
        f.host
            .display_mut()
            .unwrap()
            .write_property(
                PropertyRequest {
                    origin: None,
                    context: f.vm.root(),
                    target: Value::string(""),
                    property: MovieProperty::X,
                    source_offset: None,
                },
                Value::Number(3.0),
            )
            .unwrap();
        let before = f.observe();
        let first = f.prepare(&f.fields, &f.formats, Limits::default()).unwrap();
        assert_eq!(before, f.observe());
        assert_eq!(first.display_revision, 1);
        assert_eq!(texts(&first), ["AA", "A"]);
        assert_eq!(
            first
                .prepared
                .nodes
                .iter()
                .map(|n| n.instance.world_matrix.tx)
                .collect::<Vec<_>>(),
            [60.0, 60.0, 1160.0]
        );
        assert!(
            first
                .prepared
                .nodes
                .iter()
                .all(|n| n.instance.effective_visible)
        );
        let meta = first.text.as_ref().unwrap();
        assert_eq!(meta.fields.len(), 2);
        assert_eq!(meta.statistics.fields, 2);
        assert_eq!(meta.statistics.document_bytes, 3);
        assert!(meta.fields[0].pending_invalidation.render_dirty);
        assert!(!meta.fields[1].pending_invalidation.render_dirty);
        assert_eq!(
            first
                .prepared
                .texts
                .iter()
                .map(|t| t.glyphs.len())
                .sum::<usize>(),
            3
        );
        f.set_text(1, "AA");
        let second = f.prepare(&f.fields, &f.formats, Limits::default()).unwrap();
        assert_eq!(texts(&first), ["AA", "A"]);
        assert_eq!(texts(&second), ["AA", "AA"]);
        assert_eq!(first.display_revision, second.display_revision);
        assert_ne!(
            first.text.unwrap().host_statistics,
            second.text.unwrap().host_statistics
        );
    }

    #[test]
    fn every_bound_field_is_required_and_errors_preserve_state() {
        let mut f = Fixture::new(true);
        f.set_text(0, "AA");
        let before = f.observe();
        for supplied in [&[][..], &f.fields[..1]] {
            assert!(matches!(
                f.prepare(supplied, &f.formats, Limits::default()),
                Err(Error::FieldCoverage {
                    bound_fields: 2,
                    ..
                })
            ));
            assert_eq!(before, f.observe());
        }
        assert!(matches!(
            f.prepare(&[f.fields[0], f.fields[0]], &f.formats, Limits::default()),
            Err(Error::Text(runtime_text::Error::DuplicateField { .. }))
        ));
        let mut wrong = f.fields;
        wrong[1].field = AttachedTextFieldId(usize::MAX);
        assert!(matches!(
            f.prepare(&wrong, &f.formats, Limits::default()),
            Err(Error::Text(runtime_text::Error::MissingField { .. }))
        ));
        wrong = f.fields;
        wrong[1].identity.domain = NonZeroU64::new(82).unwrap();
        assert!(matches!(
            f.prepare(&wrong, &f.formats, Limits::default()),
            Err(Error::Text(runtime_text::Error::FieldDomain { .. }))
        ));
        assert!(matches!(
            f.prepare(&f.fields, &[], Limits::default()),
            Err(Error::Text(runtime_text::Error::MissingFormat { .. }))
        ));
        let mut limits = Limits::default();
        limits.text.max_fields = 1;
        assert!(matches!(
            f.prepare(&f.fields, &f.formats, limits),
            Err(Error::Text(runtime_text::Error::Limit(_)))
        ));
        let mut formats = f.formats.clone();
        formats[0].style.font.expected_resolved.character_id = 999;
        assert!(matches!(
            f.prepare(&f.fields, &formats, Limits::default()),
            Err(Error::Content(_))
        ));
        assert_eq!(before, f.observe());
    }

    #[test]
    fn empty_document_policy_and_hidden_fields_never_reactivate_saved_text() {
        let mut f = Fixture::new(true);
        f.set_text(0, "");
        assert!(matches!(
            f.prepare(&f.fields, &f.formats, Limits::default()),
            Err(Error::Text(runtime_text::Error::EmptyDocument { .. }))
        ));
        let mut fields = f.fields;
        fields[0].empty_style = EmptyDocumentStyle::UseSuppliedCurrentDefaults;
        // Exercise the established attached visibility transaction. The numeric
        // _visible property setter is intentionally unsupported by DisplayState.
        let clip =
            f.vm.attached_text_field_info(f.fields[0].field)
                .unwrap()
                .parent;
        let info = f.vm.attached_info(clip).unwrap();
        let request = vm::AttachedVisibilityRequest {
            root: f.vm.root(),
            context: f.vm.root(),
            receiver: clip,
            owner: info.owner,
            target: info.target,
            expected_local: true,
            visible: false,
            origin: None,
        };
        let mut flags = info.event_state;
        flags.target_flags &= !0x4000;
        let commit = f
            .host
            .display_mut()
            .unwrap()
            .prepare_attached_visibility(request)
            .unwrap();
        f.vm.update_attached_event_state(clip, flags).unwrap();
        commit();
        let before = f.observe();
        assert!(matches!(
            f.prepare(&[], &[], Limits::default()),
            Err(Error::FieldCoverage { .. })
        ));
        let current = f.prepare(&fields, &f.formats, Limits::default()).unwrap();
        assert_eq!(texts(&current), ["", "A"]);
        assert_eq!(current.text.as_ref().unwrap().statistics.empty_documents, 1);
        assert!(
            current
                .prepared
                .nodes
                .iter()
                .all(|n| !n.instance.effective_visible)
        );
        assert!(
            current.text.as_ref().unwrap().fields[0]
                .pending_invalidation
                .render_dirty
        );
        assert_eq!(before, f.observe());
    }

    #[test]
    fn missing_host_saved_path_and_source_identity_remain_explicit() {
        let f = Fixture::new(false);
        let movie = &f.library.entry(0).unwrap().asset.movie;
        let scene =
            Scene::first_frame(MovieRef::new(0, movie), &f.library, Default::default()).unwrap();
        let saved =
            scene_content::prepare(&f.assets, &f.library, &scene, Default::default()).unwrap();
        let frame = f.prepare(&[], &[], Limits::default()).unwrap();
        assert!(frame.text.is_none());
        assert_eq!(format!("{saved:?}"), format!("{:?}", frame.prepared));
        assert!(matches!(
            f.prepare(&f.fields, &[], Limits::default()),
            Err(Error::BindingsWithoutTextFieldHost { .. })
        ));
        assert!(matches!(
            f.prepare(&[], &f.formats, Limits::default()),
            Err(Error::BindingsWithoutTextFieldHost { .. })
        ));
        assert!(matches!(
            prepare(
                &f.host,
                BTreeMap::new(),
                &f.assets,
                &f.library,
                &[],
                &[],
                Limits::default()
            ),
            Err(Error::Display(_))
        ));
        let clone = gfx::Movie::parse(movie.bytes()).unwrap();
        assert!(matches!(
            prepare(
                &f.host,
                BTreeMap::from([(0, &clone)]),
                &f.assets,
                &f.library,
                &[],
                &[],
                Limits::default()
            ),
            Err(Error::Content(_))
        ));
        let mut vm = Vm::new(15, vm::Limits::default()).unwrap();
        let host = FrontendHost::install(&mut vm, 15).unwrap();
        assert!(matches!(
            prepare(
                &host,
                BTreeMap::new(),
                &f.assets,
                &f.library,
                &[],
                &[],
                Limits::default()
            ),
            Err(Error::MissingDisplay)
        ));
    }

    #[test]
    fn asset_identity_rejects_a_separately_loaded_identical_package() {
        let f = Fixture::new(true);
        let (other_assets, other_library) = synthetic_fixture().unwrap();
        let original = &f.library.entry(0).unwrap().asset.movie;
        let other = &other_library.entry(0).unwrap().asset.movie;
        assert_eq!(original.bytes(), other.bytes());
        let before = f.observe();
        let error = prepare(
            &f.host,
            BTreeMap::from([(0, original)]),
            &other_assets,
            &f.library,
            &f.fields,
            &f.formats,
            Limits::default(),
        )
        .unwrap_err();
        assert!(matches!(&error, Error::Content(e)
            if e.kind() == io::ErrorKind::InvalidData
                && e.to_string() == "scene-content library and AssetSet identity differ"));
        assert_eq!(before, f.observe());
        // The original association still prepares all current fields.
        assert!(f.prepare(&f.fields, &f.formats, Limits::default()).is_ok());
    }

    use fixture_source::synthetic_fixture;
    mod fixture_source {
        // Original constructed UPK/SWF fixture; no retail bytes.
        // Original synthetic test-only UE3 wrapper; no retail bytes or guessed runtime defaults.
        // Intended to be combined with synthetic_movie() in the final fixture helper.
        use crate::{
            frontend::{
                AssetSet,
                library::{self, MovieLibrary},
            },
            upk::PackageArchive,
        };
        use std::io;

        /// One original SwfMovie export. All bytes are constructed test-format values.
        pub fn synthetic_package(movie: &[u8]) -> Vec<u8> {
            fn u32le(out: &mut Vec<u8>, value: u32) {
                out.extend_from_slice(&value.to_le_bytes());
            }
            fn fname(out: &mut Vec<u8>, index: u32) {
                u32le(out, index);
                u32le(out, 0);
            }
            fn string(out: &mut Vec<u8>, value: &str) {
                u32le(out, u32::try_from(value.len() + 1).unwrap());
                out.extend_from_slice(value.as_bytes());
                out.push(0);
            }
            fn patch(out: &mut [u8], at: usize, value: usize) {
                out[at..at + 4].copy_from_slice(&u32::try_from(value).unwrap().to_le_bytes());
            }
            let names = [
                "None",
                "Core",
                "Class",
                "Package",
                "Engine",
                "SwfMovie",
                "SyntheticMovie",
                "RawData",
                "ArrayProperty",
                "References",
            ];
            let mut payload = Vec::new();
            u32le(&mut payload, u32::MAX); // NetIndex, no state frame.
            fname(&mut payload, 7);
            fname(&mut payload, 8);
            u32le(&mut payload, u32::try_from(movie.len() + 4).unwrap());
            u32le(&mut payload, 0);
            u32le(&mut payload, u32::try_from(movie.len()).unwrap());
            payload.extend_from_slice(movie);
            fname(&mut payload, 9);
            fname(&mut payload, 8);
            u32le(&mut payload, 4);
            u32le(&mut payload, 0);
            u32le(&mut payload, 0);
            fname(&mut payload, 0);

            let mut out = Vec::new();
            u32le(&mut out, 0x9e2a83c1);
            u32le(&mut out, 868 | (3 << 16));
            u32le(&mut out, 0);
            string(&mut out, "None");
            u32le(&mut out, 0);
            u32le(&mut out, names.len() as u32);
            let names_slot = out.len();
            u32le(&mut out, 0);
            u32le(&mut out, 1);
            let exports_slot = out.len();
            u32le(&mut out, 0);
            u32le(&mut out, 2);
            let imports_slot = out.len();
            u32le(&mut out, 0);
            let depends_slot = out.len();
            u32le(&mut out, 0);
            for _ in 0..4 {
                u32le(&mut out, 0);
            } // GUID-table/counts, thumbnails.
            out.extend_from_slice(&[0; 16]);
            u32le(&mut out, 1); // One generation.
            for word in [1, names.len() as u32, 0, 10897, 136, 0, 0, 0] {
                u32le(&mut out, word);
            }
            let names_offset = out.len();
            for name in names {
                string(&mut out, name);
                out.extend_from_slice(&0u64.to_le_bytes());
            }
            let imports_offset = out.len();
            // Core.Class SwfMovie beneath an explicit synthetic Engine package import.
            fname(&mut out, 1);
            fname(&mut out, 2);
            u32le(&mut out, (-2i32) as u32);
            fname(&mut out, 5);
            fname(&mut out, 1);
            fname(&mut out, 3);
            u32le(&mut out, 0);
            fname(&mut out, 4);
            let exports_offset = out.len();
            u32le(&mut out, u32::MAX);
            u32le(&mut out, 0);
            u32le(&mut out, 0);
            fname(&mut out, 6);
            u32le(&mut out, 0);
            out.extend_from_slice(&0u64.to_le_bytes());
            u32le(&mut out, u32::try_from(payload.len()).unwrap());
            let serial_slot = out.len();
            u32le(&mut out, 0);
            u32le(&mut out, 0);
            u32le(&mut out, 0);
            out.extend_from_slice(&[0; 16]);
            u32le(&mut out, 0);
            let depends_offset = out.len();
            u32le(&mut out, 0);
            let header_size = out.len();
            for (slot, value) in [
                (8, header_size),
                (names_slot, names_offset),
                (imports_slot, imports_offset),
                (exports_slot, exports_offset),
                (depends_slot, depends_offset),
                (serial_slot, header_size),
            ] {
                patch(&mut out, slot, value);
            }
            out.extend_from_slice(&payload);
            out
        }

        /// Exercises real public package, movie and library readers; no test-only library builder.
        pub fn synthetic_fixture() -> io::Result<(AssetSet, MovieLibrary)> {
            let archive = PackageArchive::parse(synthetic_package(&synthetic_movie()))?;
            let assets = AssetSet::new(vec![archive])?;
            let library = MovieLibrary::load(&assets, &[0], &[], library::Limits::default())?;
            Ok((assets, library))
        }

        // Original synthetic data encoder for current-scene adapter tests.
        // Studied only this project's original gfx/text.rs, glyphs.rs and edit_layout.rs.
        // No retail bytes, third-party implementation, image assets, or native defaults.

        /// Font2 ID1 with U+0041; EditText ID7 placed at depths1/2 as first/second.
        ///
        /// Font:1024 units/em, ascent768, descent256, leading0, advance600.
        /// Glyph:one closed filled rectangle x[0,512],y[-768,0].
        /// Field:bounds(0,1000,0,500),flags0x8521,font1,height320,
        /// RGBA[17,34,51,255],left alignment and zero margins/indent/leading.
        /// Source text is "A". Plain runtime "A" and "AA" use the sole glyph.
        /// Stage:2400x1200twips,24fps,one frame. Second instance translates1100twips.
        /// For a single-movie library, RuntimeFont is movie0/character1 and resolves
        /// to FontIdentity{movie_id:0,character_id:1}; use actual library IDs if changed.
        /// Supply effects explicitly:kerning=false,letter_spacing=0,underline=false,
        /// block_indent=0. These are test inputs, not recovered native effective state.
        pub fn synthetic_movie() -> Vec<u8> {
            let mut tags = Vec::new();
            sf_tag(&mut tags, 48, &sf_font());
            sf_tag(&mut tags, 37, &sf_edit_text());
            sf_tag(&mut tags, 26, &sf_place(1, 0, "first"));
            sf_tag(&mut tags, 26, &sf_place(2, 1100, "second"));
            sf_tag(&mut tags, 1, &[]); // ShowFrame.
            sf_tag(&mut tags, 0, &[]); // End.

            let mut movie = b"FWS\x0f\0\0\0\0".to_vec();
            movie.extend(sf_rect([0, 2400, 0, 1200]));
            movie.extend((24u16 << 8).to_le_bytes());
            movie.extend(1u16.to_le_bytes());
            movie.extend(tags);
            let length = u32::try_from(movie.len()).expect("small synthetic movie");
            movie[4..8].copy_from_slice(&length.to_le_bytes());
            movie
        }

        fn sf_tag(output: &mut Vec<u8>, code: u16, body: &[u8]) {
            assert!(code < 1024);
            if body.len() < 63 {
                output.extend(((code << 6) | body.len() as u16).to_le_bytes());
            } else {
                output.extend(((code << 6) | 63).to_le_bytes());
                output.extend(u32::try_from(body.len()).unwrap().to_le_bytes());
            }
            output.extend_from_slice(body);
        }

        fn sf_font() -> Vec<u8> {
            let shape = sf_glyph();
            let name = b"Fixture";
            let mut body = 1u16.to_le_bytes().to_vec();
            body.extend([0x84, 0, name.len() as u8]); // Layout,wide codes;language0.
            body.extend_from_slice(name);
            body.extend(1u16.to_le_bytes()); // One glyph.
            body.extend(4u16.to_le_bytes()); // Shape starts after two u16 offsets.
            body.extend(u16::try_from(4 + shape.len()).unwrap().to_le_bytes());
            body.extend(shape);
            body.extend(65u16.to_le_bytes());
            body.extend(768u16.to_le_bytes()); // Ascent.
            body.extend(256u16.to_le_bytes()); // Descent.
            body.extend(0i16.to_le_bytes()); // Leading.
            body.extend(600i16.to_le_bytes()); // Single advance.
            body.extend(sf_rect([0, 512, -768, 0]));
            body.extend(0u16.to_le_bytes()); // Kerning pair count.
            body
        }

        fn sf_glyph() -> Vec<u8> {
            let mut bits = SfBits::new();
            bits.put(1, 4); // One fill-index bit,implicit glyph fill1.
            bits.put(0, 4); // No stroke-index bits.
            bits.put(0, 1); // Style-change record.
            bits.put(3, 5); // StateMoveTo and StateFillStyle0.
            bits.put(11, 5);
            bits.signed(0, 11);
            bits.signed(-768, 11);
            bits.put(1, 1); // FillStyle0=1.
            for (dx, dy) in [(512, 0), (0, 768), (-512, 0), (0, -768)] {
                bits.put(1, 1); // Edge record.
                bits.put(1, 1); // Straight edge.
                bits.put(9, 4); // Eleven-bit signed deltas (NumBits+2).
                bits.put(1, 1); // General line,serializes both deltas.
                bits.signed(dx, 11);
                bits.signed(dy, 11);
            }
            bits.put(0, 6); // EndShape.
            bits.finish()
        }

        fn sf_edit_text() -> Vec<u8> {
            let mut body = 7u16.to_le_bytes().to_vec();
            body.extend(sf_rect([0, 1000, 0, 500]));
            body.extend(0x8521u16.to_be_bytes()); // Text,font,color,layout,outlines.
            body.extend(1u16.to_le_bytes());
            body.extend(320u16.to_le_bytes());
            body.extend([17, 34, 51, 255]);
            body.push(0); // Left alignment.
            body.extend(0u16.to_le_bytes()); // Left margin.
            body.extend(0u16.to_le_bytes()); // Right margin.
            body.extend(0u16.to_le_bytes()); // Indent.
            body.extend(0i16.to_le_bytes()); // Leading.
            body.push(0); // Empty variable name.
            body.extend_from_slice(b"A\0");
            body
        }

        fn sf_place(depth: u16, translate_x: i32, name: &str) -> Vec<u8> {
            let mut body = vec![0x26]; // HasName,HasMatrix,HasCharacter;not Move.
            body.extend(depth.to_le_bytes());
            body.extend(7u16.to_le_bytes());
            let mut matrix = SfBits::new();
            matrix.put(0, 1); // Implicit unit scale.
            matrix.put(0, 1); // No rotation/skew.
            let width = sf_signed_width(&[translate_x, 0]);
            matrix.put(u32::from(width), 5);
            matrix.signed(translate_x, width);
            matrix.signed(0, width);
            body.extend(matrix.finish());
            body.extend_from_slice(name.as_bytes());
            body.push(0);
            body
        }

        fn sf_rect(values: [i32; 4]) -> Vec<u8> {
            let width = sf_signed_width(&values);
            let mut bits = SfBits::new();
            bits.put(u32::from(width), 5);
            for value in values {
                bits.signed(value, width);
            }
            bits.finish()
        }

        fn sf_signed_width(values: &[i32]) -> u8 {
            (1u8..=31)
                .find(|&width| {
                    let magnitude = 1i64 << (width - 1);
                    values.iter().all(|&value| {
                        i64::from(value) >= -magnitude && i64::from(value) < magnitude
                    })
                })
                .expect("small fixture signed coordinates")
        }

        struct SfBits {
            bytes: Vec<u8>,
            count: usize,
        }
        impl SfBits {
            fn new() -> Self {
                Self {
                    bytes: Vec::new(),
                    count: 0,
                }
            }
            fn put(&mut self, value: u32, width: u8) {
                assert!((1..=31).contains(&width));
                assert!(value < (1u32 << width));
                for shift in (0..width).rev() {
                    if self.count % 8 == 0 {
                        self.bytes.push(0);
                    }
                    let bit = ((value >> shift) & 1) as u8;
                    self.bytes[self.count / 8] |= bit << (7 - self.count % 8);
                    self.count += 1;
                }
            }
            fn signed(&mut self, value: i32, width: u8) {
                let magnitude = 1i64 << (width - 1);
                assert!(i64::from(value) >= -magnitude && i64::from(value) < magnitude);
                self.put((value as u32) & ((1u32 << width) - 1), width);
            }
            fn finish(self) -> Vec<u8> {
                self.bytes
            }
        }
    }
}
