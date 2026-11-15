//! Offline CLI entry points over original frontend readers. No network access.

use marvel_heroes_forever::{
    frontend::{
        AssetLocation, AssetSet, ImageResolution,
        library::{
            Limits as LibraryLimits, MovieAlias, MovieLibrary, Resolution as SymbolResolution,
        },
        object_path,
    },
    texture::Texture2D,
    upk::{Package, PackageArchive},
};
use std::{error::Error, ffi::OsString, io, path::Path};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn invalid(message: &str) -> Box<dyn Error> {
    io::Error::new(io::ErrorKind::InvalidInput, message).into()
}
fn text(value: &OsString) -> Result<&str> {
    value
        .to_str()
        .ok_or_else(|| invalid("asset selector must be UTF-8"))
}

pub fn run(args: &[OsString]) -> Result<()> {
    match args {
        [command, operation, package, selector, output, extra @ ..]
            if command == "frontend" && operation == "render-static" =>
        {
            render_frontend(package, text(selector)?, Path::new(output), extra, None)?;
        }
        [
            command,
            operation,
            package,
            selector,
            x,
            y,
            output,
            extra @ ..,
        ] if command == "frontend" && operation == "pointer-probe" => {
            let point = [text(x)?.parse::<f64>()?, text(y)?.parse::<f64>()?];
            if point.iter().any(|value| !value.is_finite()) {
                return Err(invalid("pointer coordinates must be finite stage pixels"));
            }
            render_frontend(
                package,
                text(selector)?,
                Path::new(output),
                extra,
                Some(point),
            )?;
        }
        [command, operation, package, selector, extra @ ..]
            if command == "frontend"
                && (operation == "window-static" || operation == "window-pointer") =>
        {
            let mut library_args = Vec::new();
            let mut frames = None;
            let mut seconds = None;
            let mut index = 0;
            while index < extra.len() {
                if extra[index] == "--frames" {
                    if frames.is_some() || index + 1 == extra.len() {
                        return Err(invalid("--frames requires one positive frame count"));
                    }
                    let count = text(&extra[index + 1])?.parse::<u64>()?;
                    if count == 0 {
                        return Err(invalid("frame count must be positive"));
                    }
                    frames = Some(count);
                    index += 2;
                } else if extra[index] == "--seconds" {
                    if seconds.is_some() || index + 1 == extra.len() {
                        return Err(invalid("--seconds requires one positive duration"));
                    }
                    let count = text(&extra[index + 1])?.parse::<u64>()?;
                    if !(1..=86_400).contains(&count) {
                        return Err(invalid("QA duration must be 1..86400 seconds"));
                    }
                    seconds = Some(count);
                    index += 2;
                } else {
                    library_args.push(extra[index].clone());
                    index += 1;
                }
            }
            if operation == "window-pointer" {
                window_pointer(package, text(selector)?, &library_args, frames, seconds)?;
            } else {
                window_frontend(package, text(selector)?, &library_args, frames, seconds)?;
            }
        }
        [command, operation, package, selector]
            if command == "frontend" && operation == "init-probe" =>
        {
            use marvel_heroes_forever::{
                avm1::vm::Limits,
                frontend::{
                    bootstrap::{InitOutcome, InitSession},
                    host::FrontendHost,
                },
            };
            let archive = PackageArchive::open(package)?;
            let export = select(archive.package(), text(selector)?)?;
            let assets = AssetSet::new(vec![archive])?;
            let asset = assets.read_movie(AssetLocation { package: 0, export })?;
            let mut session =
                InitSession::new(asset.movie.header.version, Limits::default(), 2_000_000)?;
            session.install_movie_clip_metadata(u32::MAX)?;
            session.setup_vm()?.install_array_constructor()?;
            session.setup_vm()?.install_property_flags()?;
            let mut host = FrontendHost::install(session.setup_vm()?, asset.movie.header.version)?;
            let plan = session.compile_frame(0, &asset, 0)?;
            println!(
                "OBSERVED {} frame 0: {} saved initializers; {} ordinary frame actions retained separately.",
                plan.object_path,
                plan.initializers.len(),
                plan.frame_actions.len()
            );
            println!(
                "Probe bindings: Object/Array/property-flags intrinsics, MovieClip metadata, Selection listeners and root-context class registry; other host operations rejected."
            );
            let outcome = session.initialize(&plan, &mut host)?;
            println!(
                "OBSERVED Selection: {} retained listeners; {:?}.",
                host.selection().listeners().len(),
                host.selection().stats()
            );
            println!("OBSERVED class registry: {:?}.", host.classes().stats());
            match outcome {
                InitOutcome::Complete {
                    initializers,
                    instructions,
                    host_calls,
                } => {
                    println!(
                        "OBSERVED initializer phase completed: {initializers} actions, {instructions} instructions, {host_calls} host calls."
                    );
                    println!(
                        "Ordinary frame scripts, display construction, imported-movie ordering and native parity were not run."
                    );
                }
                InitOutcome::Stopped(failure) => {
                    println!(
                        "OBSERVED stopped after {} complete initializers and {} instructions: tag {}, source {}, {:?}.",
                        failure.completed_initializers,
                        failure.instructions,
                        failure.action.key.tag,
                        failure.action.source_offset,
                        failure.error
                    );
                    println!("Partial VM state is halted; this is not a completed movie startup.");
                    return Err(invalid(
                        "initializer probe reached an unsupported operation",
                    ));
                }
                InitOutcome::AlreadyInitialized => unreachable!("fresh probe session"),
            }
        }
        [command, operation, package, selector, extra @ ..]
            if command == "frontend" && operation == "scene" =>
        {
            use marvel_heroes_forever::gfx::{
                CharacterKind,
                scene::{Limits, MovieRef, NodeKind, Scene},
            };
            let (assets, library) = load_library(package, extra)?;
            let export = select(assets.archive(0)?.package(), text(selector)?)?;
            let id = library
                .id_for_location(AssetLocation { package: 0, export })
                .ok_or_else(|| invalid("selected export is not a loaded movie"))?;
            let entry = library
                .entry(id)
                .ok_or_else(|| invalid("movie library index missing"))?;
            let scene = Scene::first_frame(
                MovieRef::new(id, &entry.asset.movie),
                &library,
                Limits::default(),
            )?;
            let shapes = scene
                .nodes
                .iter()
                .filter(|node| {
                    node.kind
                        .definition()
                        .is_some_and(|d| d.kind == CharacterKind::Shape)
                })
                .count();
            let unresolved = scene
                .nodes
                .iter()
                .filter(|node| {
                    matches!(
                        node.kind,
                        NodeKind::Import {
                            unresolved: Some(_),
                            ..
                        } | NodeKind::MissingCharacter { .. }
                            | NodeKind::ClassOnly
                    )
                })
                .count();
            let unsupported: usize = scene.nodes.iter().map(|node| node.unsupported.len()).sum();
            println!(
                "OBSERVED {} static first frame: {} nodes, {shapes} shape instances, {unresolved} unresolved nodes, {unsupported} unsupported node conditions",
                entry.asset.object_path,
                scene.nodes.len()
            );
            println!(
                "Unexecuted script bindings: {}; unbound action tags: {}; retained opaque records: {}",
                scene.actions.len(),
                scene.unbound_action_tags.len(),
                scene.opaque_nodes.len()
            );
            for node in scene.nodes.iter().take(32) {
                println!(
                    "node {} parent {:?} name {:?} visible {} kind {:?}",
                    node.id,
                    node.parent,
                    node.segment.as_ref().and_then(|s| s.name.as_deref()),
                    node.effective_visible,
                    node.kind
                );
            }
            println!(
                "Serialized first-frame snapshot only; nested sprites start at frame zero and scripts have not changed the scene."
            );
        }
        [command, operation, package, extra @ ..]
            if command == "frontend" && operation == "imports" =>
        {
            let (_assets, library) = load_library(package, extra)?;
            let mut total = 0;
            let mut resolved = 0;
            for (id, entry) in library.entries().iter().enumerate() {
                let bindings: Vec<_> = library
                    .bindings()
                    .filter(|b| b.source_movie == id)
                    .collect();
                let found = bindings
                    .iter()
                    .filter(|b| matches!(b.resolution, SymbolResolution::Resolved { .. }))
                    .count();
                println!(
                    "OBSERVED {}: {found}/{} imported symbols resolved",
                    entry.asset.object_path,
                    bindings.len()
                );
                total += bindings.len();
                resolved += found;
                for binding in bindings
                    .into_iter()
                    .filter(|b| !matches!(b.resolution, SymbolResolution::Resolved { .. }))
                    .take(8)
                {
                    println!(
                        "  {} [{}] from {}: {:?}",
                        binding.symbol, binding.local_character_id, binding.url, binding.resolution
                    );
                }
            }
            println!(
                "VERIFIED {} saved movie structures; resolved {resolved}/{total} imported symbols in the supplied library.",
                library.entries().len()
            );
            println!(
                "Explicit font routes are caller selections; native import behavior and runtime execution remain unverified."
            );
        }
        [command, operation, package, selector, dependencies @ ..]
            if command == "frontend" && operation == "inspect" =>
        {
            let mut archives = vec![PackageArchive::open(package)?];
            for dependency in dependencies {
                archives.push(PackageArchive::open(dependency)?);
            }
            let index = select(archives[0].package(), text(selector)?)?;
            let assets = AssetSet::new(archives)?;
            inspect_movie(&assets, index, true)?;
        }
        [command, operation, package, dependencies @ ..]
            if command == "frontend" && operation == "verify" =>
        {
            let mut archives = vec![PackageArchive::open(package)?];
            for dependency in dependencies {
                archives.push(PackageArchive::open(dependency)?);
            }
            let indices: Vec<_> = (0..archives[0].package().exports.len())
                .filter(|&index| {
                    archives[0]
                        .package()
                        .export_class_name(index)
                        .is_some_and(|name| name.eq_ignore_ascii_case("SwfMovie"))
                })
                .collect();
            let assets = AssetSet::new(archives)?;
            let mut images = 0;
            let mut resolved = 0;
            for &index in &indices {
                let (count, found) = inspect_movie(&assets, index, false)?;
                images += count;
                resolved += found;
            }
            println!(
                "VERIFIED structural decoding of {} SwfMovie exports; resolved {resolved}/{images} image records against supplied packages.",
                indices.len()
            );
            println!(
                "Unresolved records: {}. Scripts, font aliases and live render targets were not executed.",
                images - resolved
            );
        }
        [command, operation, package, selector]
            if command == "texture" && operation == "inspect" =>
        {
            let texture = load_texture(package, text(selector)?)?;
            println!("OBSERVED Texture2D: {:?}", texture.descriptor);
            for (index, mip) in texture.mips.iter().enumerate() {
                println!(
                    "mip {index}: {}x{}, flags {:#x}, {} decoded bytes, stored size {}, offset {}",
                    mip.width,
                    mip.height,
                    mip.bulk.flags,
                    mip.bulk.element_count,
                    mip.bulk.stored_size,
                    mip.bulk.stored_offset
                );
            }
        }
        [command, operation, package, selector, output]
            if command == "texture" && operation == "render" =>
        {
            render_texture(package, text(selector)?, Path::new(output))?;
        }
        [command, operation, package, selector, extra @ ..]
            if command == "texture" && operation == "window" =>
        {
            let frames = match extra {
                [] => None,
                [flag, count] if flag == "--frames" => {
                    let count = text(count)?.parse::<u64>()?;
                    if !(1..=1_000_000).contains(&count) {
                        return Err(invalid("preview frame count must be 1..1000000"));
                    }
                    Some(count)
                }
                _ => {
                    return Err(invalid(
                        "usage: mhf texture window <package> <texture> [--frames N]",
                    ));
                }
            };
            window_texture(package, text(selector)?, frames)?;
        }
        _ => return Err(invalid("invalid asset command; use mhf --help")),
    }
    Ok(())
}

