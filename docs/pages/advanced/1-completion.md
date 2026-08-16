<h1 align="center">Completion</h1>
<p align="center">
    Fully dynamic completion system via the `comp` feature
</p>

Mingling's completion is **fully dynamic** — no static completion files, suggestions are computed at runtime based on the user's current input.

## Enable `comp`

```toml
# Cargo.toml
[dependencies.mingling]
features = ["comp"]
 
[build-dependencies.mingling]
features = [
    "comp",
    # Enable `build` for build-time support
    "build"
]
```
 
## How it works

When the user presses `TAB`, the completion script calls the program's hidden subcommand `__comp`, which dynamically queries the best suggestions based on the provided `ShellContext`.

Completion flow:

1. Re-match the user's current input to a `Dispatcher`
2. Call the corresponding `#[completion]` function
3. The function returns a `Suggest` (file completion or a list of suggestions)
4. Notify the shell to display the suggestions

## Define completions

Use `#[completion(EntryType)]` to define completion logic for an Entry:

```rust
// Features: ["comp"]
@@@use mingling::prelude::*;
@@@use mingling::{ShellContext, Suggest, SuggestItem};
@@@use std::collections::BTreeSet;
@@@dispatcher!("greet", EntryGreet);
 
#[completion(EntryGreet)]
fn complete_greet(ctx: &ShellContext) -> Suggest {
    if ctx.previous_word == "greet" {
        let mut items = BTreeSet::new();
        items.insert(SuggestItem::new_with_desc("Alice".into(), "Likes to receive messages".into()));
        items.insert(SuggestItem::new("World".into()));
        Suggest::Suggest(items)
    } else {
        Suggest::FileCompletion
    }
}
```
 
The `suggest!` macro is a more concise way to write the same thing:

```rust
// Features: ["comp"]
@@@use mingling::macros::suggest;
@@@fn example() {
suggest! {
    "Alice": "Likes to receive messages",
    "World"
};
@@@}
```
 
`ShellContext` holds the user's current input state (`previous_word`, `current_word`, `all_words`, etc.). `Suggest` has two variants: `Suggest::Suggest(list)` returns a suggestion list, `Suggest::FileCompletion` delegates file completion to the shell.

## Generate completion scripts

Call `build_comp_scripts` in `build.rs` to generate completion scripts (requires `builds` + `comp` features).

See [example-completion](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-completion).

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
