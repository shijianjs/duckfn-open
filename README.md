[English](README.md) | [简体中文](README.zh.md)

# duckfn_open

A DuckDB [loadable extension](https://duckdb.org/docs/stable/extensions/extension_development) written
with [duckfn](https://crates.io/crates/duckfn): attribute macros turn ordinary Rust functions into DuckDB
scalar functions, and the C++ build is not involved at all (the C API is used headers-only, through
DuckDB's API table).

What it adds to SQL is two steps, both of them "get this in front of a human". The
[open](https://crates.io/crates/open) crate's one job — handing a path or URL to whatever the desktop is
configured to open it with — and the step in front of it, because what a query usually holds is *content*
(a rendered page, a chart, a CSV, the bytes of an image) and a launcher only reads paths:

```sql
SELECT open_path('/var/tmp/report.html');           -- a path that already exists
SELECT open_html('<h1>hello</h1>');                 -- content: a temporary .html, in the browser
SELECT open_write(csv, 'csv');                      -- content: a file, its path back, nothing started
SELECT open_print(chart);                           -- content: stdout, past the result set's layout
```

Both halves are aimed at exploring inside a DuckDB session, not at embedding a launcher in a production
program; the content half is that persistence step written once, as a general "content → somewhere you
can look at it" instead of a copy per extension.

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

Three names start no process and so answer anywhere, which makes them the ones to try first:
`open_commands` replies with what *would* open this here, `open_write` only writes a file and hands back
its path, `open_print` only writes to stdout.

The `Justfile` wraps the same commands: `just build`, `just sql "SELECT open_commands('report.html')"`,
`just repl` (a REPL with the extension already loaded).

## Functions

Seven SQL names and sixteen signatures, every one of them a scalar function — no aggregate function and
no table function — plus one STRUCT type the extension registers,
`open_write_options(dir VARCHAR, "name" VARCHAR, part VARCHAR)`, which is where the optional arguments of
the write family live. Five of the names carry more than one signature under a single SQL name (merged
with duckfn's `overloads_name`), so the calls read `open_write(html, 'html', opts)` and
`open_path('a', 'firefox')`, never `open_write_with(…)` or `open_path_with(…)`. Implementation:
`src/extension/functions/`, the persistence rules in `src/extension/writer.rs`, the platform differences
in `src/extension/launcher.rs`.

The first group takes a path or URL that already exists and hands it to the desktop:

| Function | Input → output | open call |
| --- | --- | --- |
| `open_path(path)` | `VARCHAR` → `BOOLEAN` | `open::that` |
| `open_path(path, app)` | `VARCHAR, VARCHAR` → `BOOLEAN` | `open::with` |
| `open_path_detached(path)` | `VARCHAR` → `BOOLEAN` | `open::that_detached` |
| `open_path_detached(path, app)` | `VARCHAR, VARCHAR` → `BOOLEAN` | `open::with_detached` |
| `open_commands(path)` | `VARCHAR` → `VARCHAR[]` | `open::commands` |

The second group starts from content, which is the shape of most of what a query produces:

| Function | Signatures | Output |
| --- | --- | --- |
| `open_write` | `(content, ext)`, `(content, ext, opts)` and the same two for `BLOB` | `VARCHAR` — the path it wrote; starts nothing |
| `open_content` | the same four | `VARCHAR` — the path, and the default program already showing it |
| `open_html` | `(content)` | `VARCHAR` — the zero-configuration case: a temporary `.html` in the default browser |
| `open_print` | `(text)`, `(text, tail)` | `VARCHAR` — always NULL; the text went to stdout |

`open_content` is `open_write` composed with `open_path` under one name because it is the most common
thing to do while exploring, and `open_html` is `open_content(content, 'html')` with all three options
left at their default because what SQL renders is HTML nine times out of ten.

### What a call answers

The two groups answer differently, and each of those is its own contract.

**The launch family** — three distinguishable outcomes, and that is the whole contract:

- **`NULL`** — the row had no input. The arguments are non-`Option`, so duckfn's argument reader
  short-circuits a NULL to SQL NULL and no launcher is ever asked (`SELECT open_path(NULL); -- NULL`);
- **`true`** — a launcher was started. For the unsuffixed pair this is the launcher's own return, so on
  Linux it means the target program ran and finished; on Windows the `_detached` pair goes through one
  ShellExecuteW call and says only that the process started;
- **`false`** — no launcher could be started (no `xdg-open` on this machine, the chosen program is
  missing). open's real reason is an `io::Error` with nowhere to go in SQL, so use `open_commands` to
  see what would run at all.

**The write family** returns a path, which moves where a failure can be written:

- `open_write` starts nothing, so its three answers are about the file: a path, NULL for a row with no
  input, an error for an argument it cannot make into a file name;
- `open_content` and `open_html` add a launch to that. Their return value is already the path, so a
  launcher that cannot start is not the `false` `open_path` would answer with but an **error carrying
  that path** — the file is on disk, this machine just has nothing to show it with. They take the
  non-blocking route (`open::that_detached`): writing a chart and opening it should not hang until the
  viewer is closed;
- `open_print` returns NULL always: the text is on the console and SQL has nothing left to say.

### The rules the write family follows

All four `open_write` and all four `open_content` signatures share one persistence layer, so one list
covers both names:

- **the extension is the required setting** — it is what decides how the system treats the file. `html`
  and `.html` mean the same thing; an empty one, or one carrying a path separator (a path in disguise),
  is an error rather than a guess. For a BLOB it matters even more: without a suffix the viewer has no
  idea what it is looking at;
- **`dir`** defaults to the system temp directory, and a directory that does not exist is created with
  its parents — whoever writes `dir` means "this is where it should go", not "this is already there";
- **`name`** is used verbatim, with the required extension appended when the name carries none, and
  **an existing file at that path is overwritten**: you named it, that is the path you wanted;
- with no `name` the file is generated as `open-<%Y%m%d-%H%M%S>-<part>-<random>.<extension>`, e.g.
  `open-20260101-120000-Sales_Overview-9Qm2Zk.svg`. The time comes right after the prefix, so sorting a
  directory by name sorts it by time; the random suffix, and the guarantee that a generated name never
  lands on an existing file, are `tempfile`'s;
- **`part`** is the "which one is this" segment of a generated name (a chart title, a report name),
  capped at 48 characters — the alternative is a directory full of timestamps;
- `dir`, `name` and `part` are all treated as file-name material and go through `sanitize-filename`
  (illegal characters, control characters, Windows reserved device names, trailing dots and spaces), so
  a `part` carrying a slash cannot escape its directory and a Chinese title still fits in a name; spaces
  inside `part` become `_`;
- a directory and a file name are always joined with `/`, on every platform: that string is read by a
  human and usually concatenated further by a downstream query, and `std::fs` accepts it on Windows too.
  The directory part keeps the shape you gave it, so a default `dir` on Windows reads
  `C:\Users\…\Temp/open-….json`.

Choosing the program is deliberately not this family's job, and the options STRUCT has no `app` field for
the same reason: `open_path(open_write(html, 'html'), 'firefox')` already says it, and it keeps the path
so you can check it before opening anything.

`open_print` exists for the one thing a result set cannot do. The DuckDB CLI *lays out* its results —
column widths, borders, wrapping — and character art (the ASCII or braille charts a `textplot`-style
extension returns as a VARCHAR) is read line by line, so putting it in one cell destroys it. The text goes
to stdout exactly as it is, flushed on every call because Rust buffers stdout by line; `tail` replaces
the newline that is appended by default — `''` means "keep going on this line",
`chr(10) || chr(10)` means "leave a blank line".

### Two DuckDB quirks worth knowing

- a struct literal that omits a field is **not** implicitly widened to the three-field STRUCT, so write
  either every field or the cast:
  `SELECT open_write(svg, 'svg', {'dir':'./out'}::open_write_options);`
- a whole STRUCT passed as NULL is a NULL argument, so it short-circuits to NULL exactly like a NULL
  content does — nothing is written. It does not mean "the rest at their defaults"; **omit the third
  argument** for those.

### Calling any of this in a loop

Two things apply to every name:

- whether to wait is the visible difference inside the launch pair: `open::that` returns when the
  launcher comes back, and on Linux `xdg-open` typically only returns once the target program exits.
  On Windows, with the `shellexecute-on-windows` feature of `open` turned on, `open_path_detached` is
  the more direct route (one native API call instead of a PowerShell helper plus an `explorer.exe`
  fallback);
- DuckDB may fold a call whose arguments are all constants into a single execution, so
  `SELECT open_path('report.html')` opens one window and `SELECT open_print('x')` prints one line, while
  `SELECT open_path(p) FROM reports` opens one per row — there is no deduplication. Put a column
  reference in the argument (`open_print('x: ' || a)`) when you do want one call per row; for
  `open_write` one file per row is exactly what the generated name is for.

An empty path, an empty program name, an empty extension and an unusable option are errors rather than a
`false` or a silent default — a typo and an unopenable file deserve different answers. DuckDB reports an
invalid input error with the function's own name in front of the message:

```sql
SELECT open_path('');    -- open_path: the path or URL must not be empty
SELECT open_write('a', '');
-- open_write: the extension must not be empty (pass e.g. 'html', not '')
SELECT open_write('a', 'txt', {'dir':'   ', 'name':NULL, 'part':NULL});
-- open_write: open_write_options.dir must not be empty
```

The launcher is desktop software, which is why this extension builds for desktop platforms only (see
[Platforms](#platforms)). On a machine with no desktop the four `open_path` actions answer `false`,
`open_content` and `open_html` raise an error carrying the path they did write, and `open_commands` still
answers — listing what would be tried there. `open_write` and `open_print` need no desktop at all. See
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
just test          # make configure + make debug + make test + cleans what the tests wrote
just ci-build      # just the official build, without running the tests
```

`make test` does not rebuild, so run `just ci-build` (or `make debug`) first after touching Rust code.
Whether a launch succeeds depends on this machine's desktop configuration, and a CI runner has no
desktop, so a test never calls anything that launches. What runs instead is everything that does not
need one: the signature shapes as registered in `duckdb_functions()`, the NULL short-circuit, the
argument validation (including the checks that fail before a file is written), `open_commands` — the
launch action that starts no process — and the persistence layer for real: `open_write`'s files are
written and read back with `read_text` / `read_blob` (naming, directory creation, byte-exact BLOB
round-trips, sanitizing, overwriting), and `open_print` is actually called and asserted to answer NULL.
`open_content` and `open_html` run through the same `writer::write`, so their tests stay on the catalog
side and on the calls that error before the launch step.

Those tests write into `test/sql/` (the runner's `__TEST_DIR__`), so `.gitignore` keeps only `*.test`
there and `just test` ends with `git clean -fdX -- test/sql`. The development guide's
[testing](docs/docs/development/testing.md) page has the iteration loop (running the runner straight
against `build/debug/*.duckdb_extension`) and what each test file covers.

## Platforms

Windows, Linux and macOS only. A launcher is desktop software, so this extension builds no WebAssembly
target: CI excludes the three wasm architectures (`exclude_archs` in
`.github/workflows/MainDistributionPipeline.yml`), and `just build_wasm` / `build_wasm_eh` / `test_wasm`
are overridden to say so instead of building. On a machine with no desktop the four `open_path` actions
answer `false` and `open_content` / `open_html` raise an error carrying the path they wrote, while
`open_commands`, `open_write` and `open_print` work as usual.

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
| [docs/docs/user-guide/functions.md](docs/docs/user-guide/functions.md) | every function with its signatures and examples, per name |
| [README.zh.md](README.zh.md) | this file, in Chinese |
