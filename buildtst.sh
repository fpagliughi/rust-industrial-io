#!/bin/bash
#
# This tests building the industrial-io crate with various features
# enabled and disabled. This should be run before any pull requests.


# Extract MSRV from Cargo.toml and ensure it's the full version triplet
# The value is stored in the variable `MSRV`
get_crate_msrv() {
    MSRV=$(awk '/rust-version/ { print substr($3, 2, length($3)-2) }' Cargo.toml)
    local N_DOT
    N_DOT=$(echo "${MSRV}" | grep -o "\." | wc -l | xargs)
    [[ ${N_DOT} == 1 ]] && MSRV="${MSRV}".0
}

printf "Cleaning the crate...\n"
! cargo clean && exit 1
printf "    Ok\n"

printf "\nFormat check...\n"
! cargo +nightly fmt --check && exit 1
printf "    Ok\n"

get_crate_msrv
printf "\nUsing MSRV %s\n" "${MSRV}"

FEATURES="utilities,libiio_v0_25 libiio_v0_25 libiio_v0_24 libiio_v0_23 libiio_v0_21 libiio_v0_19"

for VER in ${MSRV} stable ; do
    printf "\n\nChecking default features for version: %s...\n" "${VER}"
    cargo clean && \
        cargo +${VER} check && \
        cargo +${VER} test && \
        cargo +${VER} clippy && \
        cargo +${VER} doc  # --all-features
    [ "$?" -ne 0 ] && exit 1

    for FEATURE in ${FEATURES}; do
        printf "\n\nBuilding with feature [%s] for version: %s...\n" "${FEATURE}" "${VER}"
        cargo clean && \
            cargo +${VER} check --no-default-features --features="$FEATURE" && \
            cargo +${VER} test --no-default-features --features="$FEATURE" && \
            cargo +${VER} clippy --no-default-features --features="$FEATURE"
        [ "$?" -ne 0 ] && exit 1
    done
done

cargo clean
printf "\n\n*** All builds succeeded ***\n"
