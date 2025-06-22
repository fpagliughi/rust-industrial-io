#!/bin/bash
#
# This tests building the industrial-io crate with various features
# enabled and disabled. This should be run before any pull requests.
#
# Note, in order for unit tests to pass, the IIO dummy device should be
# loaded in the Linux kernel.
#

# Extract MSRV from Cargo.toml and ensure it's the full version triplet
# The value is stored in the variable `MSRV`
get_crate_msrv() {
    MSRV=$(awk '/rust-version/ { print substr($3, 2, length($3)-2) }' Cargo.toml)
    local N_DOT
    N_DOT=$(echo "${MSRV}" | grep -o "\." | wc -l | xargs)
    [[ ${N_DOT} == 1 ]] && MSRV="${MSRV}".0
}

# Check that that the iio_dummy kernel module is loaded
if [ -z "$(lsmod | grep ^iio_dummy)" ]; then
    printf "The 'iio_dummy' kernel module should be loaded for the unit tests\n"
    exit 1
fi

printf "Cleaning the crate...\n"
! cargo clean && exit 1
printf "    Ok\n"

printf "\nFormat check...\n"
! cargo +nightly fmt --check --all && exit 1
printf "    Ok\n"

printf "\nCheck for typos...\n"
! typos && exit 1

get_crate_msrv
printf "\nUsing MSRV %s\n" "${MSRV}"

FEATURES="utils,libiio_v0_25 libiio_v0_25 libiio_v0_24 libiio_v0_23 libiio_v0_21 libiio_v0_19"

for VER in "${MSRV}" stable ; do
    printf "\n\nChecking default features for version: %s...\n" "${VER}"
    cargo clean && \
        cargo +"${VER}" check --all-targets && \
        cargo +"${VER}" test && \
        cargo +"${VER}" doc  # --all-features
    [ "$?" -ne 0 ] && exit 1

    for FEATURE in ${FEATURES}; do
        printf "\n\nBuilding with feature [%s] for version: %s...\n" "${FEATURE}" "${VER}"
        cargo clean && \
            cargo +"${VER}" check --no-default-features --features="$FEATURE" && \
            cargo +"${VER}" test --no-default-features --features="$FEATURE" && \
        [ "$?" -ne 0 ] && exit 1
    done
done

printf "\nChecking clippy for version: %s...\n" "${MSRV}"
cargo clean
! cargo +"${MSRV}" clippy --no-deps --all-targets -- -D warnings && exit 1

cargo clean
printf "\n\n*** All builds succeeded ***\n"
