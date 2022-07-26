# Contributing

## Rules

Please observe the following ground-rules when making contributions:

- **All** contributions must be made by opening a pull request into `main`.
- Please add a description to the pull request describing _what_ features you're
  implementing or issues you are solving and _how_. Add background information
  whenever possible.
- Please link all issues and other GitHub items that are related to the changes
  that you're making.
- Commit summaries must use the imperative, i.e. "Fix typo" instead of "Fixed
  typo" or "Fixes typo".
- Avoid force pushes when a pull request is in review.
- A pull request may only be merged when it has received at least one approval
  from a member of [zeitgeistpm] and GitHub Actions have passed successfully.
  Whenever possible, the author should execute the merge.
- Comments in a review should be resolved by the author.
- Please use _squash and merge_ to merge pull requests. Feel free to remove
  trivial items like "Fix typo", etc. from the commit summary.
- Don't modify `RuntimeVersion` under normal circumstances. Changes to
  RuntimeVersion are made in a specific PR at the end of the release cycle.
- Summarize any interface changes in docs/changelog_for_devs.md and format the
  file using `prettier -w docs/changelog_for_devs.md`.

## Labels

Labels are used to denote status (`s:*`) and priority (`p:*`) of pull requests.

- Every pull request may not have more than one status label.
- Pull requests should be marked with the
  ![label: Content](https://img.shields.io/github/labels/zeitgeistpm/zeitgeist/s:in-progress)
  label until they are ready for review.
- Pull requests that are ready for review should be marked with
  ![label: Content](https://img.shields.io/github/labels/zeitgeistpm/zeitgeist/s:review-needed).
- If a reviewer requests changes, they should use the
  ![label: Content](https://img.shields.io/github/labels/zeitgeistpm/zeitgeist/s:revision-needed).
- Once a pull request is approved (see [Rules] above), it should be marked with
  ![label: Content](https://img.shields.io/github/labels/zeitgeistpm/zeitgeist/s:accepted)
- Priority labels are placed by [zeitgeistpm] members.

The `t:*` labels are used to specify types of issues. Their use is placed at the
discretion of [zeitgeistpm] members.

If the changes made in a PR require a change to the substrate `RuntimeVersion`,
mark the PR with the following labels according to
[RuntimeVersion](https://docs.rs/sp-version/latest/sp_version/struct.RuntimeVersion.html):
![label: Content](https://img.shields.io/github/labels/zeitgeistpm/zeitgeist/i:authorship-interface-changed%20:warning:),
![label: Content](https://img.shields.io/github/labels/zeitgeistpm/zeitgeist/i:spec-changed%20:warning:),
![label: Content](https://img.shields.io/github/labels/zeitgeistpm/zeitgeist/i:transactions-changed%20:warning:).

## Style Guide

- Use `rustfmt` to format your contributions.
- Avoid panickers like `unwrap()` even if there's proof that they are infallible.
- Dispatches that don't use `#[transactional]` macro **must** contain a comment
  including `MARK(non-transactional): ...` followed by a short explanation why
  the dispatch doesn't require `#[transactional]`.
- Functions are written in snake case, i.e. `my_function`, anything else is
  declared in CamelCase (starting with a capital first letter).
- Indentations consist of spaces, unless the language used requires tabs.
- Anything that is publicly visible must be documented. This encompasses but is
  not limited to whole crates (top level documentation), public types and
  functions, dispatachble functions (functions that can be called by
  transactions), the `Error` and `Event` enum as well as the `Config` trait.
- Any newly added or modified functionality must be subject to at least one test
  case. A full code coverage is the targeted goal.

[rules]: #Rules
[zeitgeistpm]: https://github.com/zeitgeistpm
