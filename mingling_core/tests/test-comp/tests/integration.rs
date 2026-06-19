use mingling::MockProgramCollect;
use mingling::Program;
use mingling::comp::{ShellContext, ShellFlag, Suggest, SuggestItem};

#[test]
fn test_shell_context_parsing_full() {
    let args = vec![
        "-f".to_string(),
        "myapp hello ^world".to_string(),
        "-C".to_string(),
        "14".to_string(),
        "-w".to_string(),
        "hello".to_string(),
        "-p".to_string(),
        "myapp".to_string(),
        "-c".to_string(),
        "myapp".to_string(),
        "-i".to_string(),
        "1".to_string(),
        "-F".to_string(),
        "bash".to_string(),
    ];
    let ctx = ShellContext::try_from(args).unwrap();
    assert_eq!(ctx.command_line, "myapp hello -world");
    assert_eq!(ctx.cursor_position, 14);
    assert_eq!(ctx.current_word, "hello");
    assert_eq!(ctx.previous_word, "myapp");
    assert_eq!(ctx.command_name, "myapp");
    assert_eq!(ctx.word_index, 1);
    assert!(matches!(ctx.shell_flag, ShellFlag::Bash));
}

#[test]
fn test_shell_context_parsing_empty() {
    let ctx = ShellContext::try_from(vec![]).unwrap();
    assert!(ctx.all_words.is_empty());
    assert!(matches!(ctx.shell_flag, ShellFlag::Other(_)));
}

#[test]
fn test_suggest_from_vec() {
    let s: Suggest = vec!["--help".to_string(), "--version".to_string()].into();
    match &s {
        Suggest::Suggest(items) => {
            assert_eq!(items.len(), 2);
        }
        _ => panic!("expected Suggest::Suggest"),
    }
}

#[test]
fn test_suggest_item_new() {
    let item = SuggestItem::new("hello".to_string());
    assert_eq!(item.suggest(), "hello");
    assert!(item.description().is_none());
}

#[test]
fn test_suggest_item_with_description() {
    let item = SuggestItem::new_with_desc("hello".to_string(), "a greeting".to_string());
    assert_eq!(item.suggest(), "hello");
    assert_eq!(item.description(), Some(&"a greeting".to_string()));
}

#[test]
fn test_program_is_completing() {
    let program: Program<MockProgramCollect> =
        Program::new_with_args(["myapp", "__comp", "hello", ""]);
    assert!(program.is_completing());
}

#[test]
fn test_program_is_not_completing() {
    let program: Program<MockProgramCollect> = Program::new_with_args(["myapp", "hello"]);
    assert!(!program.is_completing());
}
