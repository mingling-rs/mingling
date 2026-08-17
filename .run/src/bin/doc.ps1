$env:RUSTDOCFLAGS="--html-in-header mingling/arborium-header.html"; cargo doc `
    --manifest-path mingling/Cargo.toml `
    --no-deps `
    --features docs_rs,core,macros,structural_renderer,repl,comp,picker,clap,extras,pathf `
    --open
