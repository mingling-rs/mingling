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
    assert!(result.contains("::directly_sub_mod::DirectlySubModStruct"));
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
fn test_pack_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir().unwrap().join("src/test_files/test_pack.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required: Vec<&str> = vec![
        "::ResultPack1",
        "::ErrorPack1",
        "::ErrorPack2",
        "::ResultPack2",
        "::ErrorPack3",
        "::ErrorPack4",
        "::sub::ResultPack1",
        "::sub::ErrorPack1",
        "::sub::ErrorPack2",
        "::sub::ResultPack2",
        "::sub::ErrorPack3",
        "::sub::ErrorPack4",
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
fn test_groupped_derive_analyze() {
    let analyzer = mingling_pathf::pattern_analyzer::init();
    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_groupped_derive.rs");

    let r = analyzer.analyze_file(file).unwrap();
    let required: Vec<&str> = vec![
        "::Derived1",
        "::Derived2",
        "::Derived3",
        "::EnumDerived1",
        "::EnumDerived2",
        "::sub::Derived1",
        "::sub::Derived3",
        "::sub::EnumDerived1",
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
        "::CMDGreet",
        "::EntryRemoteAdd",
        "::CMDRemoteAdd",
        "::EntryDelete",
        "::CMDDelete",
        "::EntryRemoteRm",
        "::CMDRemoteRm",
        "::sub::EntryGreet",
        "::sub::CMDGreet",
        "::sub::EntryDelete",
        "::sub::CMDDelete",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {}", entry);
    }
}

#[test]
fn test_dispatcher_dispatch_tree() {
    use mingling_pathf::config::PathfinderConfig;
    use mingling_pathf::pattern_analyzer;

    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_dispatcher_dispatch_tree.rs");

    // Without dispatch_tree: only Entry + CMD types
    let r1 = pattern_analyzer::init().analyze_file(&file).unwrap();
    // 4 root (EntryGreet, CMDGreet, EntryDelete, CMDDelete)
    // + 4 sub (sub::EntryGreet, sub::CMDGreet, sub::EntryDelete, sub::CMDDelete)
    // = 8
    assert_eq!(r1.len(), 8);
    assert!(r1.contains("::EntryGreet"));
    assert!(r1.contains("::CMDGreet"));
    assert!(r1.contains("::EntryDelete"));
    assert!(r1.contains("::CMDDelete"));
    assert!(r1.contains("::sub::EntryGreet"));
    assert!(r1.contains("::sub::CMDGreet"));

    // With dispatch_tree: Entry + CMD + __internal_dispatcher
    let r2 = pattern_analyzer::init_with_config(PathfinderConfig::with_dispatch_tree())
        .analyze_file(&file)
        .unwrap();
    // 8 (from above) + 2 __internal (root) + 2 __internal (sub) = 12
    assert_eq!(r2.len(), 12);
    assert!(r2.contains("::__internal_dispatcher_greet"));
    assert!(r2.contains("::__internal_dispatcher_delete"));
    assert!(r2.contains("::sub::__internal_dispatcher_greet"));
    assert!(r2.contains("::sub::__internal_dispatcher_delete"));
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
        // Root: with CMD type
        "::EntryWithCmd",
        "::CMDGreet",
        // Root: with CMD + error
        "::EntryWithError",
        "::CMDDelete",
        "::ErrorDelete",
        // Root: with CMD + help
        "::EntryWithHelp",
        "::CMDHelp",
        "::__internal_help_cmdhelp_help",
        // Root: with CMD + error + help
        "::EntryFull",
        "::CMDFull",
        "::ErrorFull",
        "::__internal_help_cmdfull_help",
        // Sub: entry types (bare dispatcher_clap)
        "::sub::EntryClap1",
        "::sub::EntryClap3",
        // Sub: with CMD type
        "::sub::EntryWithCmd",
        "::sub::CMDGreet",
        // Sub: with CMD + error
        "::sub::EntryWithError",
        "::sub::CMDDelete",
        "::sub::ErrorDelete",
        // Sub: with CMD + help
        "::sub::EntryWithHelp",
        "::sub::CMDHelp",
        "::sub::__internal_help_cmdhelp_help",
    ];

    assert_eq!(r.len(), required.len());
    for entry in &required {
        assert!(r.contains(*entry), "Result should contain: {}", entry);
    }
}

#[test]
fn test_dispatcher_clap_dispatch_tree() {
    use mingling_pathf::config::PathfinderConfig;
    use mingling_pathf::pattern_analyzer;

    let file = current_dir()
        .unwrap()
        .join("src/test_files/test_dispatcher_clap.rs");

    // Without dispatch_tree: 26 items (same set as test_dispatcher_clap_analyze)
    let r1 = pattern_analyzer::init().analyze_file(&file).unwrap();
    assert_eq!(r1.len(), 26);

    // With dispatch_tree: 26 + 4 __internal (root) + 3 __internal (sub, no "full") = 33
    let r2 = pattern_analyzer::init_with_config(PathfinderConfig::with_dispatch_tree())
        .analyze_file(&file)
        .unwrap();
    assert_eq!(r2.len(), 33);
    assert!(r2.contains("::__internal_dispatcher_greet"));
    assert!(r2.contains("::__internal_dispatcher_delete"));
    assert!(r2.contains("::__internal_dispatcher_helpcmd"));
    assert!(r2.contains("::__internal_dispatcher_full"));
    assert!(r2.contains("::sub::__internal_dispatcher_greet"));
    assert!(r2.contains("::sub::__internal_dispatcher_delete"));
    assert!(r2.contains("::sub::__internal_dispatcher_helpcmd"));
}
