use mingling::macros::{chain, dispatcher};

dispatcher!("mlint", CMDMinglingLinter => EntryMinglingLinter);

#[chain]
pub fn handle_mlint(_args: EntryMinglingLinter) {}
