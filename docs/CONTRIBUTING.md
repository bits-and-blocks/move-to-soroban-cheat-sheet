# Contributing

This started as one developer's porting notes (Aptos Move → Soroban) and is public because
the notes were more useful than expected. It is not a committee document. That shapes what
helps:

## What's most useful

1. **Corrections.** A wrong mapping is worse than a missing one; someone will act on it.
   If a claim here is false, say so, even bluntly, even without a fix.
2. **Missing Move concepts.** If you reached for something from Move and couldn't find the
   Soroban answer here, that gap is worth an issue on its own. You don't need the answer.
3. **Staleness.** The SDK moves. If a snippet stopped compiling, the CI may not have caught
   it yet.

Opinions on style, ordering, or tone are welcome but lower priority. The bias is toward
density: this is written for people who already know Move well and want the differences,
not an introduction.

## The one hard rule: Soroban snippets must compile

Every Soroban snippet in `docs/` exists as real code under `verify/`, and CI runs it on
every pull request and weekly: a blocking job against the pinned SDK, `verify/Cargo.lock`
is committed and the job runs `--locked`, so the version in the `docs/index.md` header is the
version actually tested, plus a non-blocking job against the latest published release.
Documentation that can rot silently is how a reference becomes a liability.

If you change or add a Soroban snippet:

```bash
cd verify
cargo test          # must pass
```

Put the snippet in `src/lib.rs` (contract-side code), `src/advanced.rs` (auth trees,
deployer, custom accounts), or `tests/` (anything asserting runtime behaviour). Behavioural
claims, "X resets after Y", "the error lands in this position", should be an `assert!`,
not prose. That is how the `try_` nesting error in the first draft was caught.

Move snippets are illustrative and are not compiled. If one is wrong, an issue is fine.

## Layout

The book is generated with [mdBook](https://rust-lang.github.io/mdBook/) and published to
GitHub Pages from `main`. `docs/` is the book source, so every file in it ends up on the
site:

- `docs/index.md`, the landing page and the quick map, the 37-row table that indexes
  everything.
- `docs/NN-slug.md`, one page per numbered section.
- `docs/SUMMARY.md`, the sidebar. A page that isn't listed here doesn't appear in the book.
- `book.toml`, mdBook configuration, at the repo root.

The root `README.md` is the GitHub landing page only. It is a pointer to the site and is not
part of the book.

Adding a section means: a new `docs/NN-slug.md`, a row (or rows) in the quick map, an entry in
the contents list in `docs/index.md`, and a line in `docs/SUMMARY.md`.

Cross-references between pages are relative Markdown links (`index.md`,
`03-state-and-storage.md#33-get-returns-a-copy--false-friend`) so they resolve on GitHub and
on the published site alike. mdBook rewrites `.md` to `.html` at build time; it does not
rewrite `README.md` to `index.html`, which is why the landing page is named `index.md`.

To preview locally:

```bash
cargo install mdbook   # or: brew install mdbook
mdbook serve --open
```

## Scope

In: Move↔Soroban differentials, traps, platform semantics (storage, auth, metering,
dispatch), tooling equivalences.

Out: Soroban tutorials from scratch (the [Stellar docs](https://developers.stellar.org/docs/build/smart-contracts)
do this well), Sui Move specifics (different enough to deserve its own document),
chain advocacy in either direction.

## Provenance

The Soroban material follows Stellar's [`stellar-dev-skill`](https://github.com/stellar/stellar-dev-skill)
and the official developer docs; the Move material follows the Aptos docs and Move book.
Both are credited in §13. If you add material, link the primary source; an unsourced
claim here is a future issue.
