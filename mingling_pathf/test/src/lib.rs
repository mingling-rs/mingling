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
        "::sub::__internal_chain_my_chain1",
        "::sub::__internal_chain_my_chain2",
        "::sub::__internal_chain_my_chain3",
        "::sub::__internal_chain_my_chain4",
        "::sub::__internal_chain_my_chain5",
        "::sub::__internal_chain_my_chain6",
        "::__internal_chain_my_chain1",
        "::__internal_chain_my_chain2",
        "::__internal_chain_my_chain3",
        "::__internal_chain_my_chain4",
        "::__internal_chain_my_chain5",
        "::__internal_chain_my_chain6",
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
        "::sub::__internal_renderer_my_renderer1",
        "::sub::__internal_renderer_my_renderer2",
        "::sub::__internal_renderer_my_renderer3",
        "::sub::__internal_renderer_my_renderer4",
        "::__internal_renderer_my_renderer1",
        "::__internal_renderer_my_renderer2",
        "::__internal_renderer_my_renderer3",
        "::__internal_renderer_my_renderer4",
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
        "::sub::__internal_help_my_help1",
        "::sub::__internal_help_my_help2",
        "::sub::__internal_help_my_help3",
        "::sub::__internal_help_my_help4",
        "::__internal_help_my_help1",
        "::__internal_help_my_help2",
        "::__internal_help_my_help3",
        "::__internal_help_my_help4",
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
        "::sub::__internal_completion_my_completion1",
        "::sub::__internal_completion_my_completion2",
        "::sub::__internal_completion_my_completion3",
        "::sub::__internal_completion_my_completion4",
        "::__internal_completion_my_completion1",
        "::__internal_completion_my_completion2",
        "::__internal_completion_my_completion3",
        "::__internal_completion_my_completion4",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {}", entry);
    }
}

#[test]
fn test_group_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir().unwrap().join("src/test_files/test_group.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required: Vec<&str> = vec![
        "::Group1",
        "::GroupAlias1",
        "::Group2",
        "::GroupAlias2",
        "::sub::Group1",
        "::sub::GroupAlias1",
        "::sub::Group2",
        "::sub::GroupAlias2",
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
        "::Derived1",
        "::Derived2",
        "::Derived3",
        "::Derived4",
        "::EnumDerived1",
        "::EnumDerived2",
        "::sub::Derived1",
        "::sub::Derived3",
        "::sub::Derived4",
        "::sub::EnumDerived1",
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
        "::EntryGreet",
        "::__DispatcherGreet",
        "::EntryRemoteAdd",
        "::__DispatcherRemoteAdd",
        "::EntryDelete",
        "::__DispatcherDelete",
        "::EntryRemoteRm",
        "::__DispatcherRemoteRm",
        "::sub::EntryGreet",
        "::sub::__DispatcherGreet",
        "::sub::EntryDelete",
        "::sub::__DispatcherDelete",
        // Dispatchers are always collected at compile time:
        "::__internal_dispatcher_greet",
        "::__internal_dispatcher_remote_add",
        "::__internal_dispatcher_delete",
        "::__internal_dispatcher_remote_rm",
        "::sub::__internal_dispatcher_greet",
        "::sub::__internal_dispatcher_delete",
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

    // Dispatchers are always collected at compile time, so the analyzer
    // always extracts the hidden dispatcher structs and statics too:
    // 8 (Entry + __Dispatcher, root + sub) + 4 __internal (root + sub) = 12
    let r = pattern_analyzer::init().analyze_file(&file).unwrap();
    assert_eq!(r.len(), 12);
    assert!(r.contains("::EntryGreet"));
    assert!(r.contains("::__DispatcherGreet"));
    assert!(r.contains("::EntryDelete"));
    assert!(r.contains("::__DispatcherDelete"));
    assert!(r.contains("::sub::EntryGreet"));
    assert!(r.contains("::sub::__DispatcherGreet"));
    assert!(r.contains("::sub::EntryDelete"));
    assert!(r.contains("::sub::__DispatcherDelete"));
    assert!(r.contains("::__internal_dispatcher_greet"));
    assert!(r.contains("::__internal_dispatcher_delete"));
    assert!(r.contains("::sub::__internal_dispatcher_greet"));
    assert!(r.contains("::sub::__internal_dispatcher_delete"));
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
        "::EntryClap1",
        "::EntryClap2",
        "::EntryClap3",
        "::EntryClap4",
        // Root: with command name only
        "::EntryWithCmd",
        // Root: with error
        "::EntryWithError",
        "::ErrorDelete",
        // Root: with help
        "::EntryWithHelp",
        "::__internal_help_helpcmd_help",
        // Root: with error + help
        "::EntryFull",
        "::ErrorFull",
        "::__internal_help_full_help",
        // Sub: entry types (bare dispatcher_clap)
        "::sub::EntryClap1",
        "::sub::EntryClap3",
        // Sub: with command name only
        "::sub::EntryWithCmd",
        // Sub: with error
        "::sub::EntryWithError",
        "::sub::ErrorDelete",
        // Sub: with help
        "::sub::EntryWithHelp",
        "::sub::__internal_help_helpcmd_help",
        // Hidden dispatcher structs + statics are always collected:
        "::__DispatcherGreet",
        "::__internal_dispatcher_greet",
        "::__DispatcherDelete",
        "::__internal_dispatcher_delete",
        "::__DispatcherHelpcmd",
        "::__internal_dispatcher_helpcmd",
        "::__DispatcherFull",
        "::__internal_dispatcher_full",
        "::sub::__DispatcherGreet",
        "::sub::__internal_dispatcher_greet",
        "::sub::__DispatcherDelete",
        "::sub::__internal_dispatcher_delete",
        "::sub::__DispatcherHelpcmd",
        "::sub::__internal_dispatcher_helpcmd",
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

    // Dispatchers are always collected at compile time:
    // 19 (entry/error/help items) + 14 (hidden structs + statics, root + sub) = 33
    let r = pattern_analyzer::init().analyze_file(&file).unwrap();
    assert_eq!(r.len(), 33);
    assert!(r.contains("::__DispatcherGreet"));
    assert!(r.contains("::__internal_dispatcher_greet"));
    assert!(r.contains("::__DispatcherDelete"));
    assert!(r.contains("::__internal_dispatcher_delete"));
    assert!(r.contains("::__DispatcherHelpcmd"));
    assert!(r.contains("::__internal_dispatcher_helpcmd"));
    assert!(r.contains("::__DispatcherFull"));
    assert!(r.contains("::__internal_dispatcher_full"));
    assert!(r.contains("::sub::__DispatcherGreet"));
    assert!(r.contains("::sub::__internal_dispatcher_greet"));
    assert!(r.contains("::sub::__DispatcherDelete"));
    assert!(r.contains("::sub::__internal_dispatcher_delete"));
    assert!(r.contains("::sub::__DispatcherHelpcmd"));
    assert!(r.contains("::sub::__internal_dispatcher_helpcmd"));
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
        "::EntryGreet1",
        "::Description1",
        "::EntryGreet2",
        "::Description2",
        "::EntryGreet3",
        "::LocalType3",
        "::EntryGreet4",
        "::std::collections::HashMap",
        "::EntryGreet5",
        "::Qualified5",
        // Sub: BindType + DataType pairs
        "::sub::EntrySub1",
        "::sub::SubType1",
        "::sub::EntrySub2",
        "::sub::SubType2",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {entry}");
    }
}
