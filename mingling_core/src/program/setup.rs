use crate::{ProgramCollect, program::Program};

pub trait ProgramSetup<C>
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>);
}

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Load and execute init logic
    pub fn with_setup<S: ProgramSetup<C> + 'static>(&mut self, setup: S) {
        S::setup(setup, self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockProgramCollect;

    struct TestSetup {
        called: std::rc::Rc<std::cell::Cell<bool>>,
    }

    impl ProgramSetup<MockProgramCollect> for TestSetup {
        fn setup(self, _program: &mut Program<MockProgramCollect>) {
            self.called.set(true);
        }
    }

    #[test]
    fn test_with_setup_calls_setup() {
        let called = std::rc::Rc::new(std::cell::Cell::new(false));
        let setup = TestSetup {
            called: std::rc::Rc::clone(&called),
        };
        let mut program: Program<MockProgramCollect> = Program::new_with_args(["test"]);
        program.with_setup(setup);
        assert!(called.get());
    }
}
