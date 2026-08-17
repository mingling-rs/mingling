cargo +nightly rustdoc `
    --manifest-path mingling/Cargo.toml `
    --features docs_rs,core,macros,structural_renderer,repl,comp,picker,clap,extras `
    --open `
    -- `
    --cfg docsrs
