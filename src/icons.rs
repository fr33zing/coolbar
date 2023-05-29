use std::{cell::OnceCell, collections::BTreeMap, sync::Mutex};

static MATERIAL_DESIGN_ICONS_CODEPOINTS: Mutex<OnceCell<BTreeMap<String, String>>> =
    Mutex::new(OnceCell::new());

pub fn material_design_icon(id: &str) -> String {
    let lock = MATERIAL_DESIGN_ICONS_CODEPOINTS.lock().unwrap();
    let codepoints = lock.get_or_init(|| {
        let text = std::include_str!("resources/MaterialIconsRound-Regular.codepoints");
        let tuples = text
            .trim()
            .split("\n")
            .filter_map(|line| line.split_once(" "))
            .map(|tup| (tup.0.to_owned(), tup.1.to_owned()));
        BTreeMap::from_iter(tuples)
    });
    let codepoint = codepoints.get(id).expect("valid icon id");
    let value = u32::from_str_radix(codepoint, 16).unwrap();
    let c = char::from_u32(value).unwrap();
    c.to_string()
}
