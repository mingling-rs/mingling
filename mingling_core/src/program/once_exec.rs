use crate::THIS_PROGRAM;
use crate::{Program, ProgramCollect, RenderResult, error::ProgramExecuteError};

// Async program
#[cfg(feature = "async")]
impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    pub(crate) async fn exec_wrapper<F, Fut>(self, f: F) -> Fut::Output
    where
        C: 'static + Send + Sync,
        F: FnOnce(&'static Program<C>) -> Fut + Send + Sync,
        Fut: Future + Send,
    {
        THIS_PROGRAM.set(Box::new(self));
        let program = THIS_PROGRAM
            .get_raw()
            .unwrap()
            .downcast_ref::<Program<C>>()
            .unwrap();

        f(program).await
    }

    /// Run the command line program
    ///
    /// # Errors
    ///
    /// Returns `Err(ProgramExecuteError)` if execution fails,
    /// e.g., if no dispatcher is found or a chain error occurs.
    ///
    /// # Panics
    ///
    /// Panics if the program encounters a non-recoverable internal error.
    pub async fn exec_without_render(mut self) -> Result<RenderResult, ProgramExecuteError>
    where
        C: 'static + Send + Sync,
    {
        // Run hooks
        self.run_hook_on_begin(crate::hook::HookBeginInfo {});

        self.args = self.args.iter().skip(1).cloned().collect();

        return self
            .exec_wrapper(|p| async { crate::exec::exec(p).await.map_err(|e| e.into()) })
            .await;
    }

    /// Run the command line program
    #[must_use]
    pub async fn exec(self) -> i32
    where
        C: 'static + Send + Sync,
    {
        use crate::error::ProgramExecuteError;

        let stdout_setting = self.stdout_setting.clone();
        let result = match self.exec_without_render().await {
            Ok(r) => r,
            Err(e) => match e {
                ProgramExecuteError::DispatcherNotFound => {
                    eprintln!("Dispatcher not found");
                    return 1;
                }
                ProgramExecuteError::RendererNotFound(renderer_name) => {
                    eprintln!("Renderer `{renderer_name}` not found");
                    return 1;
                }
                ProgramExecuteError::Other(e) => {
                    eprintln!("{e}");
                    return 1;
                }
                ProgramExecuteError::Panic(unwinded_error) => {
                    eprintln!("{unwinded_error}");
                    return 1;
                }
            },
        };

        // Read exit code
        // Render result
        if stdout_setting.render_output {
            result.std_print();
        }

        result.exit_code
    }

    /// Run the command line program, then exit
    pub async fn exec_and_exit(self)
    where
        C: 'static + Send + Sync,
    {
        let exit_code = self.exec().await;
        // SAFETY: exec() is synchronous — it returns only after all
        // chain handlers and renderers have finished. No code still
        // holds references from get_raw() at this point.
        drop(unsafe { THIS_PROGRAM.take() });
        std::process::exit(exit_code)
    }
}

// Sync program
#[cfg(not(feature = "async"))]
impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    pub(crate) fn exec_wrapper<F, R>(self, f: F) -> R
    where
        C: 'static + Send + Sync,
        F: FnOnce(&'static Program<C>) -> R + Send + Sync,
    {
        THIS_PROGRAM.set(Box::new(self));
        let program = THIS_PROGRAM
            .get_raw()
            .unwrap()
            .downcast_ref::<Program<C>>()
            .unwrap();

        #[cfg(not(panic = "abort"))]
        if program.stdout_setting.silence_panic {
            std::panic::set_hook(Box::new(|_| {}));
        }

        f(program)
    }

    /// Run the command line program
    ///
    /// # Errors
    ///
    /// Returns `Err(ProgramExecuteError)` if execution fails,
    /// e.g., if no dispatcher is found or a chain error occurs.
    ///
    /// # Panics
    ///
    /// Panics if the program encounters a non-recoverable internal error.
    pub fn exec_without_render(mut self) -> Result<RenderResult, ProgramExecuteError>
    where
        C: 'static + Send + Sync,
    {
        // Run hooks
        self.run_hook_on_begin(crate::hook::HookBeginInfo {});

        self.args = self.args.iter().skip(1).cloned().collect();

        #[cfg(panic = "abort")]
        return self.exec_wrapper(|p| crate::exec::exec(p).map_err(|e| e.into()));

        #[cfg(not(panic = "abort"))]
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.exec_wrapper(|p| crate::exec::exec(p).map_err(std::convert::Into::into))
        })) {
            Ok(result) => result,
            Err(panic_info) => {
                let panic_payload = crate::error::ProgramPanic {
                    payload: panic_info,
                };

                let program = THIS_PROGRAM
                    .get_raw()
                    .unwrap()
                    .downcast_ref::<Program<C>>()
                    .unwrap();

                program.run_hook_exec_panic(crate::hook::HookPanicInfo {
                    panic: &panic_payload,
                });

                Err(ProgramExecuteError::Panic(panic_payload))
            }
        }
    }

    /// Run the command line program
    #[must_use]
    pub fn exec(self) -> i32
    where
        C: 'static + Send + Sync,
    {
        use crate::error::ProgramExecuteError;

        let stdout_setting = self.stdout_setting.clone();
        let result = match self.exec_without_render() {
            Ok(r) => r,
            Err(e) => match e {
                ProgramExecuteError::DispatcherNotFound => {
                    eprintln!("Dispatcher not found");
                    return 1;
                }
                ProgramExecuteError::RendererNotFound(renderer_name) => {
                    eprintln!("Renderer `{renderer_name}` not found");
                    return 1;
                }
                ProgramExecuteError::Other(e) => {
                    eprintln!("{e}");
                    return 1;
                }
                ProgramExecuteError::Panic(unwinded_error) => {
                    eprintln!("{unwinded_error}");
                    return 1;
                }
            },
        };

        // Read exit code
        // Render result
        if stdout_setting.render_output {
            result.std_print();
        }

        result.exit_code
    }

    /// Run the command line program, then exit
    pub fn exec_and_exit(self)
    where
        C: 'static + Send + Sync,
    {
        let exit_code = self.exec();
        // SAFETY: exec() is synchronous — it returns only after all
        // chain handlers and renderers have finished. No code still
        // holds references from get_raw() at this point.
        drop(unsafe { THIS_PROGRAM.take() });
        std::process::exit(exit_code)
    }
}
