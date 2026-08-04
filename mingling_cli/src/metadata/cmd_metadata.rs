use cargo_metadata::Metadata;
use mingling::{
    LazyRes,
    macros::{chain, dispatcher, metadata, pack},
    metadata::Description,
};

use crate::metadata::setup::ResMetadata;

dispatcher!("metadata");

#[metadata(EntryMetadata)]
pub fn desc_metadata() -> Description {
    "Check your workspace metadata using 'cargo metadata'"
        .to_string()
        .into()
}

pack!(ResultMetadata = ResMetadata);

#[chain]
pub fn handle_metadata(_: EntryMetadata, metadata: &mut LazyRes<ResMetadata>) -> Metadata {
    let metadata = metadata.get_ref().clone();
    metadata.data().clone()
}
