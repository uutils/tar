#!/usr/bin/env bash
set -euo pipefail

ME="${0}"
ME_dir="$(dirname -- "$("${READLINK:-readlink}" -fm -- "${ME}")")"
REPO_main_dir="$(dirname -- "${ME_dir}")"

: "${PROFILE:=debug}" # default profile
export PROFILE

path_UUTILS=${path_UUTILS:-${REPO_main_dir}}
path_GNU="${path_GNU:-${path_UUTILS}/../gnu}"

retry_attempts() {
    local attempts="${GNU_TAR_SETUP_ATTEMPTS:-3}"
    local delay="${GNU_TAR_SETUP_RETRY_DELAY:-30}"
    local description="${1}"
    shift
    local attempt=1
    local status=0

    if [[ ! "${attempts}" =~ ^[1-9][0-9]*$ ]]; then
        echo "GNU_TAR_SETUP_ATTEMPTS must be a positive integer, got: ${attempts}" >&2
        return 1
    fi

    if [[ ! "${delay}" =~ ^[1-9][0-9]*$ ]]; then
        echo "GNU_TAR_SETUP_RETRY_DELAY must be a positive integer, got: ${delay}" >&2
        return 1
    fi

    while (( attempt <= attempts )); do
        if "$@"; then
            return 0
        else
            status=$?
        fi

        if (( attempt == attempts )); then
            echo "Failed to ${description} after ${attempts} attempts." >&2
            return "${status}"
        fi

        echo "Failed to ${description} (attempt ${attempt}/${attempts}); retrying in ${delay}s..." >&2
        sleep "${delay}"
        attempt=$(( attempt + 1 ))
        delay=$(( delay * 2 ))
    done
}

clone_gnu_tar() {
    local attempts="${GNU_TAR_SETUP_ATTEMPTS:-3}"
    local delay="${GNU_TAR_SETUP_RETRY_DELAY:-30}"
    local attempt=1
    local status=0

    if [[ ! "${attempts}" =~ ^[1-9][0-9]*$ ]]; then
        echo "GNU_TAR_SETUP_ATTEMPTS must be a positive integer, got: ${attempts}" >&2
        return 1
    fi

    if [[ ! "${delay}" =~ ^[1-9][0-9]*$ ]]; then
        echo "GNU_TAR_SETUP_RETRY_DELAY must be a positive integer, got: ${delay}" >&2
        return 1
    fi

    if [[ -e "${path_GNU}" ]]; then
        echo "Cannot clone GNU tar: ${path_GNU} exists but is not a git checkout." >&2
        return 1
    fi

    while (( attempt <= attempts )); do
        if git clone --recurse-submodules https://git.savannah.gnu.org/git/tar.git "${path_GNU}"; then
            return 0
        else
            status=$?
        fi

        if (( attempt == attempts )); then
            echo "Failed to clone GNU tar after ${attempts} attempts." >&2
            return "${status}"
        fi

        echo "Failed to clone GNU tar (attempt ${attempt}/${attempts}); retrying in ${delay}s..." >&2
        rm -rf -- "${path_GNU}"
        sleep "${delay}"
        attempt=$(( attempt + 1 ))
        delay=$(( delay * 2 ))
    done
}

echo "Building uutils tar..."
cd "${path_UUTILS}"
cargo build --profile="${PROFILE}" --bin tarapp

if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
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
    clone_gnu_tar
    cd "${path_GNU}"
    git checkout v1.35
    retry_attempts "update GNU tar submodules" git submodule update --init --recursive

    # Bootstrap requires gnulib and generates the configure script
    ./bootstrap --skip-po
fi

cd "${path_GNU}"

if [ ! -f Makefile ]; then
    echo "Configuring GNU tar..."
    # Configure to build native tar (needed for test suite generation)
    ./configure --quiet
fi

echo "Building GNU tar (for test suite)..."
make -j"$(nproc)"