fn load_library(package: &OsString, extra: &[OsString]) -> Result<(AssetSet, MovieLibrary)> {
    let mut archives = vec![PackageArchive::open(package)?];
    let mut aliases = Vec::new();
    let mut index = 0;
    while index < extra.len() {
        if extra[index] == "--font-library" {
            if !aliases.is_empty() || index + 1 >= extra.len() {
                return Err(invalid(
                    "--font-library needs exactly one selected movie object path",
                ));
            }
            aliases.push(MovieAlias {
                logical_object_path: "MarvelFonts.gfxfontlib".into(),
                target_object_path: text(&extra[index + 1])?.to_owned(),
            });
            index += 2;
        } else {
            if extra[index]
                .to_str()
                .is_some_and(|value| value.starts_with("--"))
            {
                return Err(invalid("unknown movie library option"));
            }
            archives.push(PackageArchive::open(&extra[index])?);
            index += 1;
        }
    }
    let indices: Vec<_> = (0..archives.len()).collect();
    let assets = AssetSet::new(archives)?;
    let library = MovieLibrary::load(&assets, &indices, &aliases, LibraryLimits::default())?;
    Ok((assets, library))
}

#[cfg(feature = "renderer")]
fn frontend_frame(
    package: &OsString,
    selector: &str,
    extra: &[OsString],
    pointer: Option<[f64; 2]>,
) -> Result<marvel_heroes_forever::render::Readback> {
    use marvel_heroes_forever::{
        frontend::{
            preview::{self, Background, BitmapMapping, Effects, PreviewPolicy},
            scene_content,
        },
        gfx::scene::{Limits, MovieRef, Scene},
        render::{
            ImageEncoding, QuadRenderer,
            mesh::{GradientAlpha, LinearRgbTransfer, TransformSpace},
        },
    };
    let (assets, library) = load_library(package, extra)?;
    let export = select(assets.archive(0)?.package(), selector)?;
    let id = library
        .id_for_location(AssetLocation { package: 0, export })
        .ok_or_else(|| invalid("selected export is not a loaded movie"))?;
    let entry = library
        .entry(id)
        .ok_or_else(|| invalid("movie library index missing"))?;
    let scene = Scene::first_frame(
        MovieRef::new(id, &entry.asset.movie),
        &library,
        Limits::default(),
    )?;
    let mut prepared =
        scene_content::prepare(&assets, &library, &scene, scene_content::Limits::default())?;
    if let Some(pointer) = pointer {
        apply_pointer_probe(&scene, &mut prepared, pointer)?;
    }
    let renderer = pollster::block_on(QuadRenderer::headless(
        wgpu::TextureFormat::Rgba8Unorm,
        false,
    ))?;
    let policy = PreviewPolicy {
        background: Background::SavedRootTagOrTransparent,
        colors: ImageEncoding::Linear,
        color_transform_space: TransformSpace::Shader,
        linear_rgb_transfer: LinearRgbTransfer::Srgb,
        gradient_alpha: GradientAlpha::Straight,
        bitmap_mapping: BitmapMapping::WholeImageReplacement,
        unknown_texture_encoding: None,
        effects: Effects::BaseGeometry,
    };
    let frame =
        preview::render_static(&renderer, &prepared, entry.asset.movie.header.stage, policy)?;
    println!(
        "OBSERVED {} geometry diagnostic: {}x{}, {} submitted fills, {} submitted triangles, {} uploaded textures.",
        entry.asset.object_path,
        frame.readback.width,
        frame.readback.height,
        frame.coverage.fill_draws,
        frame.coverage.triangles,
        frame.coverage.uploaded_textures
    );
    println!(
        "OBSERVED adapter: {:?}; explicit diagnostic policy: {:?}",
        renderer.adapter_info(),
        policy
    );
    println!(
        "Limitations: {}; text instances omitted: {}; script binding records: {}; unbound action tags: {}.",
        frame.limitations.len(),
        frame.coverage.text_instances_omitted,
        frame.coverage.source_action_bindings,
        frame.coverage.unbound_action_tags
    );
    println!(
        "Saved text: {}/{} fields prepared, {} glyph fills submitted, {} field clips; runtime text updates have not run.",
        frame.coverage.text_instances_prepared,
        frame.coverage.text_instances,
        frame.coverage.text_fill_draws,
        frame.coverage.text_fields_clipped
    );
    println!(
        "Saved background: {:?}. Filters, scale9, non-normal blends and runtime visibility remain unverified; this is not a playable frontend.",
        frame.saved_background_applied
    );
    Ok(frame.readback)
}

