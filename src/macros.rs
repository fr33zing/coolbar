#[macro_export]
macro_rules! pango_span {
    ($format_string:expr, { $( $key:ident: $value:expr ),* } ) => {
        format!(
            "<span {}>{}</span>",
            vec![ $( format!("{}=\"{}\"", stringify!($key), $value) )* ].join(" "),
            $format_string
        )
    }
}

/**
Use this macro to make components available for dynamic creation via user configuration.

This macro will:
- Declare and use each module
- Create the ComponentConfig enum
- Give AppModel vectors to store multiple of each component
- Create generate_child_from_config extension function

# Usage

```rust
// File: components/mod.rs
component_list![time, volume, workspaces, razer_mouse];
```

Each argument should be the name of an undeclared module under `crate::components`. In this module,
the model struct must be named the same as the module (but in camelcase) suffixed with `Model`. For
example, for a module named `razer_mouse`, the model struct must be named `RazerMouseModel`.

```rust
// File: razer_mouse.rs
struct RazerMouseModel { ... }
```

The model's `Init` will be available for use in the `components` section of the user configuration.

```yaml
# File: config.yaml
components:
  # Give this component instance an arbitrary name
  my_razer_mouse_component:
    # Tag struct with module name to indicate component type
    type: razer_mouse
    # Define fields of `RazerMouseModel::Init`
    icon:
      type: material
      id: mouse
    icon_charging:
      type: multiple
      icons:
      - type: material
        id: mouse
      - type: material
        id: bolt
```
*/
#[macro_export]
macro_rules! component_list {
    [ $( $module:ident ),* ] => [
        use paste::paste;
        use serde::{Deserialize, Serialize};
        use relm4::{
            gtk::traits::BoxExt,
            component::{AsyncComponent, AsyncController, AsyncComponentController}
        };

        paste! {
            // Import the module and use its model.
            $(
                pub mod $module;
                use $module::[<$module:camel Model>];
            )*

            // Create an enum based on each component's Init. This is used to define the
            // configuration options on a per-component-instance basis.
            #[derive(Debug, Clone, Serialize, Deserialize)]
            #[serde(tag = "type", rename_all = "snake_case")]
            pub enum ComponentConfig {
                $(
                    #[allow(dead_code)]
                    [<$module:camel>] {
                        #[serde(flatten)]
                        init: <[<$module:camel Model>] as AsyncComponent>::Init,
                    },
                )*
            }

            // Give AppModel a vector for each component so we have somewhere to put them after
            // they are generated from a ComponentConfig.
            #[derive(Default)]
            pub struct AppModel {
                $(
                    [<$module:snake _controllers>]: Vec<AsyncController<[<$module:camel Model>]>>,
                )*
            }

            // Enable conversion of configuration into component. Use a trait to improve clarity.
            pub trait ConfigWidgetExt {
                /// Generate a component from its configuration and append it to the container.
                fn generate_child_from_config(
                    &self,
                    app_model: &mut AppModel,
                    component_config: &ComponentConfig,
                );
            }

            impl<T: gtk::glib::IsA<gtk::Widget>> ConfigWidgetExt for T
            where
                T: BoxExt,
            {
                fn generate_child_from_config(
                    &self,
                    app_model: &mut AppModel,
                    component_config: &ComponentConfig,
                ) {
                    match component_config {
                        $(
                            ComponentConfig::[<$module:camel>] { init } => {
                                let controller = [<$module:camel Model>]::builder()
                                    .launch(init.clone())
                                    .detach();
                                app_model.[<$module:snake _controllers>].push(controller);
                                let widget = app_model
                                    .[<$module:snake _controllers>]
                                    .last()
                                    .unwrap()
                                    .widget();
                                self.append(widget);
                            }
                        )*
                    };
                }
            }
        }
    ];
}
