use crate::window;

/* macro_rules! load_strings_from_map {
    ($global:expr, $map:expr, [ $($key:literal => $setter:ident),* $(,)? ]) => {
        $(
            if let Some(val) = $map.get($key) {
                $global.$setter(val.clone().into());
            }
        )*
    };
} */

pub fn load(window: &window::WindowWrapper<window::MainWindow>) {
    // TODO: get config then load the target lang

    window.get_global::<window::AppStrings>()
        .upgrade().expect("failed upgrade")
        .set_data(window::LocalizationData::with_defaults());
}

pub trait WithDefaults {
    fn with_defaults() -> Self;
}

impl WithDefaults for window::LocalizationData {
    fn with_defaults() -> Self {
        Self {
            app_name: "Oxidrive (ilovegozo)".into(),
            mv_navbar_item_pedals: "Pedals".into(),
            mv_navbar_item_presets: "Presets".into(),
            mv_navbar_item_plugins: "Plugins".into(),
        }
    }
}

