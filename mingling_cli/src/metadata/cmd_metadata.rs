use mingling::{
    LazyRes,
    macros::{chain, dispatcher, pack_structural},
};

use crate::metadata::setup::ResMetadata;

dispatcher!("metadata");

pack_structural!(ResultMetadata = ResMetadata);

#[chain]
pub fn handle_metadata(_: EntryMetadata, metadata: &mut LazyRes<ResMetadata>) -> ResultMetadata {
    let metadata = metadata.get_ref().clone();
    ResultMetadata::new(metadata)
}
