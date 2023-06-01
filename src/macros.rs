#[macro_export]
macro_rules! component_list {
    [ $( $module:ident => $model:ident ),* ] => {

        use serde::{Deserialize, Serialize};
        use relm4::{
            gtk::traits::BoxExt,
            component::{AsyncComponent, AsyncController, AsyncComponentController}
        };

        $(
            pub mod $module;
            use $module::$model;
        )*

        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "type")]
        pub enum ConfigComponent {
            $(
                #[allow(dead_code)]
                $module {
                    #[serde(flatten)]
                    init: <$model as AsyncComponent>::Init,
                },
            )*
        }

        #[derive(Default)]
        pub struct AppModel {
            $(
                $module: Vec<AsyncController<$model>>,
            )*
        }

        pub trait ConfigWidgetExt {
            fn generate_from_config(
                &self,
                app_model: &mut AppModel,
                config_component: &ConfigComponent
            );
        }

        impl<T: gtk::glib::IsA<gtk::Widget>> ConfigWidgetExt for T
        where
            T: BoxExt,
        {
            fn generate_from_config(
                &self,
                app_model: &mut AppModel,
                config_component: &ConfigComponent
            ) {
                match config_component {
                    $(
                        ConfigComponent::$module { init } => {
                            let controller = $model::builder().launch(init.clone()).detach();
                            app_model.$module.push(controller);
                            self.append(app_model.$module.last().unwrap().widget());
                        }
                    )*
                };
            }
        }
    };
}
