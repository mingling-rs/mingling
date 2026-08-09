use mingling::{
    LazyRes, Routable, ShellContext, Suggest,
    macros::{
        arg, buffer, chain, command, completion, metadata, pack, r_println, renderer, suggest,
    },
    metadata::Description,
    picker::{EntryPicker, PickerArg, value::Flag},
};

use crate::{Entry, Next, config::ResMlingConfig};

const FLAG_PAIR: PickerArg<Flag> = arg![pair: Flag];

pack!(StateConfigEdit = (String, String));
pack!(ResultConfigKeyValuePair = String);
pack!(ResultConfigValue = String);
pack!(ResultConfig = ());

#[command]
pub fn cfg(args: Entry) -> Next {
    let (key, value, show_pair) = args
        .pick(&arg![Option<String>])
        .pick(&arg![Option<String>])
        .pick(&FLAG_PAIR)
        .unwrap();

    match (key, value) {
        (Some(k), Some(v)) => {
            // Edit
            StateConfigEdit::new((k, v)).into()
        }
        (Some(k), None) => {
            // Display
            if *show_pair {
                ResultConfigKeyValuePair::new(k).to_render()
            } else {
                ResultConfigValue::new(k).to_render()
            }
        }
        (None, None) => {
            // List
            ResultConfig::new(()).to_render()
        }
        _ => {
            unreachable!("This path is unreachable given the positional parsing done by arg-picker")
        }
    }
}

#[chain]
pub fn handle_state_config_edit(kv: StateConfigEdit, config: &mut LazyRes<ResMlingConfig>) {
    let config = config.get_mut();
    config.edit(&kv.0, &kv.1);
}

#[renderer(buffer)]
pub fn render_config_kvp(r: ResultConfigKeyValuePair, config: &mut LazyRes<ResMlingConfig>) {
    let config = config.get_ref();
    let key = r.inner;
    let value = config.get(&key);
    r_println!(
        "\"{}\" = \"{}\"",
        escape_config_value(&key),
        escape_config_value(value)
    )
}

#[renderer(buffer)]
pub fn render_config_value(r: ResultConfigValue, config: &mut LazyRes<ResMlingConfig>) {
    let config = config.get_ref();
    let key = r.inner;
    let value = config.get(&key);
    r_println!("{}", value)
}

#[renderer(buffer)]
pub fn render_config(_: ResultConfig, config: &mut LazyRes<ResMlingConfig>) {
    let config = config.get_ref();
    for (k, v) in config.get_hash_map().iter() {
        r_println!(
            "\"{}\" = \"{}\"",
            escape_config_value(k),
            escape_config_value(v)
        )
    }
}

/// Utility function: escapes `\t`, `\n`, `\b`, `\r` in the string to their
/// literal representations `\\t`, `\\n`, `\\b`, `\\r`, then trims leading and
/// trailing whitespace, and escapes `"` to `\"`.
pub fn escape_config_value(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for ch in input.trim().chars() {
        match ch {
            '\t' => result.push_str("\\t"),
            '\n' => result.push_str("\\n"),
            '\u{0008}' => result.push_str("\\b"),
            '\r' => result.push_str("\\r"),
            '"' => result.push_str("\\\""),
            _ => result.push(ch),
        }
    }
    result
}

#[completion(EntryCfg)]
pub fn complete_config(_ctx: &ShellContext, config: &mut LazyRes<ResMlingConfig>) -> Suggest {
    let config = config.get_ref();
    let keys = config.get_hash_map().keys().cloned().collect::<Vec<_>>();
    Suggest::from(keys).combine(suggest! {
        FLAG_PAIR
    })
}

#[metadata(EntryCfg)]
pub fn desc_cfg() -> Description {
    "View and edit Mling's user configuration file".into()
}
