#!/usr/bin/env bash
set -e

ME="${0}"
ME_dir="$(dirname -- "$("${READLINK:-readlink}" -fm -- "${ME}")")"
REPO_main_dir="$(dirname -- "${ME_dir}")"

: ${PROFILE:=debug} # default profile
export PROFILE

path_UUTILS=${path_UUTILS:-${REPO_main_dir}}
path_GNU="${path_GNU:-${path_UUTILS}/../gnu}"

echo "Building uutils tar..."
cd "${path_UUTILS}"
cargo build --profile="${PROFILE}" --bin tarapp

if [[ ! -z  "$CARGO_TARGET_DIR" ]]; then
    UU_BUILD_DIR="${CARGO_TARGET_DIR}/${PROFILE}"
else
    UU_BUILD_DIR="${path_UUTILS}/target/${PROFILE}"
fi

# Symlink tarapp to tar so tests find it as 'tar'
ln -sf "${UU_BUILD_DIR}/tarapp" "${UU_BUILD_DIR}/tar"
echo "Created symlink ${UU_BUILD_DIR}/tar -> tarapp"

# Clone GNU tar if needed
if test ! -d "${path_GNU}/.git"; then
    echo "Cloning GNU tar..."
    git clone --recurse-submodules https://git.savannah.gnu.org/git/tar.git "${path_GNU}"
    cd "${path_GNU}"
    git checkout v1.35
    git submodule update --init --recursive
    
    # Bootstrap requires gnulib and generates the configure script
    ./bootstrap --skip-po
fi

cd "${path_GNU}"

if [ ! -f Makefile ]; then
    echo "Configuring GNU tar..."
    # Configure to build native tar (needed for test suite generation)
    # CFLAGS for build time
    CFLAGS="${CFLAGS} -pipe -O0" ./configure --quiet
fi

echo "Building GNU tar (for test suite)..."
make -j$(nproc)
