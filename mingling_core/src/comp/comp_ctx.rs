// Doc Not Optimize
use crate::{COMPLETION_SUBCOMMAND, Program, ProgramCollect};

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Checks whether the program is currently in a completion mode.
    ///
    /// This is determined by checking if the special completion subcommand
    /// (defined by [`COMPLETION_SUBCOMMAND`]) appears among the parsed arguments.
    /// When `true`, the program should generate shell completions instead of
    /// running its normal execution path.
    #[must_use]
    pub fn is_completing(&self) -> bool {
        // Check if the first argument (args[1]) is the completion subcommand
        self.args
            .get(1)
            .is_some_and(|arg| arg == COMPLETION_SUBCOMMAND)
    }
}

#[cfg(test)]
mod tests {
    use crate::MockProgramCollect;

    use super::*;

    #[test]
    fn test_is_completing_with_comp_subcommand() {
        let program: Program<MockProgramCollect> =
            Program::new_with_args(["program", "__comp", "some", "args"]);
        assert!(program.is_completing());
    }

    #[test]
    fn test_is_completing_with_normal_subcommand() {
        let program: Program<MockProgramCollect> =
            Program::new_with_args(["program", "normal", "cmd"]);
        assert!(!program.is_completing());
    }

    #[test]
    fn test_is_completing_with_no_args() {
        let program: Program<MockProgramCollect> = Program::new_with_args(["program"]);
        assert!(!program.is_completing());
    }
}
