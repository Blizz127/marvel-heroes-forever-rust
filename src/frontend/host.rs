//! Explicit frontend native state shared by one AVM1 movie-root context.
//! Unimplemented interfaces reject calls; registration does not construct clips.

use super::{
    class_registry::{ClassRegistry, Limits as ClassLimits},
    selection::{Limits as SelectionLimits, SelectionHost},
};
use crate::avm1::vm::{ClassRegistration, Host, HostCall, HostCommand, ObjectId, Value, Vm};
use std::{error::Error, io};

pub struct FrontendHost {
    selection: SelectionHost,
    classes: ClassRegistry,
}

impl FrontendHost {
    pub fn install(vm: &mut Vm, movie_version: u8) -> Result<Self, Box<dyn Error>> {
        let classes = ClassRegistry::new(vm.root(), movie_version, ClassLimits::default())
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
        let selection = SelectionHost::install(vm, SelectionLimits::default())?;
        Ok(Self { selection, classes })
    }
    pub fn selection(&self) -> &SelectionHost {
        &self.selection
    }
    pub fn classes(&self) -> &ClassRegistry {
        &self.classes
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
}
