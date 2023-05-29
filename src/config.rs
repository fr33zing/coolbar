// Catppuccin Macchiato
// https://github.com/catppuccin
#[allow(dead_code)]
const ROSEWATER: &str = "#F4DBD6";
#[allow(dead_code)]
const FLAMINGO: &str = "#F0C6C6";
#[allow(dead_code)]
const PINK: &str = "#F5BDE6";
#[allow(dead_code)]
const MAUVE: &str = "#C6A0F6";
#[allow(dead_code)]
const RED: &str = "#ED8796";
#[allow(dead_code)]
const MAROON: &str = "#EE99A0";
#[allow(dead_code)]
const PEACH: &str = "#F5A97F";
#[allow(dead_code)]
const YELLOW: &str = "#EED49F";
#[allow(dead_code)]
const GREEN: &str = "#A6DA95";
#[allow(dead_code)]
const TEAL: &str = "#8BD5CA";
#[allow(dead_code)]
const SKY: &str = "#91D7E3";
#[allow(dead_code)]
const SAPPHIRE: &str = "#7DC4E4";
#[allow(dead_code)]
const BLUE: &str = "#8AADF4";
#[allow(dead_code)]
const LAVENDER: &str = "#B7BDF8";
#[allow(dead_code)]
const TEXT: &str = "#CAD3F5";
#[allow(dead_code)]
const SUBTEXT1: &str = "#B8C0E0";
#[allow(dead_code)]
const SUBTEXT0: &str = "#A5ADCB";
#[allow(dead_code)]
const OVERLAY2: &str = "#939AB7";
#[allow(dead_code)]
const OVERLAY1: &str = "#8087A2";
#[allow(dead_code)]
const OVERLAY0: &str = "#6E738D";
#[allow(dead_code)]
const SURFACE2: &str = "#5B6078";
#[allow(dead_code)]
const SURFACE1: &str = "#494D64";
#[allow(dead_code)]
const SURFACE0: &str = "#363A4F";
#[allow(dead_code)]
const BASE: &str = "#24273A";
#[allow(dead_code)]
const MANTLE: &str = "#1E2030";
#[allow(dead_code)]
const CRUST: &str = "#181926";

#[derive(Clone)]
pub struct Theme {
    font_family: String,
    font_size: String,
    outer_padding: String,
    background: String,
}

#[derive(Clone)]
pub struct Config {
    theme: Theme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme {
                font_family: "Iosevka".into(),
                font_size: "16px".into(),
                outer_padding: "20px".into(),
                background: BASE.into(),
            },
        }
    }
}

impl Config {
    pub fn scss_variables(&self) -> String {
        let vars = [
            ("font_family", &self.theme.font_family),
            ("font_size", &self.theme.font_size),
            ("outer_padding", &self.theme.outer_padding),
            ("background", &self.theme.background),
        ];

        vars.map(|t| format!("${}: {};", t.0, t.1)).join("\n")
    }
}
