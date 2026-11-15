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
fn render_texture(package: &OsString, selector: &str, output: &Path) -> Result<()> {
    use marvel_heroes_forever::render::{
        BlendMode, ImageEncoding, ImageFilter, Quad, QuadRenderer, RgbaImage,
    };
    use std::io::Write;
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
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&destination)?;
    write!(
        file,
        "P7\nWIDTH {}\nHEIGHT {}\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n",
        image.width, image.height
    )?;
    file.write_all(&readback.pixels)?;
    println!(
        "VERIFIED GPU readback equals {} decoded RGBA bytes; private PAM written: {}",
        image.pixels.len(),
        destination.display()
    );
    println!("Single saved texture inspection; not a composed or playable frontend.");
    Ok(())
}
