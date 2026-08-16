cargo +nightly rustdoc `
    --manifest-path mingling/Cargo.toml `
    --features docs_rs,core,macros,builds,structural_renderer,repl,comp,picker,clap,extra_macros `
    --open `
    -- `
    --cfg docsrs
