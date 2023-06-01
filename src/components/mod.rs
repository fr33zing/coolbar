#![allow(non_camel_case_types)] // TODO figure out how to get rid of this

use crate::component_list;
mod iconbutton;

// Define components here.
// Format: `module => model`
//
// The macro will handle declaring / importing / exporting everything.
// The model's `Init` will be added to the config under `components.$model`.
component_list! {
    time => TimeModel,
    volume => VolumeModel,
    workspaces => WorkspacesModel,
    razer_mouse => RazerMouseModel
}