/// A single explicit original clip-handler invocation. This is not an event
/// scheduler, class constructor or initialized movie player.
#[cfg(feature = "renderer")]
fn apply_pointer_probe(
    scene: &marvel_heroes_forever::gfx::scene::Scene<'_>,
    prepared: &mut marvel_heroes_forever::frontend::scene_content::PreparedScene,
    pointer: [f64; 2],
) -> Result<()> {
    use marvel_heroes_forever::frontend::pointer::{Limits, PointerController};
    let mut controller = PointerController::from_scene(scene, Limits::default())?;
    controller.pointer_stage_twips(Some([pointer[0] * 20.0, pointer[1] * 20.0]))?;
    let nodes = controller.nodes();
    if nodes.len() != prepared.nodes.len() {
        return Err(invalid("display snapshot node count changed"));
    }
    for (prepared_node, current) in prepared.nodes.iter_mut().zip(nodes) {
        if prepared_node.instance.id != current.id
            || prepared_node.instance.raw != current.raw
            || prepared_node.instance.parent != current.parent
        {
            return Err(invalid("display snapshot changed source identity"));
        }
        prepared_node.instance = current.clone();
    }
    let action = controller.binding();
    let stats = controller.stats();
    println!(
        "OBSERVED one saved MouseMove invocation: node {}, source {}+{}, pointer {:?} stage pixels, {} instructions, {} host calls.",
        action.owner,
        action.bytes.offset,
        action.bytes.len,
        pointer,
        stats.completed_instructions,
        stats.completed_host_calls
    );
    println!(
        "OBSERVED geometry host: {:?}; resulting local transform {:?}.",
        controller.display().stats(),
        controller.display().node(action.owner)?.local_matrix
    );
    println!(
        "The probe supplied one instance binding and pointer. Initializers, constructors, event ordering and normal startup were not run; renderer script counts describe retained source bindings."
    );
    Ok(())
}

