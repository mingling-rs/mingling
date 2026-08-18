use mingling::{
    Grouped,
    macros::{buffer, command, r_println, renderer},
};

use crate::res::ResFeatureList;

#[command(node = "show-features")]
pub fn show_features(features: &ResFeatureList) -> ResultShowFeatures {
    ResultShowFeatures {
        features: features.list.clone(),
    }
}

/// The docs.rs feature list of `mingling`.
#[derive(Grouped)]
pub struct ResultShowFeatures {
    pub features: Vec<String>,
}

#[renderer(buffer)]
pub fn render_show_features(r: ResultShowFeatures) {
    for feature in r.features {
        r_println!("{feature}");
    }
}
