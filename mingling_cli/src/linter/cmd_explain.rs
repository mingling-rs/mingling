use crate::{Next, eprintln_cargo, linter::registry::ResLintRegistry};
use mingling::{
    Grouped, LazyRes, RenderResult, Routable, ShellContext, Suggest, SuggestItem, Wrap,
    macros::{arg, buffer, chain, completion, dispatcher, metadata, r_println, renderer, routeify},
    metadata::Description,
    picker::EntryPicker,
};

dispatcher!("explain");

#[metadata(EntryExplain)]
pub fn desc_explain() -> Description {
    "Explain the meaning of the specified Lint".into()
}

#[derive(Grouped, Wrap)]
pub struct StateExplainLint(String);

#[derive(Grouped, Default)]
pub struct ErrorNoExplainLintProvided;

#[derive(Grouped, Wrap)]
pub struct ErrorNoSuchLint(String);

#[derive(Debug, Default, Grouped)]
pub struct ResultExplainLint {
    pub lint_name: String,
    pub title: String,
    pub summary: String,
    pub active_on: String,
    pub author: String,
    pub default: String,
}

#[chain(routeify)]
pub fn handle_explain(args: EntryExplain) -> Next {
    let lint_name = args
        .pick_or_route(&arg![String], || ErrorNoExplainLintProvided.to_chain())
        .to_result()?;
    StateExplainLint(lint_name).into()
}

#[chain]
pub fn handle_state_explain_lint(
    p: StateExplainLint,
    registry: &mut LazyRes<ResLintRegistry>,
) -> Next {
    let registry = registry.get_ref();
    let lint_name = p.0;
    let Some(entry) = registry.lints.iter().find(|l| l.name == lint_name) else {
        return ErrorNoSuchLint(lint_name).to_chain();
    };
    ResultExplainLint {
        lint_name: entry.name.clone(),
        title: entry.title.clone(),
        summary: entry.summary.clone(),
        active_on: entry.metadata.active_on.clone(),
        author: entry.metadata.author.clone(),
        default: entry.metadata.default.clone(),
    }
    .to_chain()
}

#[renderer(buffer)]
pub fn render_explain_lint(r: ResultExplainLint) {
    r_println!("{}", r.title);
    r_println!("");
    r_println!("  Name   : #[mlint[{}({})]", r.default, r.lint_name);
    r_println!("  Author : {}", r.author);
    r_println!("");
    r_println!("{}", r.summary);
}

#[renderer]
pub fn render_error_no_explain_lint_provided(_: ErrorNoExplainLintProvided) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "No lint name provided");
    r_println!(r, "");
    r_println!(r, "Usage: mling explain <LINT>");
    r
}

#[renderer]
pub fn render_error_no_such_lint(
    err: ErrorNoSuchLint,
    registry: &mut LazyRes<ResLintRegistry>,
) -> RenderResult {
    let mut r = RenderResult::new();
    let registry = registry.get_ref();
    eprintln_cargo!(r, "No such lint: \"{}\"", err.0);
    r_println!(r, "");
    r_println!(r, "Available lints:");
    for entry in registry.lints.iter() {
        r_println!(r, "  {}", entry.name);
    }
    r
}

#[completion(EntryExplain)]
pub fn complete_explain(ctx: &ShellContext, registry: &mut LazyRes<ResLintRegistry>) -> Suggest {
    let registry = registry.get_ref();
    if ctx.previous_word != "explain" {
        return Suggest::FileCompletion;
    }
    let lints: Vec<String> = registry.lints.iter().map(|l| l.name.clone()).collect();
    let mut suggest = Suggest::new();
    for lint in lints {
        suggest.insert(SuggestItem::Simple(lint));
    }
    suggest
}
