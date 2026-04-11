#!/bin/bash
RUST_BACKTRACE=1 cargo build --target x86_64-pc-windows-gnu
target/x86_64-pc-windows-gnu/debug/desktop.exe