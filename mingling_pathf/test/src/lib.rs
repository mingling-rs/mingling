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
        })
        .collect::<HashMap<String, String>>();

    assert!(mapping.contains_key("src/has_sub_use/sub_mod.rs"));
    assert!(mapping.contains_key("src/has_sub_use/sub_use.rs"));
    assert!(mapping.contains_key("src/has_sub_mod/sub_mod.rs"));
    assert!(mapping.contains_key("src/has_sub_use.rs"));
    assert!(mapping.contains_key("src/has_sub_mod.rs"));
    assert!(mapping.contains_key("src/directly_mod.rs"));
    assert!(mapping.contains_key("src/use_all.rs"));
    assert!(mapping.contains_key("src/main.rs"));
    assert!(!mapping.contains_key("src/unused.rs"));

    assert_eq!(
        mapping.get("src/has_sub_use/sub_mod.rs").unwrap(),
        "crate::sub_mod"
    );
    assert_eq!(mapping.get("src/has_sub_use/sub_use.rs").unwrap(), "crate");
    assert_eq!(
        mapping.get("src/has_sub_mod/sub_mod.rs").unwrap(),
        "crate::has_sub_mod::sub_mod"
    );
    assert_eq!(mapping.get("src/has_sub_use.rs").unwrap(), "crate");
    assert_eq!(
        mapping.get("src/has_sub_mod.rs").unwrap(),
        "crate::has_sub_mod"
    );
    assert_eq!(
        mapping.get("src/directly_mod.rs").unwrap(),
        "crate::directly_mod"
    );
    assert_eq!(mapping.get("src/use_all.rs").unwrap(), "crate");
    assert_eq!(mapping.get("src/main.rs").unwrap(), "crate");
}

#[test]
fn test_pattern_analyzer_once() {
    let dir = current_dir().unwrap().join("test_proj");
    let analyzer = mingling_pathf::pattern_analyzer::init();

    let result = analyzer
        .analyze_file(dir.join("src/has_sub_mod.rs"))
        .unwrap();

    // NO, basic_struct is disabled.
    assert!(!result.contains("::directly_sub_mod::DirectlySubModStruct"));
}

#[test]
fn test_chain_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir().unwrap().join("src/test_files/test_chain.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required_entries: Vec<&str> = vec![
        "::sub::__mingling_chain_my_chain1",
        "::sub::__mingling_chain_my_chain2",
        "::sub::__mingling_chain_my_chain3",
        "::sub::__mingling_chain_my_chain4",
        "::sub::__mingling_chain_my_chain5",
        "::sub::__mingling_chain_my_chain6",
        "::__mingling_chain_my_chain1",
        "::__mingling_chain_my_chain2",
        "::__mingling_chain_my_chain3",
        "::__mingling_chain_my_chain4",
        "::__mingling_chain_my_chain5",
        "::__mingling_chain_my_chain6",
    ];

    assert_eq!(
        r.len(),
        required_entries.len(),
        "Result should contain exactly {} entries",
        required_entries.len()
    );

    for entry in &required_entries {
        assert!(
            r.iter().any(|e| e == entry),
            "Result should contain: {}",
            entry
        );
    }
}

#[test]
fn test_renderer_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_renderer.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required: Vec<&str> = vec![
        "::sub::__mingling_renderer_my_renderer1",
        "::sub::__mingling_renderer_my_renderer2",
        "::sub::__mingling_renderer_my_renderer3",
        "::sub::__mingling_renderer_my_renderer4",
        "::__mingling_renderer_my_renderer1",
        "::__mingling_renderer_my_renderer2",
        "::__mingling_renderer_my_renderer3",
        "::__mingling_renderer_my_renderer4",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {}", entry);
    }
}

#[test]
fn test_help_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir().unwrap().join("src/test_files/test_help.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required: Vec<&str> = vec![
        "::sub::__mingling_help_my_help1",
        "::sub::__mingling_help_my_help2",
        "::sub::__mingling_help_my_help3",
        "::sub::__mingling_help_my_help4",
        "::__mingling_help_my_help1",
        "::__mingling_help_my_help2",
        "::__mingling_help_my_help3",
        "::__mingling_help_my_help4",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {}", entry);
    }
}

#[test]
fn test_completion_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_completion.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required: Vec<&str> = vec![
        "::sub::__mingling_completion_my_completion1",
        "::sub::__mingling_completion_my_completion2",
        "::sub::__mingling_completion_my_completion3",
        "::sub::__mingling_completion_my_completion4",
        "::__mingling_completion_my_completion1",
        "::__mingling_completion_my_completion2",
        "::__mingling_completion_my_completion3",
        "::__mingling_completion_my_completion4",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {}", entry);
    }
}

#[test]
fn test_import_type_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_import_type.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required: Vec<&str> = vec![
        "::__mingling_import_std_io_error",
        "::__mingling_import_std_fmt_error",
        "::__mingling_import_std_num_parseinterror",
        "::__mingling_import_serde_json_error",
        "::sub::__mingling_import_std_io_error",
        "::sub::__mingling_import_std_fmt_error",
        "::sub::__mingling_import_std_num_parseinterror",
        "::sub::__mingling_import_serde_json_error",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {}", entry);
    }
}

#[test]
fn test_grouped_derive_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_grouped_derive.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required: Vec<&str> = vec![
        "::__mingling_type_derived1",
        "::__mingling_type_derived2",
        "::__mingling_type_derived3",
        "::__mingling_type_derived4",
        "::__mingling_type_enum_derived1",
        "::__mingling_type_enum_derived2",
        "::sub::__mingling_type_derived1",
        "::sub::__mingling_type_derived3",
        "::sub::__mingling_type_derived4",
        "::sub::__mingling_type_enum_derived1",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {}", entry);
    }
}

