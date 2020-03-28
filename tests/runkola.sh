#!/bin/bash
set -euo pipefail
top=$(git rev-parse --show-toplevel)
cd ${top}
cosabuild=${COSA_BUILDDIR:-cosabuild}
if ! [ -d "${cosabuild}" ]; then
    echo "COSA_BUILDDIR=${COSA_BUILDDIR} invalid"
    exit 1
fi
cd tests/inst && cargo build --release
# Hook into the kola dir
ln -f target/release/ostree-test ../kola/nondestructive/insttest-rs
cd ${top}/${cosabuild}
if [ -z "$@" ]; then
    set -- 'ext.ostree.*' "$@"
fi
exec cosa kola run -E ${top} "$@"
