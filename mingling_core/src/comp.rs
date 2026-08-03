mod comp_ctx;
mod flags;
mod shell_ctx;
mod suggest;

use std::collections::BTreeSet;
use std::fmt::Display;

/// Constant defining the name of the completion subcommand.
///
/// When a user invokes this subcommand (e.g., `your_program __comp`), the
/// program enters completion mode and generates shell completions based on
/// the current shell context.
///
/// This value is used internally by the completion system to intercept the
/// command-line input and redirect to the completion handler.
pub const COMPLETION_SUBCOMMAND: &str = "__comp";

#[doc(hidden)]
pub use flags::*;
#[doc(hidden)]
pub use shell_ctx::*;
#[doc(hidden)]
pub use suggest::*;

use crate::{ProgramCollect, debug, only_debug, this, trace};

#[cfg(not(feature = "dispatch_tree"))]
use crate::exec::match_user_input;

/// Trait for implementing completion logic.
///
/// This trait defines the interface for generating command-line completions.
/// Types implementing this trait can provide custom completion suggestions
/// based on the current shell context.
pub trait Completion {
    /// The entry point type that the completion functionality will act on.
    ///
    /// It marks the **previous** type, which typically represents an `EntryXXX` type
    /// (unless you have specific requirements).
    type Previous;

    /// Generates completion suggestions based on the current shell context.
    ///
    /// This method is called when the completion system needs to provide
    /// custom suggestions for the current command or argument. Implementations
    /// should use the provided [`ShellContext`] to determine what to suggest.
    fn comp(ctx: &ShellContext) -> Suggest;
}

/// Trait for extracting user input arguments for completion.
///
/// When the `feat comp` feature is enabled, the `dispatcher!` macro will
/// automatically implement this trait for `Entry` types to extract the
/// arguments from user input for completion suggestions.
pub trait CompletionEntry {
    /// Extracts the user input arguments for completion processing.
    ///
    /// This method is called when the completion system needs to retrieve
    /// the raw input arguments from the user's command line. The returned
    /// vector of strings represents the parsed arguments that will be used
    /// to match against the program's command tree and generate completion
    /// suggestions.
    ///
    /// # Returns
    ///
    /// A vector of strings containing the individual arguments from the
    /// user's command-line input.
    fn get_input(self) -> Vec<String>;
}

/// A helper struct for handling command-line completion logic.
///
/// This struct provides static methods for executing completions based on
/// the current shell context and rendering the resulting suggestions in a
/// format appropriate for the target shell.
pub struct CompletionHelper;
impl CompletionHelper {
    /// Executes the completion logic for the given program type (`P`).
    ///
    /// This is the **entry point** of the Mingling Completion system. It converts
    /// the user's shell context (current command line, cursor position, previous
    /// words, etc.) into actual completion suggestions.
    ///
    /// The function first attempts to match the input arguments against the
    /// program's command tree. If a matching dispatcher is found, it delegates to
    /// the program's custom completion logic (`P::do_comp`). If no match is found
    /// or the dispatcher signals "not found", a default completion is used instead,
    /// which provides subcommand suggestions based on the input path.
    ///
    /// # Type Parameters
    ///
    /// * `P` — The top-level program type that implements [`ProgramCollect`], can be
    ///   displayed, compared for equality, and kept alive for the program's lifetime.
    ///
    /// # Returns
    ///
    /// A [`Suggest`] value containing either file completion instructions or a set
    /// of candidate suggestions.
    #[must_use]
    pub fn exec_completion<P>(ctx: &ShellContext) -> Suggest
    where
        P: ProgramCollect<Enum = P> + Display + PartialEq + 'static + std::fmt::Debug,
    {
        only_debug! {
            crate::debug::init_env_logger();
            trace_ctx(ctx);
        };

        let args = ctx.all_words.iter().skip(1).cloned().collect::<Vec<_>>();
        trace!("arguments=\"{}\"", args.join(", "));

        #[cfg(not(feature = "dispatch_tree"))]
        let program = this::<P>();

        #[cfg(not(feature = "dispatch_tree"))]
        let suggest = if let Ok((dispatcher, args)) = match_user_input(program, &args) {
            trace!(
                "dispatcher matched, dispatcher=\"{}\"",
                dispatcher.node().to_string(),
            );
            let begin = dispatcher.begin(args);
            if let crate::ChainProcess::Ok((any, _)) = begin {
                trace!("entry type: {}", any.member_id);
                let result = P::do_comp(&any, ctx);
                trace!("do_comp result: {:?}", result);
                Some(result)
            } else {
                trace!("begin not Ok");
                None
            }
        } else {
            trace!("no dispatcher matched");
            None
        };
        #[cfg(feature = "dispatch_tree")]
        let suggest = if let Ok(any) = P::dispatch_args_trie(&args) {
            debug!("dispatch_args_trie OK, member_id = {:?}", any.member_id);
            trace!("entry type: {}", any.member_id);

            let entry_fallback = <P::EntryFallback as crate::Grouped<P>>::member_id();

            if entry_fallback == any.member_id {
                debug!("entry_fallback matched");
                trace!("begin not Ok");
                None
            } else {
                let result = P::do_comp(&any, ctx);
                debug!("do_comp result: {:?}", result);
                trace!("do_comp result: {:?}", result);
                Some(result)
            }
        } else {
            debug!("dispatch_args_trie failed, args = {:?}", args);
            trace!("no dispatcher matched");
            None
        };

        match suggest {
            Some(suggest) => {
                trace!("using custom completion: {:?}", suggest);
                suggest
            }
            None => {
                trace!("using default completion");
                default_completion::<P>(ctx)
            }
        }
    }

