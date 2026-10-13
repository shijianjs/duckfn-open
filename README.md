[English](README.md) | [简体中文](README.zh.md)

# duckfn_open

A DuckDB [loadable extension](https://duckdb.org/docs/stable/extensions/extension_development) written
with [duckfn](https://crates.io/crates/duckfn): attribute macros turn ordinary Rust functions into DuckDB
scalar functions, and the C++ build is not involved at all (the C API is used headers-only, through
DuckDB's API table).

What it adds to SQL is the [open](https://crates.io/crates/open) crate's one job — handing a path or URL
to whatever the desktop is configured to open it with — so a query can drop a report in front of a human:

```sql
SELECT open_path('/var/tmp/report.html');
SELECT open_path_detached('https://duckdb.org');
```

This repository starts from [duckfn-extension-template](https://github.com/shijianjs/duckfn-extension-template),
which carries the full working loop — build, sqllogictest, docs export, release. Another extension written
the same way: [duckfn_quantstats](https://github.com/shijianjs/duckfn-quantstats).

## Quick start

```shell
make configure   # once: builds configure/venv (Python + the sqllogictest runner)
make debug       # -> build/debug/duckfn_open.duckdb_extension
```

Locally built extensions are unsigned, so DuckDB has to be started with `-unsigned`:

```shell
duckdb -unsigned -c "
LOAD './build/debug/duckfn_open.duckdb_extension';
SELECT open_commands('report.html');
-- the launchers this machine would try, in order, running none of them
"
```

`open_commands` is the side-effect-free entry point of the extension, which makes it the one to try
first: it answers "what would actually open this here" without starting a process.

The `Justfile` wraps the same commands: `just build`, `just sql "SELECT open_commands('report.html')"`,
`just repl` (a REPL with the extension already loaded).

## Functions

Three SQL names, five signatures, all scalar. `open_path` and `open_path_detached` each carry two —
the default program, and a program named in the second argument — merged into one function set with
duckfn's `overloads_name`. Implementation: `src/extension/functions/`.

| Function | Input → output | open call |
| --- | --- | --- |
| `open_path(path)` | `VARCHAR` → `BOOLEAN` | `open::that` |
| `open_path(path, app)` | `VARCHAR, VARCHAR` → `BOOLEAN` | `open::with` |
| `open_path_detached(path)` | `VARCHAR` → `BOOLEAN` | `open::that_detached` |
| `open_path_detached(path, app)` | `VARCHAR, VARCHAR` → `BOOLEAN` | `open::with_detached` |
| `open_commands(path)` | `VARCHAR` → `VARCHAR[]` | `open::commands` |

Three outcomes are distinguishable from SQL, and that is the whole contract:

- **`NULL`** — the row had no input. The arguments are non-`Option`, so duckfn's argument reader
  short-circuits a NULL to SQL NULL and no launcher is ever asked (`SELECT open_path(NULL); -- NULL`);
- **`true`** — a launcher was started. For the unsuffixed pair this is the launcher's own return, so on
  Linux it means the target program ran and finished; on Windows the `_detached` pair goes through one
  ShellExecuteW call and says only that the process started;
- **`false`** — no launcher could be started (no `xdg-open` on this machine, the chosen program is
  missing). open's real reason is an `io::Error` with nowhere to go in SQL, so use `open_commands` to
  see what would run at all.

An empty path or application is an error rather than a false — `Invalid Input Error: open_path: the
path or URL must not be empty` — because a typo and an unopenable file deserve different answers.

Two more things worth knowing before calling this in a loop:

- whether to wait is the visible difference between the two pairs: `open::that` returns when the
  launcher comes back, and on Linux `xdg-open` typically only returns once the target program exits.
  On Windows, with the `shellexecute-on-windows` feature of `open` turned on, `open_path_detached` is
  the more direct route (one native API call instead of a PowerShell helper plus an `explorer.exe`
  fallback);
- DuckDB may fold a call whose arguments are all constants into a single execution, so
  `SELECT open_path('report.html')` opens one window, while
  `SELECT open_path(p) FROM reports` opens one per row — there is no deduplication.

The launcher is desktop software, which is why this extension builds for desktop platforms only (see
[Platforms](#platforms)). On a machine with no desktop the four launch actions answer `false`, and
`open_commands` is the one call that still answers — listing what would be tried there. See
`src/extension/launcher.rs`.

## Build from source

The official DuckDB `extension-ci-tools` makefiles are the build path:

```shell
make configure           # once: builds configure/venv (Python + the sqllogictest runner)
make debug               # -> build/debug/duckfn_open.duckdb_extension
```

`make release` is the optimized version of the same flow. On Windows `make` has to run inside Git Bash.
The `Justfile` wraps it (`just build` = `make configure && make debug`, `just ci-build`, `just test`,
`just ci-release`). The `extension-ci-tools` submodule needs `git submodule update --init --recursive`
before the first build.

## Testing

Tests are SQLLogicTest files under `test/sql/`:

```shell
just test          # make configure + make debug + make test
just ci-build      # just the official build, without running the tests
```

`make test` does not rebuild, so run `just ci-build` (or `make debug`) first after touching Rust code.
Whether a launch succeeds depends on this machine's desktop configuration, and a CI runner has no
desktop, so the tests assert what does not depend on one: the signature shapes as registered in
`duckdb_functions()`, the NULL short-circuit, the argument validation, and `open_commands` — the action
that starts no process. The development guide's
[testing](docs/docs/development/testing.md) page has the iteration loop (running the runner straight
against `build/debug/*.duckdb_extension`) and what each test file covers.

## Platforms

Windows, Linux and macOS only. A launcher is desktop software, so this extension builds no WebAssembly
target: CI excludes the three wasm architectures (`exclude_archs` in
`.github/workflows/MainDistributionPipeline.yml`), and `just build_wasm` / `build_wasm_eh` / `test_wasm`
are overridden to say so instead of building. On a machine with no desktop the four launch actions answer
`false`; `open_commands` still answers.

## Documentation site

The repository carries a [Docusaurus](https://docusaurus.io/) site in `docs/`, in English and
Simplified Chinese, with a workflow that publishes it to GitHub Pages on every version tag:

```shell
just docs_install    # once
just docs_start      # dev server at http://localhost:3000
just docs_build      # the check that matters: onBrokenLinks is set to throw
```

The SQL on those pages is plain code blocks — running a block in the reader's browser would need a wasm
build of this extension, and there is none. The conventions (layout, commands, translation workflow,
deployment, the `{{EXTENSION_VERSION}}` version placeholder, and what a static block has to state
explicitly) are in [`docs/README.md`](docs/README.md) and [`docs/AGENTS.md`](docs/AGENTS.md).

## Installing the released extension

Releases are GitHub Releases carrying the build matrices' `.duckdb_extension` files, one per platform:

```shell
duckdb -unsigned -c "
LOAD 'https://github.com/shijianjs/duckfn-open/releases/latest/download/duckfn_open-windows_amd64.duckdb_extension';
"
```

Publishing to DuckDB's [community extensions](https://duckdb.org/community_extensions/list_of_extensions)
makes it `INSTALL duckfn_open FROM community` instead; the two files that requires are prepared in
[`community-extension/`](community-extension/AGENTS.md).

## Documentation

| File | What is in it |
| --- | --- |
| [AGENTS.md](AGENTS.md) | conventions, the duckfn knowledge map, the release flow |
| [docs/docs/development/](docs/docs/development/quick-start.md) | the development guide: directory layout, writing and testing functions, build & release (English and Simplified Chinese) |
| [docs/README.md](docs/README.md) | the documentation site: layout, commands, translations, deployment |
| [README.zh.md](README.zh.md) | this file, in Chinese |
