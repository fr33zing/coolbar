use std::collections::BTreeMap;

use tokio::sync::OnceCell;

static MATERIAL_DESIGN_ICONS_CODEPOINTS: OnceCell<BTreeMap<String, String>> = OnceCell::const_new();

pub fn load_codepoints() {
    MATERIAL_DESIGN_ICONS_CODEPOINTS
        .set({
            let text = std::include_str!("resources/MaterialIconsRound-Regular.codepoints");
            let tuples = text
                .trim()
                .split("\n")
                .filter_map(|line| line.split_once(" "))
                .map(|tup| (tup.0.to_owned(), tup.1.to_owned()));
            BTreeMap::from_iter(tuples)
        })
        .expect("failed to load codepoints for material design icons");
}

pub fn material_design_icon(id: &str) -> String {
    let codepoint = MATERIAL_DESIGN_ICONS_CODEPOINTS
        .get()
        .expect("failed to get codepoints for material design icons")
        .get(id)
        .expect(format!("failed to lookup icon: {id}").as_str());
    let value = u32::from_str_radix(codepoint, 16).unwrap();
    let c = char::from_u32(value).unwrap();
    c.to_string()
}