    /// Renders the completion suggestions to standard output.
    ///
    /// This method takes the [`Suggest`] value produced by
    /// [`exec_completion`], and prints the results in a format appropriate for the
    /// user's shell environment.
    ///
    /// - If the suggestion is [`Suggest::FileCompletion`], it prints `"_file_"` and
    ///   exits, instructing the shell to fall back to file completion.
    /// - If the suggestion is [`Suggest::Suggest`] with a set of candidates, it
    ///   formats and prints them according to the shell type (Zsh/PowerShell, Fish,
    ///   or default).
    pub fn render_suggest<P>(ctx: ShellContext, suggest: Suggest)
    where
        P: ProgramCollect<Enum = P> + Display + 'static,
    {
        trace!("render_suggest called with: {:?}", suggest);
        match suggest {
            Suggest::FileCompletion => {
                trace!("rendering file completion");
                println!("_file_");
                std::process::exit(0);
            }
            Suggest::Suggest(suggestions) => {
                trace!("rendering {} suggestions", suggestions.len());
                match ctx.shell_flag {
                    ShellFlag::Zsh | ShellFlag::Powershell => {
                        trace!("using zsh/pwsh format");
                        print_suggest_with_description(suggestions);
                    }
                    ShellFlag::Fish => {
                        trace!("using fish format");
                        print_suggest_with_description_fish(suggestions);
                    }
                    _ => {
                        trace!("using default format");
                        print_suggest(suggestions);
                    }
                }
            }
        }
    }
}