#[cfg(feature = "renderer")]
fn render_frontend(
    package: &OsString,
    selector: &str,
    output: &Path,
    extra: &[OsString],
    pointer: Option<[f64; 2]>,
) -> Result<()> {
    let destination = private_output(output)?;
    let frame = frontend_frame(package, selector, extra, pointer)?;
    write_private_pam(&destination, &frame)?;
    println!("OBSERVED private scene PAM: {}", destination.display());
    Ok(())
}
#[cfg(not(feature = "renderer"))]
fn render_frontend(
    _: &OsString,
    _: &str,
    _: &Path,
    _: &[OsString],
    _: Option<[f64; 2]>,
) -> Result<()> {
    Err(invalid(
        "this build lacks renderer support; build with --features renderer",
    ))
}

#[cfg(feature = "window")]
fn window_frontend(
    package: &OsString,
    selector: &str,
    extra: &[OsString],
    frames: Option<u64>,
    seconds: Option<u64>,
) -> Result<()> {
    use marvel_heroes_forever::window::{PreviewImage, PreviewOptions, run_image_preview};
    let frame = frontend_frame(package, selector, extra, None)?;
    let mut options = PreviewOptions::new(frame.encoding);
    options.title = format!("Marvel Heroes Forever — {selector} (static diagnostic)");
    options.frame_limit = frames;
    options.close_after = seconds
        .or_else(|| frames.map(|_| 15))
        .map(std::time::Duration::from_secs);
    let report = run_image_preview(
        PreviewImage {
            width: frame.width,
            height: frame.height,
            pixels: frame.pixels,
            encoding: frame.encoding,
        },
        options,
    )?;
    println!("OBSERVED native static-scene presentation: {report:?}");
    if frames.is_some_and(|expected| report.frames_presented != expected) {
        return Err(invalid("preview closed before the requested frame count"));
    }
    Ok(())
}
#[cfg(not(feature = "window"))]
fn window_frontend(
    _: &OsString,
    _: &str,
    _: &[OsString],
    _: Option<u64>,
    _: Option<u64>,
) -> Result<()> {
    Err(invalid(
        "this build lacks native window support; build with --features window",
    ))
}

