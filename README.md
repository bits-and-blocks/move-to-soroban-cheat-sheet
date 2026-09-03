# Move -> Soroban: the differential cheat sheet

**→ [Read it here](https://bits-and-blocks.github.io/move-to-soroban-cheat-sheet/)**

For Aptos Move developers shipping Soroban contracts. Each entry is composed in the following
structure:

* What it is in Move
* What it is in Soroban
* Where the analogy breaks.
* The breakage is the payload.

**Legend:**

* ⚠ **FALSE FRIEND**: your Move instinct produces compiling, working-looking Soroban code that is wrong.
* ∅ **NO ANALOGUE**: nothing on the other side, don't force one.
* 🔧 **DIRECT SWAP**: mechanical rename, low risk.

The [quick map](https://bits-and-blocks.github.io/move-to-soroban-cheat-sheet/) is a 37-row
table indexing every differential; each row links to the section that explains it.

## Contents

1. [Identity & authorization](docs/01-identity-and-authorization.md)
2. [Capabilities & the object model](docs/02-capabilities-and-the-object-model.md)
3. [State & storage](docs/03-state-and-storage.md)
4. [Types & abilities](docs/04-types-and-abilities.md)
5. [Modules vs. contracts](docs/05-modules-vs-contracts.md)
6. [Execution semantics](docs/06-execution-semantics.md)
7. [Events](docs/07-events.md)
8. [Tokens](docs/08-tokens.md)
9. [Testing & assurance](docs/09-testing-and-assurance.md)
10. [Toolchain quick map](docs/10-toolchain.md)
11. [Porting reflexes, the grep-able checklist](docs/11-porting-reflexes.md)
12. [Verification](docs/12-verification.md)
13. [Primary sources](docs/13-primary-sources.md)

## Verification

Every Soroban snippet in `docs/` exists as real code under `verify/` and runs in CI on every
pull request and weekly; a blocking job against the pinned SDK and an advisory job against
the latest published release. Move snippets are illustrative and are not compiled. See
[Verification](docs/12-verification.md).

## Building the site

The book is [mdBook](https://rust-lang.github.io/mdBook/); `docs/` is its source.

```bash
cargo install mdbook   # or: brew install mdbook
mdbook serve --open
```

## Contributing

Corrections are the most valuable contribution. Missing Move concepts are equally welcome as
issues even without an answer. See [CONTRIBUTING.md](docs/CONTRIBUTING.md); All Soroban
snippets must compile.
