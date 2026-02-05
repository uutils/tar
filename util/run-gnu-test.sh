#!/usr/bin/env bash
set -e

# Use GNU version for make, nproc, readlink on *BSD
case "$OSTYPE" in
    *bsd*)
        MAKE="gmake"
        NPROC="gnproc"
        READLINK="greadlink"
        ;;
    *)
        MAKE="make"
        NPROC="nproc"
        READLINK="readlink"
        ;;
esac

ME="${0}"
ME_dir="$(dirname -- "$("${READLINK:-readlink}" -fm -- "${ME}")")"
REPO_main_dir="$(dirname -- "${ME_dir}")"

path_UUTILS=${path_UUTILS:-${REPO_main_dir}}
path_GNU="${path_GNU:-${path_UUTILS}/../gnu}"

# Determine profile
if [[ -d "${path_UUTILS}/target/release" ]]; then
    UU_BUILD_DIR="${path_UUTILS}/target/release"
elif [[ -d "${path_UUTILS}/target/debug" ]]; then
    UU_BUILD_DIR="${path_UUTILS}/target/debug"
else
    echo "Could not find build directory in ${path_UUTILS}/target"
    exit 1
fi

echo "Using uutils tar from: ${UU_BUILD_DIR}"

cd "${path_GNU}"

export RUST_BACKTRACE=1

# The GNU tar testsuite usually looks for 'tar' in the path or uses the one in src/
# We force it to use ours by putting it first in PATH.
export PATH="${UU_BUILD_DIR}:$PATH"
export TAR="${UU_BUILD_DIR}/tar"

echo "Running GNU tar tests..."

# Run with timeout and make check
# We use $* to pass any additional user arguments (e.g. TESTSUITEFLAGS="1-5")
cp "${TAR}" src/tar
timeout -sKILL 4h "${MAKE}" -j "$("${NPROC}")" check "$@"
