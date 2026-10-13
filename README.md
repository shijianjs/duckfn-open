[English](README.md) | [简体中文](README.zh.md)

# my_extension

A DuckDB [loadable extension](https://duckdb.org/docs/stable/extensions/extension_development) written
with [duckfn](https://crates.io/crates/duckfn): attribute macros turn ordinary Rust functions into DuckDB
scalar / aggregate / table functions, and the C++ build is not involved at all (the C API is used
headers-only, through DuckDB's API table).

This repository is a **template**: [duckfn-extension-template](https://github.com/shijianjs/duckfn-extension-template).
It carries the full working loop — build, sqllogictest, docs export, release — around two sample
functions, so a new extension starts from a green build instead of from an empty directory. A real
extension written the same way: [duckfn_quantstats](https://github.com/shijianjs/duckfn-quantstats).

## Starting a new extension from this template

```shell
git clone https://github.com/shijianjs/duckfn-extension-template my_new_extension
cd my_new_extension
rm -rf .git && git init    # optional: drop the template's history and start your own
just rename my_new_extension
```

`just rename` (that is, `scripts/rename.sh`) rewrites every place the extension name has to match — the
crate name and `[[example]] name`, `EXTENSION_NAME` in the Makefile, the entry-point symbol, the Justfile,
the CI workflow and the docs — and regenerates the `Cargo.lock` entry. It ends by printing the few things
left for a human, all of them listed in the development guide's
[quick start](docs/docs/development/quick-start.md) (next steps) and
[AGENTS.md](AGENTS.md) (conventions, including the `{{PROJECT_GOAL}}` placeholder).
Replacing the two sample functions with your own API is one of them.

## Quick start

```shell
make configure   # once: builds configure/venv (Python + the sqllogictest runner)
make debug       # -> build/debug/my_extension.duckdb_extension
```

Locally built extensions are unsigned, so DuckDB has to be started with `-unsigned`:

```shell
duckdb -unsigned -c "
LOAD './build/debug/my_extension.duckdb_extension';
SELECT my_greet('world');
-- Hello, world!
SELECT my_sum(x) FROM (VALUES (1.5::DOUBLE), (2.5::DOUBLE), (3.0::DOUBLE)) t(x);
-- 7.0
"
```

The `Justfile` wraps the same commands: `just build`, `just sql "SELECT my_greet('world')"`,
`just repl` (a REPL with the extension already loaded).

## Functions

Two sample functions, one per registration path. They are meant to be replaced — see
`src/extension/functions/`.

| Function | Kind | Input → output |
| --- | --- | --- |
| `my_greet(name)` | scalar | `VARCHAR` → `VARCHAR`, never NULL |
| `my_greet_checked(name)` | scalar | `VARCHAR` → `VARCHAR`, `NULL` for an empty name, an error for surrounding whitespace |
| `my_sum(value)` | aggregate | `DOUBLE` → `DOUBLE`, NULLs skipped, `NULL` for an empty group |

Behaviour worth knowing, because it is duckfn's rule rather than this template's:

- a non-`Option` argument short-circuits NULL to SQL NULL — the function body never runs for that row;
  write the parameter as `Option<T>` to see the NULL and decide its meaning yourself;
- `-> DuckOptionResult<T>` is how a scalar function returns NULL (`Ok(None)`) or fails the query (`Err`);
- an aggregate is "a function with a `&mut` state parameter": the state's `Output` decides the SQL
  return type, and `result` decides whether the group yields a value or NULL.

## Build from source

The official DuckDB `extension-ci-tools` makefiles are the build path:

```shell
make configure           # once: builds configure/venv (Python + the sqllogictest runner)
make debug               # -> build/debug/my_extension.duckdb_extension
```

`make release` is the optimized version of the same flow. On Windows `make` has to run inside Git Bash.
The `Justfile` wraps it (`just build` = `make configure && make debug`, `just ci-build`, `just test`,
`just ci-release`).

## Testing

Tests are SQLLogicTest files under `test/sql/`:

```shell
just test          # make configure + make debug + make test
just ci-build      # just the official build, without running the tests
```

`make test` does not rebuild, so run `just ci-build` (or `make debug`) first after touching Rust code.
The development guide's [testing](docs/docs/development/testing.md) page has the iteration loop (running
the runner straight against `build/debug/*.duckdb_extension`) and what each test file covers.

## WebAssembly

```shell
just config_env   # once: pin the toolchain and add the wasm target
just build_wasm
```

The wasm build uses `src/wasm_lib.rs` (a `staticlib` mirror of `src/lib.rs`); the two crate roots must
always declare the same set of `mod`s.

## Documentation site

The repository carries a [Docusaurus](https://docusaurus.io/) site in `docs/`, in English and
Simplified Chinese, with a workflow that publishes it to GitHub Pages on every version tag:

```shell
just docs_install    # once
just docs_start      # dev server at http://localhost:3000
just docs_build      # the check that matters: onBrokenLinks is set to throw
```

The template's pages describe the sample functions; rewrite them (and their translations under
`docs/i18n/zh-Hans/`) as your API grows, or delete `docs/` — nothing else depends on it. The pages
carry runnable SQL blocks (powered by [`duckfn-docs-kit`](https://www.npmjs.com/package/duckfn-docs-kit))
that call the extension right in the browser; `cd docs && npm test` re-runs them. The conventions —
layout, commands, translation workflow, deployment, the `{{EXTENSION_VERSION}}` version placeholder —
are in [`docs/README.md`](docs/README.md).

## Installing the released extension

Releases are GitHub Releases carrying the build matrices' `.duckdb_extension` files, one per platform:

```shell
duckdb -unsigned -c "
LOAD 'https://github.com/<owner>/<repo>/releases/latest/download/my_extension-windows_amd64.duckdb_extension';
"
```

Publishing to DuckDB's [community extensions](https://duckdb.org/community_extensions/list_of_extensions)
makes it `INSTALL my_extension FROM community` instead; the two files that requires are prepared in
[`community-extension/`](community-extension/AGENTS.md).

## Documentation

| File | What is in it |
| --- | --- |
| [AGENTS.md](AGENTS.md) | conventions, the duckfn knowledge map, the release flow |
| [docs/docs/development/](docs/docs/development/quick-start.md) | the development guide: directory layout, skeleton trade-offs, writing and testing functions, build & release (English and Simplified Chinese) |
| [docs/README.md](docs/README.md) | the documentation site: layout, commands, translations, deployment |
| [README.zh.md](README.zh.md) | this file, in Chinese |
