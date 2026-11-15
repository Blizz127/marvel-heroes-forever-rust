use marvel_heroes_forever::{
    BUILD_VERSION, TARGET_BUILD,
    calligraphy::{AssetType, Blueprint, Curve, Directory, FileKind, Header},
    options::{ClientOptions, Preset},
    sip::SipArchive,
    upk::{Package, Summary},
};
use std::{error::Error, ffi::OsString, io, path::Path, process::ExitCode};

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
    println!("  mhf upk summary|inspect|exports <package.upk>");
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
        "upk" if args.len() == 3 => {
            let action = text(&args[1])?;
            let path = Path::new(&args[2]);
            if action == "summary" {
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
                println!("Object payloads and rendering are not implemented.");
            } else {
                return Err(invalid(
                    "usage: mhf upk summary|inspect|exports <package.upk>",
                ));
            }
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
                        "Skipped {} other members; prototype bodies/inheritance remain unimplemented",
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
    println!("Implemented: SIP/LZ4, Calligraphy metadata, UE3/LZO object-table readers.");
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
