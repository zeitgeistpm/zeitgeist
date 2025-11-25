# SPDX-License-Identifier: GPL-3.0-or-later
#!/usr/bin/env bash

RUST_FILES_CHANGED_NOT_DELETED=$(git diff --diff-filter=d --name-only main | grep -E '.*\.rs$')
check-license -w ${RUST_FILES_CHANGED_NOT_DELETED}