#[cfg(feature = "window")]
fn window_pointer(
    package: &OsString,
    selector: &str,
    extra: &[OsString],
    frames: Option<u64>,
    seconds: Option<u64>,
) -> Result<()> {
    use marvel_heroes_forever::{
        frontend::{
            pointer::{Limits as PointerLimits, PointerController},
            preview::{Background, BitmapMapping, Effects, PreviewPolicy},
            scene_content,
        },
        gfx::scene::{Limits, MovieRef, Scene, SceneNode},
        render::{
            ImageEncoding,
            mesh::{GradientAlpha, LinearRgbTransfer, TransformSpace},
        },
        window::{SceneController, ScenePreviewOptions, run_scene_preview},
    };
    struct Controller(PointerController);
    impl SceneController for Controller {
        fn nodes(&self) -> &[SceneNode] {
            self.0.nodes()
        }
        fn pointer_stage_twips(&mut self, point: Option<[f64; 2]>) -> io::Result<()> {
            self.0.pointer_stage_twips(point).map(|_| ())
        }
    }
    let (assets, library) = load_library(package, extra)?;
    let export = select(assets.archive(0)?.package(), selector)?;
    let id = library
        .id_for_location(AssetLocation { package: 0, export })
        .ok_or_else(|| invalid("selected export is not a loaded movie"))?;
    let entry = library
        .entry(id)
        .ok_or_else(|| invalid("movie library index missing"))?;
    let scene = Scene::first_frame(
        MovieRef::new(id, &entry.asset.movie),
        &library,
        Limits::default(),
    )?;
    let prepared =
        scene_content::prepare(&assets, &library, &scene, scene_content::Limits::default())?;
    let mut controller = Controller(PointerController::from_scene(
        &scene,
        PointerLimits::default(),
    )?);
    println!(
        "OBSERVED persistent pointer binding: {:?}.",
        controller.0.binding()
    );
    println!(
        "This diagnostic runs one saved MouseMove handler on native pointer movement. Initializers, constructors, normal startup and other events have not run."
    );
    let policy = PreviewPolicy {
        background: Background::SavedRootTagOrTransparent,
        colors: ImageEncoding::Linear,
        color_transform_space: TransformSpace::Shader,
        linear_rgb_transfer: LinearRgbTransfer::Srgb,
        gradient_alpha: GradientAlpha::Straight,
        bitmap_mapping: BitmapMapping::WholeImageReplacement,
        unknown_texture_encoding: None,
        effects: Effects::BaseGeometry,
    };
    let mut options = ScenePreviewOptions::new(ImageEncoding::Linear);
    options.title = format!("Marvel Heroes Forever — {selector} (pointer diagnostic)");
    options.frame_limit = frames;
    options.close_after = seconds
        .or_else(|| frames.map(|_| 15))
        .map(std::time::Duration::from_secs);
    let result = run_scene_preview(
        prepared,
        entry.asset.movie.header.stage,
        policy,
        &mut controller,
        options,
    );
    println!(
        "OBSERVED saved pointer execution: {:?}; halted: {}; display: {:?}; cursor transform: {:?}.",
        controller.0.stats(),
        controller.0.halted(),
        controller.0.display().stats(),
        controller
            .0
            .display()
            .node(controller.0.binding().owner)?
            .local_matrix
    );
    let report = result?;
    println!("OBSERVED native persistent-scene presentation: {report:?}");
    if frames.is_some_and(|expected| report.window.frames_presented != expected) {
        return Err(invalid(
            "scene preview closed before the requested frame count",
        ));
    }
    Ok(())
}
#[cfg(not(feature = "window"))]
fn window_pointer(
    _: &OsString,
    _: &str,
    _: &[OsString],
    _: Option<u64>,
    _: Option<u64>,
) -> Result<()> {
    Err(invalid(
        "this build lacks native window support; build with --features window",
    ))
}