#[test]
fn test_structural_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_structural.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required: Vec<&str> = vec![
        // Root: local types registered via structural!
        "::Struct1",
        "::Struct2",
        // Sub: local types registered via structural!
        "::sub::Struct1",
        "::sub::Struct2",
        // Foreign type imported via `use` (deduplicated across modules)
        "::std::io::Error",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {}", entry);
    }
}

#[test]
fn test_dispatcher_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_dispatcher.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required: Vec<&str> = vec![
        "::__mingling_type_entry_greet",
        "::__mingling_dispatcher_greet",
        "::__mingling_type_entry_remote_add",
        "::__mingling_dispatcher_remote_add",
        "::__mingling_type_entry_delete",
        "::__mingling_dispatcher_delete",
        "::__mingling_type_entry_remote_rm",
        "::__mingling_dispatcher_remote_rm",
        "::sub::__mingling_type_entry_greet",
        "::sub::__mingling_dispatcher_greet",
        "::sub::__mingling_type_entry_delete",
        "::sub::__mingling_dispatcher_delete",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {}", entry);
    }
}

#[test]
fn test_dispatcher_dispatch_tree() {
    use mingling_pathf::pattern_analyzer;

    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_dispatcher_dispatch_tree.rs");

    // Dispatchers are always collected at compile time. Each dispatcher now
    // contributes only its Entry type plus the `__mingling_dispatcher_*`
    // namespace module.
    let r = pattern_analyzer::init().analyze_file(&file).unwrap();
    assert_eq!(r.len(), 8);
    assert!(r.contains("::__mingling_type_entry_greet"));
    assert!(r.contains("::__mingling_dispatcher_greet"));
    assert!(r.contains("::__mingling_type_entry_delete"));
    assert!(r.contains("::__mingling_dispatcher_delete"));
    assert!(r.contains("::sub::__mingling_type_entry_greet"));
    assert!(r.contains("::sub::__mingling_dispatcher_greet"));
    assert!(r.contains("::sub::__mingling_type_entry_delete"));
    assert!(r.contains("::sub::__mingling_dispatcher_delete"));
}

#[test]
fn test_dispatcher_clap_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_dispatcher_clap.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required: Vec<&str> = vec![
        // Root: entry types (bare dispatcher_clap, no params)
        "::__mingling_type_entry_clap1",
        "::__mingling_type_entry_clap2",
        "::__mingling_type_entry_clap3",
        "::__mingling_type_entry_clap4",
        // Root: with command name only
        "::__mingling_type_entry_with_cmd",
        // Root: with error
        "::__mingling_type_entry_with_error",
        "::__mingling_type_error_delete",
        // Root: with help
        "::__mingling_type_entry_with_help",
        "::__mingling_help_helpcmd_help",
        // Root: with error + help
        "::__mingling_type_entry_full",
        "::__mingling_type_error_full",
        "::__mingling_help_full_help",
        // Root dispatcher modules
        "::__mingling_dispatcher_greet",
        "::__mingling_dispatcher_delete",
        "::__mingling_dispatcher_helpcmd",
        "::__mingling_dispatcher_full",
        // Sub: entry types (bare dispatcher_clap)
        "::sub::__mingling_type_entry_clap1",
        "::sub::__mingling_type_entry_clap3",
        // Sub: with command name only
        "::sub::__mingling_type_entry_with_cmd",
        // Sub: with error
        "::sub::__mingling_type_entry_with_error",
        "::sub::__mingling_type_error_delete",
        // Sub: with help
        "::sub::__mingling_type_entry_with_help",
        "::sub::__mingling_help_helpcmd_help",
        // Sub dispatcher modules
        "::sub::__mingling_dispatcher_greet",
        "::sub::__mingling_dispatcher_delete",
        "::sub::__mingling_dispatcher_helpcmd",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {}", entry);
    }
}

#[test]
fn test_dispatcher_clap_dispatch_tree() {
    use mingling_pathf::pattern_analyzer;

    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_dispatcher_clap.rs");

    // Dispatchers are always collected at compile time. Each dispatcher now
    // contributes its `__mingling_dispatcher_*` namespace module instead of
    // separate `__Dispatcher*` structs and `__internal_dispatcher_*` statics.
    let r = pattern_analyzer::init().analyze_file(&file).unwrap();
    assert_eq!(r.len(), 26);
    assert!(r.contains("::__mingling_dispatcher_greet"));
    assert!(r.contains("::__mingling_dispatcher_delete"));
    assert!(r.contains("::__mingling_dispatcher_helpcmd"));
    assert!(r.contains("::__mingling_dispatcher_full"));
    assert!(r.contains("::sub::__mingling_dispatcher_greet"));
    assert!(r.contains("::sub::__mingling_dispatcher_delete"));
    assert!(r.contains("::sub::__mingling_dispatcher_helpcmd"));
}

#[test]
fn test_metadata_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_metadata.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required: Vec<&str> = vec![
        // Root: BindType + DataType pairs
        "::__mingling_type_entry_greet1",
        "::Description1",
        "::__mingling_type_entry_greet2",
        "::Description2",
        "::__mingling_type_entry_greet3",
        "::LocalType3",
        "::__mingling_type_entry_greet4",
        "::std::collections::HashMap",
        "::__mingling_type_entry_greet5",
        "::Qualified5",
        // Sub: BindType + DataType pairs
        "::sub::__mingling_type_entry_sub1",
        "::sub::SubType1",
        "::sub::__mingling_type_entry_sub2",
        "::sub::SubType2",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {entry}");
    }
}
