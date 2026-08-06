use crate::{Next, linter::registry::ResLintRegistry};
use mingling::{
    Grouped, LazyRes, Routable, ShellContext, StructuralData, Suggest, SuggestItem,
    macros::{
        arg, buffer, chain, completion, dispatcher, metadata, pack, pack_err_structural, r_println,
        renderer, routeify,
    },
    metadata::Description,
    picker::EntryPicker,
};
use serde::Serialize;

dispatcher!("explain");

#[metadata(EntryExplain)]
pub fn desc_explain() -> Description {
    "Explain the meaning of the specified Lint".into()
}

pack!(StateExplainLint = String);
pack_err_structural!(ErrorNoExplainLintProvided);
pack_err_structural!(ErrorNoSuchLint = String);

#[derive(Debug, Default, Grouped, StructuralData, Serialize)]
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
        .pick_or_route(&arg![String], || {
            ErrorNoExplainLintProvided::default().to_chain()
        })
        .to_result()?;
    StateExplainLint::new(lint_name).into()
}

#[chain]
pub fn handle_state_explain_lint(
    p: StateExplainLint,
    registry: &mut LazyRes<ResLintRegistry>,
) -> Next {
    let registry = registry.get_ref();
    let lint_name = p.inner;
    let Some(entry) = registry.lints.iter().find(|l| l.name == lint_name) else {
        return ErrorNoSuchLint::new(lint_name).to_chain();
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

#[renderer(buffer)]
pub fn render_error_no_explain_lint_provided(_: ErrorNoExplainLintProvided) {
    r_println!("No lint name provided");
    r_println!("");
    r_println!("Usage: mling explain <LINT>");
}

#[renderer(buffer)]
pub fn render_error_no_such_lint(err: ErrorNoSuchLint, registry: &mut LazyRes<ResLintRegistry>) {
    let registry = registry.get_ref();
    r_println!("No such lint: \"{}\"", err.info);
    r_println!("");
    r_println!("Available lints:");
    for entry in registry.lints.iter() {
        r_println!("  {}", entry.name);
    }
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
