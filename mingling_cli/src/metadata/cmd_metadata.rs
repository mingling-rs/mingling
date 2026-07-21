use cargo_metadata::Metadata;
use mingling::{
    LazyRes,
    macros::{chain, dispatcher, pack},
};

use crate::metadata::setup::ResMetadata;

dispatcher!("metadata");

pack!(ResultMetadata = ResMetadata);

#[chain]
pub fn handle_metadata(_: EntryMetadata, metadata: &mut LazyRes<ResMetadata>) -> Metadata {
    let metadata = metadata.get_ref().clone();
    metadata.data().clone()
}
