#!/bin/bash

cargo doc --workspace --no-deps --features builds,structural_renderer,repl,comp,parser,picker,clap,extra_macros --open
