#!/usr/bin/env bash

usage() {
    cat <<EOF
Formats all the rust code and Cargo.toml files in the repository.

usage: format.sh [options]

Options:
    -h, --help      Shows this dialogue
    -c, --check     Check only, exiting with a non-zero exit code if not
                    formatted correctly
    -v, --verbose   Use verbose output
EOF
}

# Global configuration variables, read by all the formatting functions
check=""
verbose=""

check_return_code() {
    if [ $? -eq 0 ]; then
        echo "OK"
    fi
}

cargo_fmt() {
    cargofmt_check=""
    cargofmt_verbose=""

    if [[ ${check} = "check" ]]; then
        cargofmt_check="-- --check"
    fi

    if [[ ${verbose} = "verbose" ]]; then
        cargofmt_verbose="--verbose"
    fi

    echo "Running cargo formatting ..."

    cargo fmt --all ${cargofmt_verbose} ${cargofmt_check}

    check_return_code
}

taplo_fmt() {
    taplo_verbose=""
    if [[ ${verbose} = "verbose" ]]; then
        taplo_verbose="--verbose"
    fi

    echo "Running taplo formatting ..."

    if [[ ${check} = "check" ]]; then
        taplo fmt --check ${taplo_verbose}
    else
        taplo fmt ${taplo_verbose}
    fi

    check_return_code
}

taplo_validate() {
    taplo_verbose=""
    if [[ ${verbose} = "verbose" ]]; then
        taplo_verbose="--verbose"
    fi

    echo "Running taplo validation ..."

    taplo check ${taplo_verbose}

    check_return_code
}

# install taplo if it isn't already
has_taplo=$(whereis taplo)
if [[ ${has_taplo} = "taplo: " ]]; then
    cargo install taplo-cli 2>/dev/null
fi

for arg in "$@"; do
    case $arg in
    "--help" | "-h")
        usage
        exit 0
        ;;
    "--check" | "-c")
        check="check"
        ;;
    "--verbose" | "-v")
        verbose="verbose"
        ;;
    *)
        echo "Unknown option '$arg'"
        usage
        exit 1
        ;;
    esac
done

cargo_fmt
taplo_validate
taplo_fmt
