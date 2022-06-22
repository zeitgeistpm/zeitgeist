# Contributing

## Rules

Please observe the following ground-rules when making contributions:

-   **All** contributions must be made by opening a pull request into `main`
-   Please add a description to the pull request describing _what_ features
    you're implementing or issues you are solving and _how_; add background
    information whenever possible
-   Please link all issues and other GitHub items that are related to the
    changes that you're making
-   Commit summaries must use the imperative, i.e. "Fix typo" instead of "Fixed
    typo" or "Fixes typo"
-   Avoid force pushes when a pull request is in review
-   A pull request may only be merged when it has received at least one approval
    from a member of [zeitgeistpm] and GitHub Actions have passed successfully;
    whenever possible, the author should execute the merge
-   Comments in a review should be resolved by the author
-   Please use _squash and merge_ to merge pull requests; feel free to remove
    trivial items like "Fix typo", etc. from the commit summary

## Labels

Labels are used to denote status (`s:*`) and priority (`p:*`) of pull requests.

-   Every pull request may not have more than one status label
-   Pull requests should be marked with the
    ![label: Content](https://img.shields.io/github/labels/zeitgeistpm/zeitgeist/s:in-progress)
    label until they are ready for review
-   Pull requests that are ready for review should be marked with
    ![label: Content](https://img.shields.io/github/labels/zeitgeistpm/zeitgeist/s:review-needed)
-   If a reviewer requests changes, they should use the
    ![label: Content](https://img.shields.io/github/labels/zeitgeistpm/zeitgeist/s:revision-needed)
-   Once a pull request is approved (see [Rules] above), it should be marked
    with
    ![label: Content](https://img.shields.io/github/labels/zeitgeistpm/zeitgeist/s:accepted)
-   Priority labels are placed by [zeitgeistpm] members

The `t:*` labels are used to specify types of issues. Their use is placed at the
discretion of [zeitgeistpm] members.

## Style Guide

-   Use `rustfmt` to format your contributions
-   Avoid panickers like `unwrap()` even if there's proof that they are
    infallible
-   Dispatches that don't use `#[transactional]` macro **must** contain a
    comment including `MARK(non-transactional): ...` followed by a short
    explanation why the dispatch doesn't require `#[transactional]`

[rules]: #Rules
[zeitgeistpm]: https://github.com/zeitgeistpm
