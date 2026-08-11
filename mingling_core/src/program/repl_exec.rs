#![allow(unused_imports)]
#![allow(dead_code)]

use std::io::Write;

#[doc(hidden)]
pub mod res;

mod splitter;

use crate::error::{ProgramInternalExecuteError, ProgramPanic};
use crate::program::repl_exec::splitter::split_input_string;
use crate::{Program, ProgramCollect, RenderResult};
use crate::{program::repl_exec::res::ResREPL, this};

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C> + Send + Sync + 'static,
{
    /// Executes the REPL interactive CLI mode.
    ///
    /// This method starts an infinite loop that continuously reads user input, parses commands, executes them,
    /// and displays the execution result or error message. It is suitable for scenarios requiring command-line interaction with the user.
    ///
    /// **Note:** When the `async` feature is enabled, panic unwinding is not supported.
    /// Any panics during command execution will result in an abort rather than being caught and handled gracefully.
    #[might_be_async::func]
    pub fn exec_repl(mut self) {
        // Inject default REPL resource
        self.with_resource(ResREPL::default());

        self.run_hook_repl_on_begin(&crate::hook::HookREPLBeginInfo {});

        might_be_async::select!(
            self.exec_wrapper(async |p| -> () {
                    repl_loop(p).await;
                })
            .await
            else
            self.exec_wrapper(|p| -> () { repl_loop(p); }
            )
        );
    }
}

#[might_be_async::func]
fn repl_loop<C>(p: &'static Program<C>)
where
    C: ProgramCollect<Enum = C> + Send + Sync + 'static,
{
    loop {
        p.run_hook_repl_pre_readline(&crate::hook::HookREPLPreReadlineInfo {});
        let mut readline = p
            .run_hook_repl_readline(&crate::hook::HookREPLReadlineInfo {})
            .unwrap_or_default();
        p.run_hook_repl_post_readline(&crate::hook::HookREPLPostReadlineInfo {
            line: &mut readline,
        });

        let args = split_input_string(&readline);

        p.run_hook_repl_pre_exec(&crate::hook::HookREPLPreExecInfo { args: &args });
        match might_be_async::invoke!(exec_once(p, &args)) {
            Ok(r) => {
                p.run_hook_repl_on_receive_result(&crate::hook::HookREPLOnReceiveResultInfo {
                    result: &r,
                });
            }
            #[allow(unused_variables)]
            Err(ProgramInternalExecuteError::REPLPanic(panic)) => {
                might_be_async::select![
                    {} else {
                        p.run_hook_repl_on_panic(&crate::hook::HookREPLOnPanicInfo { panic: &panic });
                    }
                ];
            }
            _ => {}
        }
        p.run_hook_repl_post_exec(&crate::hook::HookREPLPostExecInfo {});

        if this::<C>().res::<ResREPL>().unwrap().exit {
            p.run_hook_repl_exit(&crate::hook::HookREPLExitInfo {});
            break;
        }

        p.run_hook_repl_loop_once(&crate::hook::HookREPLLoopOnceInfo {});
    }
}

#[cfg(not(feature = "async"))]
fn exec_once<C>(
    p: &'static Program<C>,
    args: &[String],
) -> Result<RenderResult, ProgramInternalExecuteError>
where
    C: ProgramCollect<Enum = C> + Send + Sync + 'static,
{
    #[cfg(panic = "abort")]
    let exec_result = super::exec::exec_with_args(p, &args);

    #[cfg(not(panic = "abort"))]
    let exec_result = {
        let exec_unwind_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::exec::exec_with_args(p, args)
        }));

        match exec_unwind_result {
            Err(panic_info) => {
                let panic_payload = ProgramPanic {
                    payload: panic_info,
                };
                let program = crate::program::THIS_PROGRAM
                    .get_raw()
                    .unwrap()
                    .downcast_ref::<Program<C>>()
                    .unwrap();
                program.run_hook_repl_on_panic(&crate::hook::HookREPLOnPanicInfo {
                    panic: &panic_payload,
                });
                Err(ProgramInternalExecuteError::REPLPanic(panic_payload))
            }
            Ok(r) => r,
        }
    };

    exec_result
}

#[cfg(feature = "async")]
async fn exec_once<C>(
    p: &'static Program<C>,
    args: &[String],
) -> Result<RenderResult, ProgramInternalExecuteError>
where
    C: ProgramCollect<Enum = C> + Send + Sync + 'static,
{
    super::exec::exec_with_args(p, args).await
}