#[cfg(not(feature = "window"))]
fn window_texture(_package: &OsString, _selector: &str, _frames: Option<u64>) -> Result<()> {
    Err(invalid(
        "this build lacks native window support; build with --features window",
    ))
}

#[cfg(feature = "window")]
fn window_texture(package: &OsString, selector: &str, frames: Option<u64>) -> Result<()> {
    use marvel_heroes_forever::{
        render::ImageEncoding,
        window::{PreviewImage, PreviewOptions, run_image_preview},
    };
    let texture = load_texture(package, selector)?;
    let decoded = texture.decode_mip(0)?;
    println!(
        "OBSERVED Texture2D SRGB metadata: {:?}. Preview uses explicit Linear input/output for raw-channel inspection.",
        texture.descriptor.srgb
    );
    let image = PreviewImage {
        width: decoded.width,
        height: decoded.height,
        pixels: decoded.pixels,
        encoding: ImageEncoding::Linear,
    };
    let mut options = PreviewOptions::new(ImageEncoding::Linear);
    options.title = format!("Marvel Heroes Forever — {selector} (saved texture)");
    options.frame_limit = frames;
    if frames.is_some() {
        options.close_after = Some(std::time::Duration::from_secs(15));
    }
    let report = run_image_preview(image, options)?;
    println!("OBSERVED native image preview: {report:?}");
    println!("Saved texture presentation; retail scene and display parity remain unverified.");
    if frames.is_some_and(|expected| report.frames_presented != expected) {
        return Err(invalid("preview closed before the requested frame count"));
    }
    Ok(())
}

