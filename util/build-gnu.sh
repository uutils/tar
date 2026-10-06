#!/usr/bin/env bash
set -euo pipefail

ME="${0}"
ME_dir="$(dirname -- "$("${READLINK:-readlink}" -fm -- "${ME}")")"
REPO_main_dir="$(dirname -- "${ME_dir}")"

: "${PROFILE:=debug}" # default profile
export PROFILE

path_UUTILS=${path_UUTILS:-${REPO_main_dir}}
path_GNU="${path_GNU:-${path_UUTILS}/../gnu}"

echo "Building uutils tar..."
cd "${path_UUTILS}"
cargo build --profile="${PROFILE}" --bin tarapp

if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    UU_BUILD_DIR="${CARGO_TARGET_DIR}/${PROFILE}"
else
    UU_BUILD_DIR="${path_UUTILS}/target/${PROFILE}"
fi

# Symlink tarapp to tar so tests find it as 'tar'
ln -svf "${UU_BUILD_DIR}/tarapp" "${UU_BUILD_DIR}/tar"

# Extract GNU tar source if needed
if test ! -f "${path_GNU}/README"; then
    echo "Extracting GNU tar..."
    mkdir -p "${path_GNU}"
    cd "${path_GNU}"
    curl -fsSL https://ftpmirror.gnu.org/tar/tar-1.35.tar.xz -o gnu-tar.tar.xz
    echo e1a200d21f433cd7d917dd979db16919a9167056ae62cf7d038a6118e56b2fe419cd4a396eee66f1f4dc13a8dc380e23f6ffd7ee0ca84e5dd9ad9411f60e002c  gnu-tar.tar.xz \
     | b2sum --check
    tar xJf gnu-tar.tar.xz --strip-components=1
fi

cd "${path_GNU}"

if [ ! -f Makefile ]; then
    echo "Configuring GNU tar..."
    # Configure to build native tar (needed for test suite generation)
    ./configure --quiet
fi

echo "Building GNU tar (for test suite)..."
make -j"$(nproc 2>/dev/null || echo 2)"
