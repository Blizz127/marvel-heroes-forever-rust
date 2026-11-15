//! Prototype lookup and lazy decoding from a caller-selected Calligraphy package.
//! This preserves encoded values; it does not merge parent values or bind game classes.

use std::collections::HashMap;
use std::io;
use std::path::Path;

use crate::calligraphy::{
    Blueprint, Directory, DirectoryRecord, PrototypeRecord,
    prototype::{FieldData, Prototype, SchemaSet, Value},
};
use crate::sip::SipArchive;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

pub struct PrototypeCatalog {
    archive: SipArchive,
    blueprints: Vec<Blueprint>,
    directory: Directory,
    by_id: HashMap<u64, usize>,
    by_member: HashMap<String, usize>,
}

impl PrototypeCatalog {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let mut archive = SipArchive::open(path)?;
        let directory = Directory::parse(&archive.read("Calligraphy/Prototype.directory")?)
            .map_err(|error| invalid(format!("Prototype.directory: {error}")))?;
        let blueprint_entries: Vec<_> = archive
            .entries()
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.name.ends_with(".blueprint"))
            .map(|(index, _)| index)
            .collect();
        let mut blueprints = Vec::new();
        blueprints
            .try_reserve(blueprint_entries.len())
            .map_err(|error| invalid(error.to_string()))?;
        for index in blueprint_entries {
            let bytes = archive.read_index(index)?;
            blueprints.push(
                Blueprint::parse(&bytes).map_err(|error| {
                    invalid(format!("{}: {error}", archive.entries()[index].name))
                })?,
            );
        }
        // Validate metadata relationships before exposing lookup or file reads.
        SchemaSet::new(&blueprints, &directory).map_err(|error| invalid(error.to_string()))?;
        let mut by_id = HashMap::new();
        let mut by_member = HashMap::new();
        by_id
            .try_reserve(directory.records.len())
            .map_err(|error| invalid(error.to_string()))?;
        by_member
            .try_reserve(directory.records.len())
            .map_err(|error| invalid(error.to_string()))?;
        for (index, entry) in directory.records.iter().enumerate() {
            let DirectoryRecord::Prototype(record) = entry else {
                return Err(invalid("non-prototype directory record"));
            };
            let member = format!("Calligraphy/{}", record.path);
            if archive.entry(&member).is_none() {
                return Err(invalid(format!(
                    "prototype directory member missing: {member}"
                )));
            }
            if by_id.insert(record.id, index).is_some() || by_member.insert(member, index).is_some()
            {
                return Err(invalid("duplicate prototype ID or member path"));
            }
        }
        Ok(Self {
            archive,
            blueprints,
            directory,
            by_id,
            by_member,
        })
    }

    pub fn prototype_count(&self) -> usize {
        self.directory.records.len()
    }
    pub fn blueprint_count(&self) -> usize {
        self.blueprints.len()
    }

    /// Construct one reusable schema index and borrow the archive for a batch.
    pub fn reader(&mut self) -> io::Result<PrototypeReader<'_>> {
        let schemas = SchemaSet::new(&self.blueprints, &self.directory)
            .map_err(|error| invalid(error.to_string()))?;
        Ok(PrototypeReader {
            archive: &mut self.archive,
            schemas,
            directory: &self.directory,
            by_id: &self.by_id,
            by_member: &self.by_member,
        })
    }
}

pub struct PrototypeReader<'a> {
    archive: &'a mut SipArchive,
    schemas: SchemaSet<'a>,
    directory: &'a Directory,
    by_id: &'a HashMap<u64, usize>,
    by_member: &'a HashMap<String, usize>,
}

impl PrototypeReader<'_> {
    pub fn record(&self, index: usize) -> Option<&PrototypeRecord> {
        match self.directory.records.get(index)? {
            DirectoryRecord::Prototype(record) => Some(record),
            _ => None,
        }
    }

    pub fn read_index(&mut self, index: usize) -> io::Result<Prototype> {
        let record = self
            .record(index)
            .ok_or_else(|| invalid("prototype index out of range"))?;
        let blueprint_id = record.blueprint_id;
        let member = format!("Calligraphy/{}", record.path);
        let bytes = self.archive.read(&member)?;
        Prototype::parse(&bytes, blueprint_id, &self.schemas)
            .map_err(|error| invalid(format!("{member}: {error}")))
    }

    pub fn read_id(&mut self, id: u64) -> io::Result<Prototype> {
        let index = self
            .by_id
            .get(&id)
            .copied()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "prototype ID not found"))?;
        self.read_index(index)
    }

    /// Exact archive member name, including the Calligraphy/ prefix.
    pub fn read_member(&mut self, member: &str) -> io::Result<Prototype> {
        let index =
            self.by_member.get(member).copied().ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "prototype member not found")
            })?;
        self.read_index(index)
    }

    pub fn verify_all(&mut self) -> io::Result<PrototypeStatistics> {
        let mut statistics = PrototypeStatistics::default();
        for index in 0..self.directory.records.len() {
            let prototype = self.read_index(index)?;
            statistics.files += 1;
            statistics.include(&prototype);
        }
        Ok(statistics)
    }
}

/// Counts of decoded structure only, not runtime object or behavior counts.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PrototypeStatistics {
    pub files: u64,
    pub nodes: u64,
    pub groups: u64,
    pub simple_fields: u64,
    pub list_fields: u64,
    pub list_elements: u64,
    pub values: u64,
    pub max_depth: usize,
}

impl PrototypeStatistics {
    pub fn include(&mut self, root: &Prototype) {
        let mut pending = vec![(root, 0)];
        while let Some((prototype, depth)) = pending.pop() {
            self.nodes += 1;
            self.groups += prototype.groups.len() as u64;
            self.max_depth = self.max_depth.max(depth);
            for group in &prototype.groups {
                self.simple_fields += group.simple_fields.len() as u64;
                self.list_fields += group.list_fields.len() as u64;
                for field in group.simple_fields.iter().chain(&group.list_fields) {
                    let values = match &field.data {
                        FieldData::Simple(value) => std::slice::from_ref(value),
                        FieldData::List(values) => {
                            self.list_elements += values.len() as u64;
                            values.as_slice()
                        }
                    };
                    self.values += values.len() as u64;
                    for value in values {
                        if let Value::EmbeddedPrototype(child) = value {
                            pending.push((child, depth + 1));
                        }
                    }
                }
            }
        }
    }
}
