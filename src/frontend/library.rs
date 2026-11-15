//! Saved movie import/export linkage. No URL is opened and no script executes.

use super::{AssetLocation, AssetSet, MovieAsset, invalid, object_class, object_path};
use crate::gfx::TagData;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

pub type MovieId = usize;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_movies: usize,
    pub max_serialized_bytes: usize,
    pub max_symbols: usize,
    pub max_aliases: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_movies: 256,
            max_serialized_bytes: 256 * 1024 * 1024,
            max_symbols: 250_000,
            max_aliases: 256,
        }
    }
}

/// An explicit application-level route, separate from saved References.
/// The caller is responsible for choosing the correct locale/native policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovieAlias {
    pub logical_object_path: String,
    pub target_object_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportOrigin {
    SavedReference,
    ExplicitAlias {
        logical_object_path: String,
        target_object_path: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution {
    Resolved {
        movie: MovieId,
        character_id: u16,
        origin: ImportOrigin,
    },
    UnsupportedLogicalUrl,
    MissingReferenceProperty,
    MissingNamedReference {
        object_path: String,
    },
    AmbiguousReferences {
        object_path: String,
        count: usize,
    },
    MissingMovie {
        object_path: String,
    },
    AmbiguousMovies {
        object_path: String,
        candidates: Vec<MovieId>,
    },
    MissingSymbol {
        movie: MovieId,
    },
    AmbiguousSymbol {
        movie: MovieId,
        candidates: Vec<u16>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolBinding {
    pub source_movie: MovieId,
    pub local_character_id: u16,
    pub url: String,
    pub symbol: String,
    pub resolution: Resolution,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Reference {
    Local(AssetLocation),
    Imported(String),
}

pub struct Entry {
    pub asset: MovieAsset,
    /// Symbol identifiers retain case. Only UE3 object paths are ASCII-folded.
    pub exports: BTreeMap<String, BTreeSet<u16>>,
    references: Option<BTreeMap<String, BTreeSet<Reference>>>,
}

pub struct MovieLibrary {
    entries: Vec<Entry>,
    by_location: BTreeMap<AssetLocation, MovieId>,
    by_path: BTreeMap<String, Vec<MovieId>>,
    bindings: BTreeMap<(MovieId, u16), SymbolBinding>,
}

impl crate::gfx::scene::ImportResolver for MovieLibrary {
    fn resolve(
        &self,
        request: crate::gfx::scene::ImportRequest<'_>,
    ) -> io::Result<Option<crate::gfx::scene::ResolvedCharacter<'_>>> {
        let binding = self
            .binding(request.from_movie, request.local_character_id)
            .ok_or_else(|| invalid("scene requested an undeclared imported character"))?;
        if binding.url != request.url || binding.symbol != request.symbol {
            return Err(invalid("scene import request disagrees with saved movie"));
        }
        let Resolution::Resolved {
            movie,
            character_id,
            ..
        } = binding.resolution
        else {
            return Ok(None);
        };
        let entry = self
            .entry(movie)
            .ok_or_else(|| invalid("resolved movie index is invalid"))?;
        Ok(Some(crate::gfx::scene::ResolvedCharacter {
            movie: crate::gfx::scene::MovieRef::new(movie, &entry.asset.movie),
            character_id,
        }))
    }
}

impl MovieLibrary {
    /// Load SwfMovie exports only from the explicitly selected archive indices.
    /// Duplicate paths are retained; exact positive references retain identity.
    pub fn load(
        assets: &AssetSet,
        packages: &[usize],
        aliases: &[MovieAlias],
        limits: Limits,
    ) -> io::Result<Self> {
        if aliases.len() > limits.max_aliases {
            return Err(invalid("movie alias budget exceeded"));
        }
        let mut routes = BTreeMap::new();
        for alias in aliases {
            let source = canonical_path(&alias.logical_object_path)?;
            let target = canonical_path(&alias.target_object_path)?;
            if routes.insert(source, target).is_some() {
                return Err(invalid("duplicate logical movie alias"));
            }
        }
        let mut library = Self {
            entries: Vec::new(),
            by_location: BTreeMap::new(),
            by_path: BTreeMap::new(),
            bindings: BTreeMap::new(),
        };
        let mut serialized_bytes = 0usize;
        let mut symbol_count = 0usize;
        let mut seen = BTreeSet::new();
        for &package_index in packages {
            if !seen.insert(package_index) {
                return Err(invalid("duplicate selected movie archive"));
            }
            let archive = assets.archive(package_index)?;
            let package = archive.package();
            for (export_index, export) in package.exports.iter().enumerate() {
                if !package
                    .export_class_name(export_index)
                    .is_some_and(|s| s.eq_ignore_ascii_case("SwfMovie"))
                {
                    continue;
                }
                if library.entries.len() >= limits.max_movies {
                    return Err(invalid("movie count budget exceeded"));
                }
                serialized_bytes = serialized_bytes
                    .checked_add(export.serial_size)
                    .ok_or_else(|| invalid("movie byte count overflow"))?;
                if serialized_bytes > limits.max_serialized_bytes {
                    return Err(invalid("movie byte budget exceeded"));
                }
                let location = AssetLocation {
                    package: package_index,
                    export: export_index,
                };
                let asset = assets.read_movie(location)?;
                let references = asset
                    .references
                    .as_ref()
                    .map(|refs| {
                        let mut result: BTreeMap<String, BTreeSet<Reference>> = BTreeMap::new();
                        for &reference in refs {
                            if reference == 0
                                || !object_class(package, reference)?
                                    .eq_ignore_ascii_case("SwfMovie")
                            {
                                continue;
                            }
                            let path = canonical_path(&object_path(package, reference)?)?;
                            let value = if reference > 0 {
                                Reference::Local(AssetLocation {
                                    package: package_index,
                                    export: reference as usize - 1,
                                })
                            } else {
                                Reference::Imported(path.clone())
                            };
                            result.entry(path).or_default().insert(value);
                        }
                        Ok::<_, io::Error>(result)
                    })
                    .transpose()?;
                let mut exports: BTreeMap<String, BTreeSet<u16>> = BTreeMap::new();
                for tag in &asset.movie.tags {
                    match &tag.data {
                        TagData::ExportAssets(symbols) => {
                            symbol_count = symbol_count
                                .checked_add(symbols.len())
                                .ok_or_else(|| invalid("symbol count overflow"))?;
                            if symbol_count > limits.max_symbols {
                                return Err(invalid("movie symbol budget exceeded"));
                            }
                            for symbol in symbols {
                                if !asset.movie.dictionary.contains_key(&symbol.id) {
                                    return Err(invalid(
                                        "exported symbol has no dictionary character",
                                    ));
                                }
                                exports
                                    .entry(symbol.name.clone())
                                    .or_default()
                                    .insert(symbol.id);
                            }
                        }
                        TagData::ImportAssets { symbols, .. } => {
                            symbol_count = symbol_count
                                .checked_add(symbols.len())
                                .ok_or_else(|| invalid("symbol count overflow"))?;
                            if symbol_count > limits.max_symbols {
                                return Err(invalid("movie symbol budget exceeded"));
                            }
                        }
                        _ => {}
                    }
                }
                let id = library.entries.len();
                library.by_location.insert(location, id);
                library
                    .by_path
                    .entry(canonical_path(&asset.object_path)?)
                    .or_default()
                    .push(id);
                library.entries.push(Entry {
                    asset,
                    exports,
                    references,
                });
            }
        }
        for (movie_id, entry) in library.entries.iter().enumerate() {
            for tag in &entry.asset.movie.tags {
                if let TagData::ImportAssets { url, symbols } = &tag.data {
                    for symbol in symbols {
                        let resolution =
                            library.resolve_symbol(movie_id, url, &symbol.name, &routes)?;
                        let binding = SymbolBinding {
                            source_movie: movie_id,
                            local_character_id: symbol.id,
                            url: url.clone(),
                            symbol: symbol.name.clone(),
                            resolution,
                        };
                        if library
                            .bindings
                            .insert((movie_id, symbol.id), binding)
                            .is_some()
                        {
                            return Err(invalid("duplicate imported dictionary identity"));
                        }
                    }
                }
            }
        }
        Ok(library)
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    pub fn entry(&self, id: MovieId) -> Option<&Entry> {
        self.entries.get(id)
    }
    pub fn id_for_location(&self, location: AssetLocation) -> Option<MovieId> {
        self.by_location.get(&location).copied()
    }
    pub fn ids_for_path(&self, path: &str) -> io::Result<&[MovieId]> {
        Ok(self
            .by_path
            .get(&canonical_path(path)?)
            .map(Vec::as_slice)
            .unwrap_or(&[]))
    }
    pub fn bindings(&self) -> impl Iterator<Item = &SymbolBinding> {
        self.bindings.values()
    }
    pub fn binding(&self, movie: MovieId, id: u16) -> Option<&SymbolBinding> {
        self.bindings.get(&(movie, id))
    }

    fn resolve_symbol(
        &self,
        from: MovieId,
        url: &str,
        symbol: &str,
        aliases: &BTreeMap<String, String>,
    ) -> io::Result<Resolution> {
        let entry = &self.entries[from];
        let Ok(logical) = logical_movie_path(&entry.asset.object_path, url) else {
            return Ok(Resolution::UnsupportedLogicalUrl);
        };
        let (candidates, target, origin) = if let Some(target) = aliases.get(&logical) {
            (
                self.by_path.get(target).cloned().unwrap_or_default(),
                target.clone(),
                ImportOrigin::ExplicitAlias {
                    logical_object_path: logical,
                    target_object_path: target.clone(),
                },
            )
        } else {
            let Some(references) = &entry.references else {
                return Ok(Resolution::MissingReferenceProperty);
            };
            let Some(references) = references.get(&logical) else {
                return Ok(Resolution::MissingNamedReference {
                    object_path: logical,
                });
            };
            if references.len() != 1 {
                return Ok(Resolution::AmbiguousReferences {
                    object_path: logical,
                    count: references.len(),
                });
            }
            let candidates = match references.first().unwrap() {
                Reference::Local(location) => self
                    .by_location
                    .get(location)
                    .copied()
                    .into_iter()
                    .collect(),
                Reference::Imported(path) => self.by_path.get(path).cloned().unwrap_or_default(),
            };
            (candidates, logical, ImportOrigin::SavedReference)
        };
        let target_movie = match candidates.as_slice() {
            [id] => *id,
            [] => {
                return Ok(Resolution::MissingMovie {
                    object_path: target,
                });
            }
            _ => {
                return Ok(Resolution::AmbiguousMovies {
                    object_path: target,
                    candidates,
                });
            }
        };
        let Some(ids) = self.entries[target_movie].exports.get(symbol) else {
            return Ok(Resolution::MissingSymbol {
                movie: target_movie,
            });
        };
        if ids.len() != 1 {
            return Ok(Resolution::AmbiguousSymbol {
                movie: target_movie,
                candidates: ids.iter().copied().collect(),
            });
        }
        Ok(Resolution::Resolved {
            movie: target_movie,
            character_id: *ids.first().unwrap(),
            origin,
        })
    }
}

fn canonical_path(path: &str) -> io::Result<String> {
    if path.is_empty()
        || path.len() > 65_536
        || path
            .split('.')
            .any(|s| s.is_empty() || s.contains(['/', '\\', ':', '?', '#', '\0']))
    {
        return Err(invalid("invalid logical object path"));
    }
    Ok(path.to_ascii_lowercase())
}

/// Normalize a saved relative SWF/GFX identifier into a UE3 logical object path.
/// This is an identifier operation; it never traverses the filesystem.
pub fn logical_movie_path(source_object_path: &str, url: &str) -> io::Result<String> {
    let source = canonical_path(source_object_path)?;
    if url.is_empty()
        || url.len() > 65_536
        || url.starts_with(['/', '\\'])
        || url.contains([':', '?', '#', '\0'])
    {
        return Err(invalid("unsupported logical movie URL"));
    }
    let mut parts: Vec<_> = source.split('.').map(str::to_owned).collect();
    parts.pop();
    let normalized = url.replace('\\', "/");
    let elements: Vec<_> = normalized.split('/').collect();
    for (index, element) in elements.iter().enumerate() {
        if index + 1 == elements.len() {
            let stem = if let Some((stem, extension)) = element.rsplit_once('.') {
                if !["swf", "gfx"]
                    .iter()
                    .any(|s| extension.eq_ignore_ascii_case(s))
                {
                    return Err(invalid("unsupported movie identifier extension"));
                }
                stem
            } else {
                element
            };
            if stem.is_empty() || stem.contains('.') {
                return Err(invalid("unsupported movie identifier stem"));
            }
            parts.push(stem.to_owned());
        } else {
            match *element {
                "." => {}
                ".." => {
                    parts
                        .pop()
                        .ok_or_else(|| invalid("movie identifier escapes logical root"))?;
                }
                "" => return Err(invalid("empty logical movie path component")),
                name if name.contains('.') => {
                    return Err(invalid("dotted logical movie directory"));
                }
                name => parts.push(name.to_owned()),
            }
        }
    }
    canonical_path(&parts.join("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn logical_paths_preserve_movie_names_and_never_open_urls() {
        assert_eq!(
            logical_movie_path(
                "MarvelFrontEnd.marvel_login_dialog",
                "../MarvelHUD/!UISource.swf"
            )
            .unwrap(),
            "marvelhud.!uisource"
        );
        assert_eq!(
            logical_movie_path("MarvelFrontEnd.marvel_login_dialog", "marvel_assets.swf").unwrap(),
            "marvelfrontend.marvel_assets"
        );
        assert_eq!(
            logical_movie_path("MarvelFrontEnd.marvel_cinematics_panel", "marvel_assets").unwrap(),
            "marvelfrontend.marvel_assets"
        );
        assert_eq!(
            logical_movie_path(
                "MarvelFrontEnd.marvel_login_dialog",
                "../MarvelFonts/gfxfontlib.swf"
            )
            .unwrap(),
            "marvelfonts.gfxfontlib"
        );
        for url in [
            "https://example.com/a.swf",
            "../../other.swf",
            "/a.swf",
            "a.swf?x=1",
            "../x/../..",
            "../x/a.b.swf",
        ] {
            assert!(
                logical_movie_path("MarvelFrontEnd.login", url).is_err(),
                "{url}"
            );
        }
    }
}
