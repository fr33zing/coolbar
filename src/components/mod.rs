use crate::component_list;
mod iconbutton;

/*
Define components here.

The macro will:
  - Declare the module
  - Create the ComponentConfig enum
  - Give AppModel a vector to store multiple of each component
  - Create generate_child_from_config extension function
*/
component_list![power, time, volume, workspaces, razer_mouse];
