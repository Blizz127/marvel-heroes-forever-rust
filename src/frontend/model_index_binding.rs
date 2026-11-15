//! Model index construction from the same retained Level/component namespace.
//!
//! Source bytes remain borrowed from the supplied binding. Component invocation
//! order, resolved identities and the post-callback snapshot are caller inputs;
//! this bridge does not supply runtime or material state.

use super::{AssetSet, model_component_binding::Binding};
use crate::upk::model_index_resource::{
    self, ExportInput, InitialBuffer, Limits, MapEntry, ModelIndexResource, ReferenceKey,
    SnapshotPhase,
};
use std::io;

#[derive(Clone, Copy, Debug)]
pub struct Request<'a> {
    /// Zero-based ordinals in Binding::components(), in requested invocation order.
    /// Empty and repeated selections remain explicit; no component discovery runs.
    pub component_ordinals: &'a [usize],
    pub phase: SnapshotPhase,
    pub references: &'a [ReferenceKey],
    pub initial_buffers: &'a [InitialBuffer<'a>],
    pub initial_map: &'a [MapEntry],
}

#[derive(Debug)]
pub struct Prepared<'a> {
    source: &'a Binding,
    resource: ModelIndexResource<'a>,
}
impl<'a> Prepared<'a> {
    pub fn source(&self) -> &'a Binding {
        self.source
    }
    pub fn resource(&self) -> &ModelIndexResource<'a> {
        &self.resource
    }
    pub fn matches_assets(&self, assets: &AssetSet) -> bool {
        self.source.matches_assets(assets)
    }
}

/// Compose a previously checked Level binding with the selected integer writer.
///
/// The temporary request slices are copied or consumed during construction; only
/// original export bytes borrow from `source`. Retaining this result retains that
/// borrow and namespace identity, including after the temporary request is gone.
/// No files are reread, references resolved, callbacks run or renderer enabled.
pub fn prepare<'a>(
    assets: &AssetSet,
    source: &'a Binding,
    request: Request<'_>,
    limits: Limits,
) -> io::Result<Prepared<'a>> {
    if !source.matches_assets(assets) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Model index source belongs to a different AssetSet",
        ));
    }
    if request.component_ordinals.len() > limits.components {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Model index component selection limit exceeded",
        ));
    }
    let components = request
        .component_ordinals
        .iter()
        .map(|&index| {
            let component = source.components().get(index).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Model index component ordinal is out of range",
                )
            })?;
            Ok(ExportInput {
                index: component.export.location.export,
                bytes: &component.export.bytes,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    let model = source.model();
    let resource = ModelIndexResource::construct(
        assets.archive(model.location.package)?.package(),
        model_index_resource::Request {
            model: ExportInput {
                index: model.location.export,
                bytes: &model.bytes,
            },
            components: &components,
            phase: request.phase,
            references: request.references,
            initial_buffers: request.initial_buffers,
            initial_map: request.initial_map,
        },
        limits,
    )?;
    Ok(Prepared { source, resource })
}
