use crate::THIS_PROGRAM;
use crate::renderer::stream::{RenderStreamState, RenderedRecord};
use crate::{Program, ProgramCollect, RenderResult, error::ProgramExecuteError};

/// Merges inline records captured during execution in front of the final
/// renderer result.
fn merge_captured_records<C>(
    result: Result<RenderResult, ProgramExecuteError>,
) -> Result<RenderResult, ProgramExecuteError>
where
    C: ProgramCollect<Enum = C> + 'static,
{
    let final_result = result?;
    let captured = crate::this::<C>().take_captured_records();

    if captured.is_empty() {
        return Ok(final_result);
    }

    Ok(prepend_records(captured, final_result))
}

fn prepend_records(records: Vec<RenderedRecord>, final_result: RenderResult) -> RenderResult {
    let exit_code = final_result.exit_code;
    let mut merged = RenderResult::default();

    for record in records {
        merged.append_to_buffer(record.content, record.mode);
    }
    merged.append_other(final_result);
    merged.exit_code = exit_code;

    merged
}

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
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
    #[might_be_async::func]
    pub fn exec_without_render(mut self) -> Result<RenderResult, ProgramExecuteError>
    where
        C: 'static + Send + Sync,
    {
        // Run hooks
        self.run_hook_on_begin(&crate::hook::HookBeginInfo {});

        self.args = self.args.iter().skip(1).cloned().collect();
        let mut args = std::mem::take(&mut self.args);

        #[cfg(not(feature = "async"))]
        {
            #[cfg(panic = "abort")]
            return merge_captured_records::<C>(self.exec_wrapper(|p| {
                crate::exec::exec_with_args(p, &mut args).map_err(|e| e.into())
            }));

            #[cfg(not(panic = "abort"))]
            return merge_captured_records::<C>(
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    self.exec_wrapper(|p| {
                        crate::exec::exec_with_args(p, &mut args).map_err(std::convert::Into::into)
                    })
                })) {
                    Ok(result) => result,
                    Err(panic_info) => {
                        let panic_payload = crate::error::ProgramPanic {
                            payload: panic_info,
                        };

                        let program = THIS_PROGRAM
                            .get_raw()
                            .unwrap()
                            .downcast_ref::<Self>()
                            .unwrap();

                        #[cfg(not(feature = "async"))]
                        program.run_hook_exec_panic(&crate::hook::HookPanicInfo {
                            panic: &panic_payload,
                        });

                        Err(ProgramExecuteError::Panic(panic_payload))
                    }
                },
            );
        }

        #[cfg(feature = "async")]
        {
            return merge_captured_records::<C>(
                self.exec_wrapper(|p| async move {
                    crate::exec::exec_with_args(p, &mut args)
                        .await
                        .map_err(Into::into)
                })
                .await,
            );
        }
    }

    /// Run the command line program
    #[must_use]
    #[might_be_async::func]
    pub fn exec(mut self) -> i32
    where
        C: 'static + Send + Sync,
    {
        use crate::error::ProgramExecuteError;

        let stdout_setting = self.stdout_setting.clone();
        self.render_stream = std::sync::Mutex::new(RenderStreamState::Direct {
            enabled: stdout_setting.render_output == crate::config::RenderOutput::Show,
        });

        let result = match might_be_async::invoke!(self.exec_without_render()) {
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
        if stdout_setting.render_output == crate::config::RenderOutput::Show {
            result.std_print();
        }

        result.exit_code
    }

    /// Run the command line program, then exit
    #[might_be_async::func]
    pub fn exec_and_exit(self)
    where
        C: 'static + Send + Sync,
    {
        let exit_code = might_be_async::invoke!(self.exec());
        // SAFETY: exec() is synchronous — it returns only after all
        // chain handlers and renderers have finished. No code still
        // holds references from get_raw() at this point.
        drop(unsafe { THIS_PROGRAM.take() });
        std::process::exit(exit_code)
    }
}

// Async program
#[cfg(feature = "async")]
impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    pub(crate) async fn exec_wrapper<F, Fut>(self, f: F) -> Fut::Output
    where
        C: 'static + Send + Sync,
        F: FnOnce(&'static Self) -> Fut + Send + Sync,
        Fut: Future + Send,
    {
        THIS_PROGRAM.set(Box::new(self));
        let program = THIS_PROGRAM
            .get_raw()
            .unwrap()
            .downcast_ref::<Self>()
            .unwrap();

        f(program).await
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
        F: FnOnce(&'static Self) -> R + Send + Sync,
    {
        THIS_PROGRAM.set(Box::new(self));
        let program = THIS_PROGRAM
            .get_raw()
            .unwrap()
            .downcast_ref::<Self>()
            .unwrap();

        #[cfg(not(panic = "abort"))]
        if program.stdout_setting.silence_panic == super::config::PanicSilence::Silence {
            std::panic::set_hook(Box::new(|_| {}));
        }

        f(program)
    }
}
