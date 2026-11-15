//! Explicit frontend native state shared by one AVM1 movie-root context.
//! Unimplemented interfaces reject calls; registration does not construct clips.

use super::{
    class_registry::{ClassRegistry, Limits as ClassLimits},
    display_state::DisplayState,
    external_host::{CALL_BINDING, ExternalHost},
    selection::{Limits as SelectionLimits, SelectionHost},
    textfield_host::TextFieldHost,
};
use crate::avm1::vm::{
    ClassRegistration, Host, HostCall, HostCommand, HostResult, ObjectId, PropertyRequest,
    TextFieldMemberRequest, Value, Vm,
};
use std::{error::Error, io};

pub struct FrontendHost {
    selection: SelectionHost,
    classes: ClassRegistry,
    display: Option<DisplayState>,
    external: Option<ExternalHost>,
    text_fields: Option<TextFieldHost>,
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
            external: None,
            text_fields: None,
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
    pub fn attach_external(&mut self, external: ExternalHost) -> Result<(), String> {
        if self.external.is_some() {
            return Err("frontend external interface is already attached".into());
        }
        if external.root() != self.classes.context().root {
            return Err("frontend external interface must use the owning VM root".into());
        }
        self.external = Some(external);
        Ok(())
    }
    pub fn external(&self) -> Option<&ExternalHost> {
        self.external.as_ref()
    }
    pub fn external_mut(&mut self) -> Option<&mut ExternalHost> {
        self.external.as_mut()
    }
    /// Use the same VM/Scene domain supplied during field binding. This retains
    /// admitted state without creating a native text target or its lifecycle.
    pub fn attach_text_fields(&mut self, fields: TextFieldHost) -> Result<(), String> {
        if self.text_fields.is_some() {
            return Err("frontend TextField host is already attached".into());
        }
        if fields.root() != self.classes.context().root {
            return Err("frontend TextField host must use the owning VM root".into());
        }
        self.text_fields = Some(fields);
        Ok(())
    }
    pub fn text_fields(&self) -> Option<&TextFieldHost> {
        self.text_fields.as_ref()
    }
    pub fn text_fields_mut(&mut self) -> Option<&mut TextFieldHost> {
        self.text_fields.as_mut()
    }
}

impl Host for FrontendHost {
    fn call(&mut self, call: HostCall<'_>) -> Result<Value, String> {
        if call.binding == CALL_BINDING {
            return Err("frontend external calls require HostResult materialization".into());
        }
        self.selection.call(call)
    }
    fn call_result(&mut self, call: HostCall<'_>) -> Result<HostResult, String> {
        if call.binding == CALL_BINDING {
            return self
                .external
                .as_mut()
                .ok_or("frontend external interface is not attached")?
                .handle_call(&call)
                .expect("binding checked before routing");
        }
        self.selection.call(call).map(HostResult::Value)
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
    fn get_text_field_member(&mut self, request: TextFieldMemberRequest) -> Result<Value, String> {
        self.text_fields
            .as_mut()
            .ok_or("frontend TextField host is not attached")?
            .get(request)
    }
    fn set_text_field_member(
        &mut self,
        request: TextFieldMemberRequest,
        value: Value,
    ) -> Result<(), String> {
        self.text_fields
            .as_mut()
            .ok_or("frontend TextField host is not attached")?
            .set(request, value)
    }
}