fn select(package: &Package, selector: &str) -> Result<usize> {
    if let Ok(number) = selector.parse::<usize>() {
        let index = number
            .checked_sub(1)
            .ok_or_else(|| invalid("export indices begin at 1"))?;
        if index < package.exports.len() {
            return Ok(index);
        }
        return Err(invalid("export index out of bounds"));
    }
    let mut found = None;
    for (index, export) in package.exports.iter().enumerate() {
        let matches = if selector.contains('.') {
            object_path(package, i32::try_from(index + 1)?)?.eq_ignore_ascii_case(selector)
        } else {
            package
                .resolve_name(export.object_name)
                .is_some_and(|name| name.eq_ignore_ascii_case(selector))
        };
        if matches {
            if found.is_some() {
                return Err(invalid(
                    "ambiguous export name; use full object path or numeric index",
                ));
            }
            found = Some(index);
        }
    }
    found.ok_or_else(|| invalid("export name not found"))
}

fn inspect_movie(assets: &AssetSet, index: usize, verbose: bool) -> Result<(usize, usize)> {
    let asset = assets.read_movie(AssetLocation {
        package: 0,
        export: index,
    })?;
    let bindings = assets.bind_images(&asset)?;
    let resolved = bindings
        .iter()
        .filter(|binding| matches!(binding.resolution, ImageResolution::Resolved { .. }))
        .count();
    println!(
        "OBSERVED {}: stage {}x{}, {} tags, {} dictionary entries, {} timelines, images {resolved}/{} resolved",
        asset.object_path,
        asset.movie.header.stage.width_pixels(),
        asset.movie.header.stage.height_pixels(),
        asset.movie.tags.len(),
        asset.movie.dictionary.len(),
        asset.movie.timelines.len(),
        bindings.len()
    );
    if verbose {
        for binding in bindings.iter().take(100) {
            println!(
                "image {} {}x{} {:?}: {:?}",
                binding.image.id,
                binding.image.width,
                binding.image.height,
                binding.basis,
                binding.resolution
            );
        }
        if bindings.len() > 100 {
            println!("Shown first 100 image records.");
        }
        println!(
            "Serialized dependencies only; no ActionScript or live frontend controller execution."
        );
    }
    Ok((bindings.len(), resolved))
}