fn default_completion<P>(ctx: &ShellContext) -> Suggest
where
    P: ProgramCollect<Enum = P> + Display + 'static,
{
    let cmd_nodes: Vec<String> = this::<P>()
        .get_nodes()
        .into_iter()
        .filter(|(s, _)| !s.starts_with('_'))
        .map(|(s, _)| s)
        .collect();
    debug!("cmd_nodes: {:?}", cmd_nodes);

    // If the current position is less than 1, do not perform completion
    if ctx.word_index < 1 {
        debug!("word_index < 1, returning file suggestions");
        return file_suggest();
    }

    // Get the current input path
    let input_end = ctx.word_index.min(ctx.all_words.len());

    debug!(
        "input_path before filter: {:?}",
        &ctx.all_words.get(1..input_end).unwrap_or(&[])
    );

    let input_path: Vec<&str> = ctx
        .all_words
        .get(1..input_end)
        .unwrap_or(&[])
        .iter()
        .filter(|s| !s.is_empty())
        .map(std::string::String::as_str)
        .collect();
    debug!(
        "input_path={:?}, current_word='{}'",
        input_path, ctx.current_word
    );
    debug!("input_path after filter: {:?}", input_path);

    debug!(
        "default_completion: input_path = {:?}, word_index = {}, all_words = {:?}",
        input_path, ctx.word_index, ctx.all_words
    );

    // Filter command nodes that match the input path
    let mut suggestions = Vec::new();

    // Special case: if input_path is empty, return all first-level commands
    if input_path.is_empty() {
        debug!("input_path empty, returning first-level commands");
        for node in cmd_nodes {
            let node_parts: Vec<&str> = node.split(' ').collect();
            if !node_parts.is_empty() && !suggestions.contains(&node_parts[0].to_string()) {
                suggestions.push(node_parts[0].to_string());
            }
        }
    } else {
        debug!("input_path NOT empty, doing next-level suggestions");
        // Get the current word
        let current_word = input_path.last().unwrap();

        // First, handle partial match completion for the current word
        // Only perform current word completion when current_word is not empty
        if input_path.len() == 1 && !ctx.current_word.is_empty() {
            for node in &cmd_nodes {
                let node_parts: Vec<&str> = node.split(' ').collect();
                if !node_parts.is_empty()
                    && node_parts[0].starts_with(current_word)
                    && !suggestions.contains(&node_parts[0].to_string())
                {
                    suggestions.push(node_parts[0].to_string());
                }
            }

            // If suggestions for the current word are found, return directly
            if !suggestions.is_empty() {
                suggestions.sort();
                suggestions.dedup();
                debug!(
                    "default_completion: current word suggestions = {:?}",
                    suggestions
                );
                return suggestions.into();
            }
        }

        // Handle next-level command suggestions
        for node in cmd_nodes {
            let node_parts: Vec<&str> = node.split(' ').collect();

            debug!("Checking node: '{}', parts: {:?}", node, node_parts);

            // If input path is longer than node parts, skip
            if input_path.len() > node_parts.len() {
                continue;
            }

            // Check if input path matches the beginning of node parts
            let mut matches = true;
            for i in 0..input_path.len() {
                if i >= node_parts.len() {
                    matches = false;
                    break;
                }

                if i == input_path.len() - 1 {
                    if !node_parts[i].starts_with(input_path[i]) {
                        matches = false;
                        break;
                    }
                } else if input_path[i] != node_parts[i] {
                    matches = false;
                    break;
                }
            }

            if matches && input_path.len() <= node_parts.len() {
                let last_idx = input_path.len() - 1;
                let is_partial = input_path[last_idx] != node_parts[last_idx];

                if input_path.len() == node_parts.len() {
                    if !ctx.current_word.is_empty() {
                        suggestions.push(node_parts[last_idx].to_string());
                    }
                } else if input_path.len() < node_parts.len() {
                    if is_partial {
                        suggestions.push(node_parts[last_idx].to_string());
                    } else {
                        suggestions.push(node_parts[input_path.len()].to_string());
                    }
                }
            }
        }
    }

    // Remove duplicates and sort
    suggestions.sort();
    suggestions.dedup();

    debug!("default_completion: suggestions = {:?}", suggestions);

    if suggestions.is_empty() {
        file_suggest()
    } else {
        suggestions.into()
    }
}

fn file_suggest() -> Suggest {
    trace!("file_suggest called");
    Suggest::FileCompletion
}

fn print_suggest(suggestions: BTreeSet<SuggestItem>) {
    trace!("print_suggest called with {} items", suggestions.len());
    let mut sorted_suggestions: Vec<SuggestItem> = suggestions.into_iter().collect();
    sorted_suggestions.sort();

    for suggest in sorted_suggestions {
        println!("{}", suggest.suggest());
    }
    std::process::exit(0);
}

fn print_suggest_with_description(suggestions: BTreeSet<SuggestItem>) {
    trace!(
        "print_suggest_with_description called with {} items",
        suggestions.len()
    );
    let mut sorted_suggestions: Vec<SuggestItem> = suggestions.into_iter().collect();
    sorted_suggestions.sort();

    for suggest in sorted_suggestions {
        match suggest.description() {
            Some(desc) => println!("{}$({})", suggest.suggest(), desc),
            None => println!("{}", suggest.suggest()),
        }
    }
    std::process::exit(0);
}

fn print_suggest_with_description_fish(suggestions: BTreeSet<SuggestItem>) {
    trace!(
        "print_suggest_with_description_fish called with {} items",
        suggestions.len()
    );
    let mut sorted_suggestions: Vec<SuggestItem> = suggestions.into_iter().collect();
    sorted_suggestions.sort();

    for suggest in sorted_suggestions {
        match suggest.description() {
            Some(desc) => println!("{}\t{}", suggest.suggest(), desc),
            None => println!("{}", suggest.suggest()),
        }
    }
    std::process::exit(0);
}

#[cfg(feature = "debug")]
fn trace_ctx(ctx: &ShellContext) {
    trace!("=== SHELL CTX BEGIN ===");
    trace!("command_line={}", ctx.command_line);
    trace!("cursor_position={}", ctx.cursor_position);
    trace!("current_word={}", ctx.current_word);
    trace!("previous_word={}", ctx.previous_word);
    trace!("command_name={}", ctx.command_name);
    trace!("word_index={}", ctx.word_index);
    trace!("all_words={:?}", ctx.all_words);
    trace!("shell_flag={:?}", ctx.shell_flag);
    trace!("===  SHELL CTX END  ===");
}

#[cfg(test)]
mod tests {
    use super::COMPLETION_SUBCOMMAND;

    #[test]
    fn completion_subcommand_constant() {
        assert_eq!(COMPLETION_SUBCOMMAND, "__comp");
    }
}
