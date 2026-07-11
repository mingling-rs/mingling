#![allow(clippy::borrowed_box)]

use crate::{
    AnyOutput, ChainProcess, Dispatcher, NextProcess, Program, ProgramCollect, RenderResult,
    error::ProgramInternalExecuteError, hook::ProgramControls,
};

#[doc(hidden)]
pub mod error;

#[cfg(feature = "async")]
pub async fn exec<C>(
    program: &'static Program<C>,
) -> Result<RenderResult, ProgramInternalExecuteError>
where
    C: ProgramCollect<Enum = C>,
{
    exec_with_args(program, &program.args).await
}

#[cfg(not(feature = "async"))]
pub fn exec<C>(program: &'static Program<C>) -> Result<RenderResult, ProgramInternalExecuteError>
where
    C: ProgramCollect<Enum = C>,
{
    exec_with_args(program, &program.args)
}

#[cfg(feature = "async")]
pub async fn exec_with_args<C>(
    program: &'static Program<C>,
    args: &[String],
) -> Result<RenderResult, ProgramInternalExecuteError>
where
    C: ProgramCollect<Enum = C>,
{
    // Exit code
    let mut exit_code: i32 = 0;

    macro_rules! control {
        ($hook_call:expr, $current:expr) => {
            let __ccc = $hook_call;
            if let Some(r) =
                handle_program_control(program, __ccc, Some(&mut $current), &mut exit_code)
            {
                return Ok(r);
            }
        };
        ($hook_call:expr) => {
            let __ccc = $hook_call;
            if let Some(r) = handle_program_control(program, __ccc, None, &mut exit_code) {
                return Ok(r);
            }
        };
    }

    // Current
    let mut current = C::build_dispatcher_not_found(vec![]);

    // Run hooks
    control!(
        program.run_hook_pre_dispatch(crate::hook::HookPreDispatchInfo { arguments: args }),
        current
    );

    // Dispatch args - either via dynamic dispatch or trie dispatch based on feature flag
    let mut current = if cfg!(not(feature = "dispatch_tree")) {
        dispatch_args_dynamic(program, args)?
    } else {
        C::dispatch_args_trie(args)?
    };

    // Run hook
    control!(
        program.run_hook_post_dispatch(crate::hook::HookPostDispatchInfo {
            entry: &current.member_id,
        }),
        current
    );

    let mut stop_next = false;

    // If the program has Help enabled, skip actual logic and jump to Help
    if program.user_context.help {
        let mut render_result = render_help::<C>(program, current);

        // Run hook
        control!(program.run_hook_finish(crate::hook::HookFinishInfo {}));
        render_result.exit_code = exit_code;

        return Ok(render_result);
    }

    loop {
        let final_exec = stop_next;

        current = {
            // If a chain exists, execute as a chain
            if C::has_chain(&current) {
                // Run hook
                control!(
                    program.run_hook_pre_chain(crate::hook::HookPreChainInfo {
                        input: &current.member_id,
                        raw: current.inner.as_ref(),
                    }),
                    current
                );

                match C::do_chain(current).await {
                    ChainProcess::Ok((any, NextProcess::Renderer)) => {
                        {
                            let mut render_result = render::<C>(program, any);

                            // Run hook
                            control!(program.run_hook_finish(crate::hook::HookFinishInfo {}));
                            render_result.exit_code = exit_code;

                            return Ok(render_result);
                        };
                    }
                    ChainProcess::Ok((mut any, NextProcess::Chain)) => {
                        // Run hook
                        control!(
                            program.run_hook_post_chain(crate::hook::HookPostChainInfo {
                                output: &any
                            }),
                            any
                        );
                        any
                    }
                    ChainProcess::Err(e) => {
                        // Run hook
                        control!(
                            program.run_hook_finish(crate::hook::HookFinishInfo {}),
                            &mut C::build_empty_result()
                        );
                        return Err(e.into());
                    }
                }
            }
            // If no chain exists, attempt to render
            else if C::has_renderer(&current) {
                // Run hook
                control!(
                    program.run_hook_pre_render(crate::hook::HookPreRenderInfo {
                        input: &current.member_id,
                        raw: current.inner.as_ref(),
                    }),
                    current
                );

                let mut render_result = render::<C>(program, current);

                // Run hooks
                control!(
                    program.run_hook_post_render(crate::hook::HookPostRenderInfo {
                        result: &render_result,
                    })
                );

                control!(program.run_hook_finish(crate::hook::HookFinishInfo {}));

                render_result.exit_code = exit_code;
                return Ok(render_result);
            }
            // No renderer exists
            else {
                stop_next = true;
                C::build_renderer_not_found(current.member_id)
            }
        };

        if final_exec && stop_next {
            break;
        }
    }
    let mut render_result = RenderResult::default();

    // Run hook
    control!(
        program.run_hook_finish(crate::hook::HookFinishInfo {}),
        current
    );
    render_result.exit_code = exit_code;
    Ok(render_result)
}

#[cfg(not(feature = "async"))]
pub fn exec_with_args<C>(
    program: &'static Program<C>,
    args: &[String],
) -> Result<RenderResult, ProgramInternalExecuteError>
where
    C: ProgramCollect<Enum = C>,
{
    // Exit code
    let mut exit_code: i32 = 0;

    macro_rules! control {
        ($hook_call:expr, $current:expr) => {
            let __ccc = $hook_call;
            if let Some(r) =
                handle_program_control(program, __ccc, Some(&mut $current), &mut exit_code)
            {
                return Ok(r);
            }
        };
        ($hook_call:expr) => {
            let __ccc = $hook_call;
            if let Some(r) = handle_program_control(program, __ccc, None, &mut exit_code) {
                return Ok(r);
            }
        };
    }

    // Current
    let mut current = C::build_dispatcher_not_found(vec![]);

    // Run hooks
    control!(
        program.run_hook_pre_dispatch(crate::hook::HookPreDispatchInfo { arguments: args }),
        current
    );

    // Dispatch args - either via dynamic dispatch or trie dispatch based on feature flag
    let mut current = if cfg!(not(feature = "dispatch_tree")) {
        dispatch_args_dynamic(program, args)?
    } else {
        C::dispatch_args_trie(args)?
    };

    // Run hook
    control!(
        program.run_hook_post_dispatch(crate::hook::HookPostDispatchInfo {
            entry: &current.member_id,
        }),
        current
    );

    let mut stop_next = false;

    // If the program has Help enabled, skip actual logic and jump to Help
    if program.user_context.help {
        let mut render_result = render_help::<C>(program, current);

        // Run hook
        control!(program.run_hook_finish(crate::hook::HookFinishInfo {}));
        render_result.exit_code = exit_code;

        return Ok(render_result);
    }

    loop {
        let final_exec = stop_next;

        current = {
            // If a chain exists, execute as a chain
            if C::has_chain(&current) {
                // Run hook
                control!(
                    program.run_hook_pre_chain(crate::hook::HookPreChainInfo {
                        input: &current.member_id,
                        raw: current.inner.as_ref(),
                    }),
                    current
                );

                match C::do_chain(current) {
                    ChainProcess::Ok((any, NextProcess::Renderer)) => {
                        {
                            let mut render_result = render::<C>(program, any);

                            // Run hook
                            control!(program.run_hook_finish(crate::hook::HookFinishInfo {}));
                            render_result.exit_code = exit_code;

                            return Ok(render_result);
                        };
                    }
                    ChainProcess::Ok((mut any, NextProcess::Chain)) => {
                        // Run hook
                        control!(
                            program.run_hook_post_chain(crate::hook::HookPostChainInfo {
                                output: &any
                            }),
                            any
                        );
                        any
                    }
                    ChainProcess::Err(e) => {
                        // Run hook
                        control!(
                            program.run_hook_finish(crate::hook::HookFinishInfo {}),
                            &mut C::build_empty_result()
                        );
                        return Err(e.into());
                    }
                }
            }
            // If no chain exists, attempt to render
            else if C::has_renderer(&current) {
                // Run hook
                control!(
                    program.run_hook_pre_render(crate::hook::HookPreRenderInfo {
                        input: &current.member_id,
                        raw: current.inner.as_ref(),
                    }),
                    current
                );

                let mut render_result = render::<C>(program, current);

                // Run hooks
                control!(
                    program.run_hook_post_render(crate::hook::HookPostRenderInfo {
                        result: &render_result,
                    })
                );

                control!(program.run_hook_finish(crate::hook::HookFinishInfo {}));

                render_result.exit_code = exit_code;
                return Ok(render_result);
            }
            // No renderer exists
            else {
                stop_next = true;
                C::build_renderer_not_found(current.member_id)
            }
        };

        if final_exec && stop_next {
            break;
        }
    }
    let mut render_result = RenderResult::default();

    // Run hook
    control!(
        program.run_hook_finish(crate::hook::HookFinishInfo {}),
        current
    );
    render_result.exit_code = exit_code;
    Ok(render_result)
}

/// Dynamically dispatch input arguments to registered entry types
pub(crate) fn dispatch_args_dynamic<C>(
    program: &'static Program<C>,
    args: &[String],
) -> Result<AnyOutput<C>, ProgramInternalExecuteError>
where
    C: ProgramCollect<Enum = C>,
{
    let next = match match_user_input(program, args) {
        Ok((dispatcher, args)) => {
            // Entry point
            match dispatcher.begin(args) {
                ChainProcess::Ok((any, _)) => any,
                ChainProcess::Err(e) => return Err(e.into()),
            }
        }
        Err(ProgramInternalExecuteError::DispatcherNotFound) => {
            // No matching Dispatcher is found
            C::build_dispatcher_not_found(args.to_vec())
        }
        Err(e) => return Err(e),
    };
    Ok(next)
}

/// Match user input against registered dispatchers and return the matched dispatcher and remaining arguments.
#[allow(clippy::type_complexity)]
pub(crate) fn match_user_input<C>(
    program: &'static Program<C>,
    args: &[String],
) -> Result<(&'static (dyn Dispatcher<C> + Send + Sync), Vec<String>), ProgramInternalExecuteError>
where
    C: ProgramCollect<Enum = C>,
{
    let nodes = program.get_nodes();
    let command = format!("{} ", args.join(" "));

    // Find all nodes that match the command prefix
    let matching_nodes: Vec<&(String, &(dyn Dispatcher<C> + Send + Sync))> = nodes
        .iter()
        // Also add a space to the node string to ensure consistent matching logic
        .filter(|(node_str, _)| command.starts_with(&format!("{node_str} ")))
        .collect();

    match matching_nodes.len() {
        0 => {
            // No matching node found
            Err(ProgramInternalExecuteError::DispatcherNotFound)
        }
        1 => {
            let matched_prefix = matching_nodes[0];
            let prefix_len = matched_prefix.0.split_whitespace().count();
            let trimmed_args: Vec<String> = args.iter().skip(prefix_len).cloned().collect();
            Ok((matched_prefix.1, trimmed_args))
        }
        _ => {
            // Multiple matching nodes found
            // Find the node with the longest length (most specific match)
            let matched_prefix = matching_nodes
                .iter()
                .max_by_key(|node| node.0.len())
                .unwrap();

            let prefix_len = matched_prefix.0.split_whitespace().count();
            let trimmed_args: Vec<String> = args.iter().skip(prefix_len).cloned().collect();
            Ok((matched_prefix.1, trimmed_args))
        }
    }
}

#[inline]
pub(crate) fn handle_program_control<C: ProgramCollect<Enum = C>>(
    program: &Program<C>,
    controls: ProgramControls<C>,
    mut current: Option<&mut AnyOutput<C>>,
    exit_code: &mut i32,
) -> Option<RenderResult> {
    for unit in controls.into_iter() {
        match unit {
            super::hook::ProgramControlUnit::OverrideExitCode(c) => *exit_code = c,
            super::hook::ProgramControlUnit::RouteToChain(any_output) => {
                if let Some(ref mut current) = current {
                    **current = any_output;
                }
            }
            super::hook::ProgramControlUnit::RouteToRender(any_output) => {
                // Note: Hooks triggered by ProgramControl will not trigger ProgramControl again

                // Pre render
                let _ = program.run_hook_pre_render(crate::hook::HookPreRenderInfo {
                    input: &any_output.member_id,
                    raw: any_output.inner.as_ref(),
                });

                let mut r = render::<C>(program, any_output);
                r.exit_code = *exit_code;

                // Post render
                program.run_hook_post_render(crate::hook::HookPostRenderInfo { result: &r });

                return Some(r);
            }
            super::hook::ProgramControlUnit::RouteToHelp(any_output) => {
                let mut r = render_help::<C>(program, any_output);
                r.exit_code = *exit_code;
                return Some(r);
            }
        }
    }
    None
}

#[inline]
#[allow(unused_variables)]
fn render<C: ProgramCollect<Enum = C>>(program: &Program<C>, any: AnyOutput<C>) -> RenderResult {
    #[cfg(not(feature = "structural_renderer"))]
    {
        C::render(any)
    }
    #[cfg(feature = "structural_renderer")]
    {
        #[allow(unreachable_patterns)]
        match program.structural_renderer_name {
            super::StructuralRendererSetting::Disable => {
                C::render(any)
            }
            _ => C::structural_render(any, &program.structural_renderer_name).unwrap(),
        }
    }
}

#[inline]
#[allow(unused_variables)]
fn render_help<C: ProgramCollect<Enum = C>>(
    program: &Program<C>,
    entry: AnyOutput<C>,
) -> RenderResult {
    #[cfg(not(feature = "structural_renderer"))]
    {
        C::render_help(entry)
    }
    #[cfg(feature = "structural_renderer")]
    {
        #[allow(unreachable_patterns)]
        match program.structural_renderer_name {
            super::StructuralRendererSetting::Disable => {
                C::render_help(entry)
            }
            _ => RenderResult::default(),
        }
    }
}
