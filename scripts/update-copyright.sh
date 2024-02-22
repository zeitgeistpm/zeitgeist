RUST_FILES_CHANGED_NOT_DELETED=$(git diff --diff-filter=d --name-only main | grep -E '.*\.rs$')
check-license -w ${RUST_FILES_CHANGED_NOT_DELETED}
