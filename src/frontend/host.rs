//! Explicit frontend native state shared by one AVM1 movie-root context.
//! Unimplemented interfaces reject calls; registration does not construct clips.

use super::{
    class_registry::{ClassRegistry, Limits as ClassLimits},
    display_state::DisplayState,
    selection::{Limits as SelectionLimits, SelectionHost},
};
use crate::avm1::vm::{
    ClassRegistration, Host, HostCall, HostCommand, ObjectId, PropertyRequest, Value, Vm,
};
use std::{error::Error, io};

pub struct FrontendHost {
    selection: SelectionHost,
    classes: ClassRegistry,
    display: Option<DisplayState>,
}

impl FrontendHost {
    pub fn install(vm: &mut Vm, movie_version: u8) -> Result<Self, Box<dyn Error>> {
        let classes = ClassRegistry::new(vm.root(), movie_version, ClassLimits::default())
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
        let selection = SelectionHost::install(vm, SelectionLimits::default())?;
        Ok(Self {
            selection,
            classes,
            display: None,
        })
    }
    pub fn selection(&self) -> &SelectionHost {
        &self.selection
    }
    pub fn classes(&self) -> &ClassRegistry {
        &self.classes
    }

    /// Associate an explicit scene from this host's VM. Numeric object IDs are
    /// VM-local; the caller must not supply bindings from a different VM.
    /// No display object, event or native lifecycle state is manufactured here.
    pub fn attach_display(&mut self, display: DisplayState) -> Result<(), String> {
        if self.display.is_some() {
            return Err("frontend display is already attached".into());
        }
        if display.bindings().get(&self.classes.context().root) != Some(&display.root()) {
            return Err("frontend display root must bind the owning VM root".into());
        }
        self.display = Some(display);
        Ok(())
    }
    pub fn display(&self) -> Option<&DisplayState> {
        self.display.as_ref()
    }
    pub fn display_mut(&mut self) -> Option<&mut DisplayState> {
        self.display.as_mut()
    }
}

impl Host for FrontendHost {
    fn call(&mut self, call: HostCall<'_>) -> Result<Value, String> {
        self.selection.call(call)
    }
    fn command(&mut self, _: ObjectId, command: HostCommand) -> Result<(), String> {
        Err(format!("unhandled frontend command {command:?}"))
    }
    fn register_class(&mut self, registration: ClassRegistration) -> Result<bool, String> {
        self.classes.register(registration)
    }
    fn get_property(&mut self, request: PropertyRequest) -> Result<Value, String> {
        self.display
            .as_mut()
            .ok_or("frontend display is not attached")?
            .get_property(request)
    }
    fn set_property(&mut self, request: PropertyRequest, value: Value) -> Result<(), String> {
        self.display
            .as_mut()
            .ok_or("frontend display is not attached")?
            .set_property(request, value)
    }
}
