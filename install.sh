#!/usr/bin/bash

cd "$(dirname "$0")"
myfunc() {
	cd -
}
trap 'myfunc' ERR

set -e
cargo build --release
cp -f ~/.cargo-target/release/refactorer ~/.bin

# Disabling for now, since this causes a weird 
# cargo build
# cp -f ~/.cargo-target/debug/ftaggenator ~/.bin
