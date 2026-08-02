#!/bin/bash

cargo doc \
  --manifest-path mingling/Cargo.toml \
  --no-deps \
  --features docs_rs,core,macros,builds,structural_renderer,repl,comp,parser,picker,clap,extra_macros,pathf \
  --open
