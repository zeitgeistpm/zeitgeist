RUST_FILES_CHANGED=$(git diff --name-only main | grep -E .*\.rs$)
check-license -w ${RUST_FILES_CHANGED}
