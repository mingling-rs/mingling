// Doc Not Optimize
/// Internal resource for the REPL runtime, used to control the REPL's state during execution
#[derive(Default, Clone)]
pub struct ResREPL {
    /// Marks whether the REPL should exit after the current loop ends
    pub exit: bool,
}
