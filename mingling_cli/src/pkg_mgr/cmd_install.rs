use mingling::{LazyRes, macros::command};

use crate::metadata::setup::ResMetadata;

#[command]
pub fn install(_metadata: &mut LazyRes<ResMetadata>) {
    // let metadata = metadata.get_ref().data();
}
