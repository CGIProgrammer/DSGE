#!/bin/bash
cargo build --release
strip target/release/dsge_vk
upx --lzma --best target/release/dsge_vk
ls -lh target/release/dsge_vk