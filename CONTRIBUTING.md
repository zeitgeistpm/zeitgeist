# Contributing

## Rules

Please observe the following ground-rules when making contributions:

- **All** contributions must be made by opening a pull request into `main`
- Please add a description to the pull request describing _what_ features you're
  implementing or issues you are solving and _how_
- Commit summaries **must** use the imperative, i.e. "Fix typo" instead of
  "Fixed typo" or "Fixes typo"
- Avoid force pushes after requesting a review
- A pull request may only be merged when it has received at least one approval
  from a member of [zeitgeistpm](https://github.com/zeitgeistpm) and GitHub
  Actions have passed successfully
- Please use _squash and merge_ to merge pull requests; feel free to remove
  trivial items like "Fix typo", etc. from the commit summary

## Style Guide

- Use `rustfmt` to format your contributions
- Avoid panickers like `unwrap()` even if there's proof
- Dispatches that don't use `#[transactional]` macro **must** contain a comment
  including `MARK(non-transactional): ...` followed by a short explanation why
  the dispatch doesn't require `#[transactional]`