fn load_texture(package: &OsString, selector: &str) -> Result<Texture2D> {
    let archive = PackageArchive::open(package)?;
    let index = select(archive.package(), selector)?;
    let bytes = archive.read_export(index)?;
    Ok(Texture2D::from_export(archive.package(), index, &bytes)?)
}

#[cfg(not(feature = "renderer"))]
fn render_texture(_package: &OsString, _selector: &str, _output: &Path) -> Result<()> {
    Err(invalid(
        "this build lacks renderer support; build with --features renderer",
    ))
}

#[cfg(feature = "renderer")]
fn private_output(output: &Path) -> Result<std::path::PathBuf> {
    // Extracted pixels belong only in the private asset root. Existing outputs
    // are never overwritten, and movie identifiers are never filesystem paths.
    let root = std::env::var_os("HOME").ok_or_else(|| invalid("HOME is unavailable"))?;
    let private = Path::new(&root)
        .join(".local/share/marvel-heroes-forever-rust")
        .canonicalize()?;
    let parent = output
        .parent()
        .ok_or_else(|| invalid("output must have a private parent directory"))?
        .canonicalize()?;
    if !parent.starts_with(&private) {
        return Err(invalid(
            "asset outputs must stay under ~/.local/share/marvel-heroes-forever-rust/",
        ));
    }
    let name = output
        .file_name()
        .ok_or_else(|| invalid("output requires a file name"))?;
    let destination = parent.join(name);
    if destination.exists() {
        return Err(invalid("output already exists"));
    }
    Ok(destination)
}

#[cfg(feature = "renderer")]
fn write_private_pam(
    destination: &Path,
    image: &marvel_heroes_forever::render::Readback,
) -> Result<()> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(destination)?;
    write!(
        file,
        "P7\nWIDTH {}\nHEIGHT {}\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n",
        image.width, image.height
    )?;
    file.write_all(&image.pixels)?;
    Ok(())
}

#[cfg(feature = "renderer")]
fn render_texture(package: &OsString, selector: &str, output: &Path) -> Result<()> {
    use marvel_heroes_forever::render::{
        BlendMode, ImageEncoding, ImageFilter, Quad, QuadRenderer, RgbaImage,
    };
    let destination = private_output(output)?;
    let texture = load_texture(package, selector)?;
    let image = texture.decode_mip(0)?;
    // This is a raw-byte inspection path. Linear sampling/target and replacement
    // preserve channels, without claiming the original display transfer curve.
    let renderer = pollster::block_on(QuadRenderer::headless(
        wgpu::TextureFormat::Rgba8Unorm,
        false,
    ))?;
    println!("OBSERVED adapter: {:?}", renderer.adapter_info());
    let uploaded = renderer.upload_rgba(
        RgbaImage {
            width: image.width,
            height: image.height,
            pixels: &image.pixels,
        },
        ImageEncoding::Linear,
        ImageFilter::Nearest,
    )?;
    let mut quad = Quad::rectangle(
        &uploaded,
        [0., 0.],
        [image.width as f32, image.height as f32],
    );
    quad.blend = BlendMode::Replace;
    let readback = renderer.offscreen_rgba(image.width, image.height, [0.; 4], &[quad])?;
    if readback.pixels != image.pixels {
        return Err(invalid("GPU readback differs from decoded input pixels"));
    }
    write_private_pam(&destination, &readback)?;
    println!(
        "VERIFIED GPU readback equals {} decoded RGBA bytes; private PAM written: {}",
        image.pixels.len(),
        destination.display()
    );
    println!("Single saved texture inspection; not a composed or playable frontend.");
    Ok(())
}
