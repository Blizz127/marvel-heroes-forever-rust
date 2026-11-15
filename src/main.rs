use marvel_heroes_forever::{
    BUILD_VERSION, TARGET_BUILD,
    calligraphy::{AssetType, Blueprint, Curve, Directory, FileKind, Header},
    options::{ClientOptions, Preset},
    prototype_catalog::{PrototypeCatalog, PrototypeStatistics},
    sip::SipArchive,
    upk::{
        Package, PackageArchive, Summary,
        objects::{ObjectProperties, supports_asset_class},
    },
};
use std::{error::Error, ffi::OsString, io, path::Path, process::ExitCode};

mod cli_assets;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("mhf: {error}");
            ExitCode::from(2)
        }
    }
}
fn invalid(message: &str) -> Box<dyn Error> {
    io::Error::new(io::ErrorKind::InvalidInput, message).into()
}
fn text(value: &OsString) -> Result<&str> {
    value
        .to_str()
        .ok_or_else(|| invalid("command or member name must be UTF-8"))
}
fn usage() {
    println!("Marvel Heroes Forever {BUILD_VERSION} — private asset tools");
    println!("  mhf status [--preset forever|original]");
    println!("  mhf sip inspect|verify <archive.sip>");
    println!("  mhf sip list <archive.sip> [name-filter]");
    println!("  mhf calligraphy inspect <archive.sip> <member-name>");
    println!("  mhf calligraphy verify <archive.sip>");
    println!("  mhf prototype inspect <archive.sip> <member-name|0xID>");
    println!("  mhf prototype verify <archive.sip>");
    println!("  mhf upk summary|inspect|exports <package.upk>");
    println!("  mhf upk properties <package.upk> <export-name|1-based-index>");
    println!("  mhf upk verify-objects <package.upk>");
    println!("  mhf frontend inspect <package.upk> <movie-name|1-based-index> [dependency.upk ...]");
    println!("  mhf frontend verify <package.upk> [dependency.upk ...]");
    println!("  mhf texture inspect <package.upk> <texture-name|1-based-index>");
    println!("  mhf texture render <package.upk> <texture-name|1-based-index> <private-output.pam>");
    println!("  mhf --version");
    println!("No assets are embedded. No server connection is made.");
}
fn run(args: Vec<OsString>) -> Result<()> {
    if args.is_empty() {
        status(Preset::default());
        return Ok(());
    }
    match text(&args[0])? {
        "--help" | "-h" if args.len() == 1 => usage(),
        "--version" if args.len() == 1 => println!("mhf {BUILD_VERSION}"),
        "frontend" | "texture" => cli_assets::run(&args)?,
        "status" => {
            let preset = match args.as_slice() {
                [_] => Preset::default(),
                [_, flag, value] if flag == "--preset" => text(value)?.parse()?,
                _ => return Err(invalid("usage: mhf status [--preset forever|original]")),
            };
            status(preset);
        }
        "sip" if args.len() >= 3 => {
            let mut archive = SipArchive::open(Path::new(&args[2]))?;
            match text(&args[1])? {
                "inspect" if args.len() == 3 => {
                    let compressed: u64 = archive
                        .entries()
                        .iter()
                        .map(|x| x.compressed_size as u64)
                        .sum();
                    let decoded: u64 = archive
                        .entries()
                        .iter()
                        .map(|x| x.uncompressed_size as u64)
                        .sum();
                    println!(
                        "OBSERVED GPAK v1: {} members, {} file bytes",
                        archive.entries().len(),
                        archive.file_len()
                    );
                    println!(
                        "Payload offset: {}; members: {compressed} compressed bytes, {decoded} decoded bytes",
                        archive.data_offset()
                    );
                    println!("Index/ranges checked; payload decoding not requested.");
                }
                "list" if (3..=4).contains(&args.len()) => {
                    let filter = if args.len() == 4 { text(&args[3])? } else { "" };
                    let matching: Vec<_> = archive
                        .entries()
                        .iter()
                        .filter(|x| x.name.contains(filter))
                        .collect();
                    for entry in matching.iter().take(100) {
                        println!(
                            "{}\t{}\t{}",
                            entry.compressed_size, entry.uncompressed_size, entry.name
                        );
                    }
                    println!(
                        "Shown {} of {} matches; columns: compressed bytes, decoded bytes, name",
                        matching.len().min(100),
                        matching.len()
                    );
                }
                "verify" if args.len() == 3 => {
                    let mut bytes = 0_u64;
                    for index in 0..archive.entries().len() {
                        bytes += archive.read_index(index)?.len() as u64;
                    }
                    println!(
                        "VERIFIED {} raw LZ4 members decoded ({bytes} bytes)",
                        archive.entries().len()
                    );
                    println!(
                        "Bounds and output lengths checked; no source authentication or retail behavior claim."
                    );
                }
                _ => {
                    return Err(invalid(
                        "usage: mhf sip inspect|list|verify <archive.sip> [list-filter]",
                    ));
                }
            }
        }
        "upk" if args.len() == 4 && args[1] == "properties" => {
            let archive = PackageArchive::open(Path::new(&args[2]))?;
            let package = archive.package();
            let selector = text(&args[3])?;
            let index = if let Ok(number) = selector.parse::<usize>() {
                number
                    .checked_sub(1)
                    .ok_or_else(|| invalid("export indices begin at 1"))?
            } else {
                let matches: Vec<_> = package
                    .exports
                    .iter()
                    .enumerate()
                    .filter(|(_, export)| {
                        package
                            .resolve_name(export.object_name)
                            .is_some_and(|name| name.eq_ignore_ascii_case(selector))
                    })
                    .map(|(index, _)| index)
                    .collect();
                match matches.as_slice() {
                    [index] => *index,
                    [] => return Err(invalid("export name not found")),
                    _ => {
                        return Err(invalid(
                            "export name is ambiguous; select its numeric index",
                        ));
                    }
                }
            };
            let bytes = archive.read_export(index)?;
            let object = ObjectProperties::parse_export(package, index, &bytes)?;
            println!(
                "VERIFIED {} property stream: {} tags, {} native payload bytes",
                object.class_name,
                object.properties.entries.len(),
                object.native_payload.length
            );
            for property in object.properties.entries.iter().take(100) {
                let name = package
                    .resolve_name(property.name)
                    .ok_or_else(|| invalid("invalid property name"))?;
                let kind = package
                    .resolve_name(property.property_type)
                    .ok_or_else(|| invalid("invalid property type"))?;
                println!(
                    "{name}[{}] {kind}: {:?}",
                    property.array_index, property.value
                );
            }
            println!(
                "Unknown arrays/structures and native payloads remain opaque; no object execution performed."
            );
        }
        "upk" if args.len() == 3 => {
            let action = text(&args[1])?;
            let path = Path::new(&args[2]);
            if action == "verify-objects" {
                let archive = PackageArchive::open(path)?;
                let package = archive.package();
                let (mut parsed, mut skipped, mut properties) = (0_u64, 0_u64, 0_u64);
                for index in 0..package.exports.len() {
                    let class = package
                        .export_class_name(index)
                        .ok_or_else(|| invalid("unresolved export class"))?;
                    if !supports_asset_class(&class) {
                        skipped += 1;
                        continue;
                    }
                    let bytes = archive.read_export(index)?;
                    let object = ObjectProperties::parse_export(package, index, &bytes)?;
                    parsed += 1;
                    properties += object.properties.entries.len() as u64;
                }
                println!(
                    "VERIFIED {parsed} supported asset property streams, {properties} property tags; skipped {skipped} unsupported class preambles"
                );
                println!("Native payloads and object behavior are not validated by this command.");
            } else if action == "summary" {
                let summary = Summary::open(path)?;
                println!(
                    "OBSERVED UE3 {}/{}: {} names, {} imports, {} exports; compression {}, {} chunks",
                    summary.file_version,
                    summary.licensee_version,
                    summary.names.count,
                    summary.imports.count,
                    summary.exports.count,
                    summary.compression_flags,
                    summary.chunks.len()
                );
                println!("Summary only; table and payload decoding not requested.");
            } else if action == "inspect" || action == "exports" {
                let package = Package::open(path)?;
                println!(
                    "VERIFIED UE3 metadata: {} names, {} imports, {} exports",
                    package.names.len(),
                    package.imports.len(),
                    package.exports.len()
                );
                if action == "exports" {
                    for (index, export) in package.exports.iter().enumerate().take(100) {
                        let name = package
                            .resolve_name(export.object_name)
                            .ok_or_else(|| invalid("invalid export name"))?;
                        println!(
                            "{}\t{}\t{}\t{}",
                            index + 1,
                            export.serial_offset,
                            export.serial_size,
                            name
                        );
                    }
                    println!(
                        "Shown {} of {} exports; columns: object index, logical offset, size, name",
                        package.exports.len().min(100),
                        package.exports.len()
                    );
                }
                println!(
                    "Metadata only; use properties or verify-objects for supported asset property streams."
                );
            } else {
                return Err(invalid(
                    "usage: mhf upk summary|inspect|exports <package.upk>",
                ));
            }
        }
        "prototype" if args.len() >= 3 => {
            let mut catalog = PrototypeCatalog::open(Path::new(&args[2]))?;
            println!(
                "OBSERVED prototype catalogue: {} records, {} blueprints",
                catalog.prototype_count(),
                catalog.blueprint_count()
            );
            let mut reader = catalog.reader()?;
            let statistics = match text(&args[1])? {
                "verify" if args.len() == 3 => reader.verify_all()?,
                "inspect" if args.len() == 4 => {
                    let name = text(&args[3])?;
                    let prototype = if let Some(hex) = name.strip_prefix("0x") {
                        reader.read_id(u64::from_str_radix(hex, 16)?)?
                    } else {
                        reader.read_member(name)?
                    };
                    println!("OBSERVED encoded header: {:?}", prototype.header);
                    let mut statistics = PrototypeStatistics {
                        files: 1,
                        ..Default::default()
                    };
                    statistics.include(&prototype);
                    statistics
                }
                _ => {
                    return Err(invalid(
                        "usage: mhf prototype inspect <archive.sip> <member|0xID> | verify <archive.sip>",
                    ));
                }
            };
            println!("VERIFIED structural decoding: {statistics:?}");
            println!(
                "Parent-value merging, runtime class/property binding and gameplay behavior are not implemented."
            );
        }
        "calligraphy" if args.len() >= 3 => {
            let mut archive = SipArchive::open(Path::new(&args[2]))?;
            match text(&args[1])? {
                "inspect" if args.len() == 4 => {
                    let bytes = archive.read(text(&args[3])?)?;
                    let header = Header::parse(&bytes)?;
                    println!(
                        "OBSERVED {:?}, version {}, {} bytes",
                        header.kind,
                        header.version,
                        bytes.len()
                    );
                    println!("{}", inspect_metadata(&bytes)?);
                }
                "verify" if args.len() == 3 => {
                    let indices: Vec<_> = archive
                        .entries()
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| {
                            [".directory", ".blueprint", ".curve", ".type"]
                                .iter()
                                .any(|s| e.name.ends_with(s))
                        })
                        .map(|(i, _)| i)
                        .collect();
                    for &index in &indices {
                        let bytes = archive.read_index(index)?;
                        inspect_metadata(&bytes).map_err(|error| {
                            invalid(&format!("{}: {error}", archive.entries()[index].name))
                        })?;
                    }
                    println!(
                        "VERIFIED {} Calligraphy directory/blueprint/curve/type files parsed",
                        indices.len()
                    );
                    println!(
                        "Skipped {} other members; use prototype verify for body decoding",
                        archive.entries().len() - indices.len()
                    );
                }
                _ => {
                    return Err(invalid(
                        "usage: mhf calligraphy inspect <archive.sip> <member> | verify <archive.sip>",
                    ));
                }
            }
        }
        _ => return Err(invalid("unknown command or arguments; use mhf --help")),
    }
    Ok(())
}
fn status(preset: Preset) {
    let options = ClientOptions::new(preset);
    println!("Marvel Heroes Forever {BUILD_VERSION}");
    println!("Target retail build: {TARGET_BUILD}");
    println!(
        "Preset: {}; finished upgrades: {}",
        options.preset(),
        options.enabled_upgrades().len()
    );
    println!(
        "Implemented: SIP/LZ4, Calligraphy prototype bodies, UE3/LZO, textures, GFX structure and movie image binding."
    );
    println!("Offscreen image renderer compiled: {}", cfg!(feature = "renderer"));
    println!("Not implemented: playable frontend, login, world or combat.");
    println!("Retail behavior parity is not verified.");
}
fn inspect_metadata(bytes: &[u8]) -> Result<String> {
    Ok(match Header::parse(bytes)?.kind {
        FileKind::CurveDirectory
        | FileKind::TypeDirectory
        | FileKind::BlueprintDirectory
        | FileKind::PrototypeDirectory
        | FileKind::ReplacementDirectory => {
            let value = Directory::parse(bytes)?;
            format!(
                "{:?} directory: {} records",
                value.kind,
                value.records.len()
            )
        }
        FileKind::Curve => {
            let value = Curve::parse(bytes)?;
            format!(
                "Curve: {} values, positions {}..={}",
                value.values.len(),
                value.start,
                value.end
            )
        }
        FileKind::AssetType => format!(
            "Asset type: {} records",
            AssetType::parse(bytes)?.assets.len()
        ),
        FileKind::Blueprint => format!(
            "Blueprint: {} members",
            Blueprint::parse(bytes)?.members.len()
        ),
        FileKind::Prototype => "Header only; prototype body is not decoded".to_owned(),
    })
}
