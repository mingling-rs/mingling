#![cfg(test)]

use std::{collections::HashMap, env::current_dir};

#[test]
fn test_module_pathf() {
    let dir = current_dir().unwrap().join("test_proj");
    let mapping = mingling_pathf::module_pathf::analyze(&dir)
        .unwrap()
        .into_iter()
        .map(|i| {
            let file_path = just_fmt::fmt_path::fmt_path(i.file_path())
                .unwrap()
                .display()
                .to_string();
            let module_path = i.module_path();
            (file_path, module_path.to_string())
        }
        ).collect::<HashMap<String, String>>();

    assert!(mapping.contains_key("src/has_sub_use/sub_mod.rs"));
    assert!(mapping.contains_key("src/has_sub_use/sub_use.rs"));
    assert!(mapping.contains_key("src/has_sub_mod/sub_mod.rs"));
    assert!(mapping.contains_key("src/has_sub_use.rs"));
    assert!(mapping.contains_key("src/has_sub_mod.rs"));
    assert!(mapping.contains_key("src/directly_mod.rs"));
    assert!(mapping.contains_key("src/use_all.rs"));
    assert!(mapping.contains_key("src/main.rs"));
    assert!(!mapping.contains_key("src/unused.rs"));

    assert_eq!(mapping.get("src/has_sub_use/sub_mod.rs").unwrap(), "crate::sub_mod");
    assert_eq!(mapping.get("src/has_sub_use/sub_use.rs").unwrap(), "crate");
    assert_eq!(mapping.get("src/has_sub_mod/sub_mod.rs").unwrap(), "crate::has_sub_mod::sub_mod");
    assert_eq!(mapping.get("src/has_sub_use.rs").unwrap(), "crate");
    assert_eq!(mapping.get("src/has_sub_mod.rs").unwrap(), "crate::has_sub_mod");
    assert_eq!(mapping.get("src/directly_mod.rs").unwrap(), "crate::directly_mod");
    assert_eq!(mapping.get("src/use_all.rs").unwrap(), "crate");
    assert_eq!(mapping.get("src/main.rs").unwrap(), "crate");
}
