#!/usr/bin/env bash

# Auxiliar functions used by the testing scripts

check_package_with_feature() {
    local package=$1
    local features=$2

    /bin/echo -e "\e[0;33m***** Checking '$package' with features '$features' *****\e[0m\n"
    cargo check --features $features --manifest-path $package/Cargo.toml --no-default-features
}

test_package_with_feature() {
    local package=$1
    local features=$2

    /bin/echo -e "\e[0;33m***** Testing '$package' with features '$features' *****\e[0m\n"
    cargo test --features $features --manifest-path $package/Cargo.toml --no-default-features
}
