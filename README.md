[English](README.md) | [简体中文](README.zh.md)

# duckfn_open

`duckfn_open` puts two mature Rust crates on DuckDB SQL: [open](https://crates.io/crates/open), which hands a
path or URL to whatever the desktop is configured to open it with, and
[sanitize-filename](https://crates.io/crates/sanitize-filename), which settles whether a string can be a file
name. Neither is reimplemented here — the per-platform launchers and their argument quoting, and the rules for
what a legal file name is, are the crates' own, exposed call for call. Writing content out also leans on
`tempfile` and `chrono`, but only inside `op_write_file`'s generated name: that is one internal policy of one
function, not a third crate's API put on SQL.

What it adds to SQL is two steps, both of them "get this in front of a human": open's one job, and the step in
front of it, because what a query usually holds is *content* (a rendered page, a chart, a CSV, the bytes of an
image) and a launcher only reads paths:

```sql
SELECT op_open_path('/var/tmp/report.html');           -- a path that already exists
SELECT op_open_html('<h1>hello</h1>');                 -- content: a temporary .html, in the browser
SELECT op_write_file(csv, 'csv');                      -- content: a file, its path back, nothing started
SELECT op_print(chart);                                -- content: stdout, past the result set's layout
```

A third group stands outside both steps: `op_sanitize_filename` and `op_sanitize_filename_check` turn a string
into something usable as a file name, or answer whether it already is. No process, no file, no path.

Its direction is **interactive exploration**: a person is at the machine, running DuckDB, and wants the result
where they can look at it. The launcher it calls is the one configured on that same machine, which is also why
this is nothing to embed in an application — where DuckDB runs headless (a server, a scheduled job, a CI
runner) there is no desktop to hand a file to, and a library call that pops a window on somebody's machine is
not what the caller asked for. Of the two halves, the one that starts no process (`op_write_file`, `op_print`
and the two `op_sanitize_filename` names) is what still makes sense there.

## Install

`duckfn_open` needs **DuckDB 1.3 or newer** and a desktop — Windows, Linux or macOS. It is an ordinary
loadable extension:

```sql
INSTALL duckfn_open FROM community;   -- once per installation
LOAD duckfn_open;                     -- once per session
```

A GitHub release file or a locally built artifact works too, and both need DuckDB started with `-unsigned`;
[Installation](https://shijianjs.github.io/duckfn-open/docs/user-guide/installation) covers the three routes.

Five names start no process and so answer anywhere, which makes them the ones to try first:
`op_open_commands` replies with what *would* open this here, `op_write_file` only writes a file and hands back
its path, `op_print` only writes to stdout, and the two `op_sanitize_filename` names ask for neither a desktop
nor a file — a string in, a usable name (or a yes/no) out.

```sql
SELECT op_open_commands('report.html');
-- the launchers this machine would try, in order, running none of them
```

## Functions

Nine SQL names and twenty-one signatures, every one of them a scalar function — no aggregate function and
no table function — plus one STRUCT type the extension registers,
`op_write_file_options(dir VARCHAR, "name" VARCHAR, part VARCHAR)`, which is where the optional arguments of
the write family live. Seven of the names carry more than one signature under a single SQL name, so the calls
read `op_write_file(html, 'html', opts)` and `op_open_path('a', 'firefox')`, never
`op_write_file_with(…)` or `op_open_path_with(…)`.

The first group takes a path or URL that already exists and hands it to the desktop:

| Function | Input → output | open call |
| --- | --- | --- |
| `op_open_path(path)` | `VARCHAR` → `BOOLEAN` | `open::that` |
| `op_open_path(path, app)` | `VARCHAR, VARCHAR` → `BOOLEAN` | `open::with` |
| `op_open_path_detached(path)` | `VARCHAR` → `BOOLEAN` | `open::that_detached` |
| `op_open_path_detached(path, app)` | `VARCHAR, VARCHAR` → `BOOLEAN` | `open::with_detached` |
| `op_open_commands(path)` | `VARCHAR` → `VARCHAR[]` | `open::commands` |

The second group starts from content, which is the shape of most of what a query produces:

| Function | Signatures | Output |
| --- | --- | --- |
| `op_write_file` | `(content, ext)`, `(content, ext, opts)` and the same two for `BLOB` | `VARCHAR` — the path it wrote; starts nothing |
| `op_open_content` | the same four | `VARCHAR` — the path, and the default program already showing it |
| `op_open_html` | `(content)` | `VARCHAR` — the zero-configuration case: a temporary `.html` in the default browser |
| `op_print` | `(text)`, `(text, tail)` | `VARCHAR` — the text it was given; it went to stdout on the way |

The third group is pure strings and belongs to neither of the two above: nothing in it starts a process or
touches a file.

| Function | Signatures | Output |
| --- | --- | --- |
| `op_sanitize_filename` | `(value)`, `(value, replacement)`, `(value, replacement, truncate, windows)` | `VARCHAR` — the same string rewritten into a usable file name |
| `op_sanitize_filename_check` | `(value)`, `(value, truncate, windows)` | `BOOLEAN` — whether that rewrite would change nothing |

### What a call answers

- **the launch family** — `NULL` when the row had no input, `true` once a launcher was started, `false` when
  none could be started on this machine. An empty path or program name is an error rather than a `false`,
  because a typo and an unopenable file deserve different answers;
- **the write family** — the path it wrote, or an error for an argument that cannot become a file name.
  `op_open_content` and `op_open_html` add a launch, and a launcher that cannot start is an **error carrying
  that path**: the file is on disk, this machine just has nothing to show it with;
- **the filename pair** — a string or a `BOOLEAN`, `NULL` in and `NULL` out, and no error for a bad value:
  an unusable name is an answer here, not a rejection.

Every name that touches the outside world is registered **volatile**, so one call is one row: a constant
argument behaves exactly like a column one, and nothing runs while a query is being planned
(`EXPLAIN SELECT op_write_file('a', 'txt')` writes no file).

The rules behind those answers — the options STRUCT, how a generated name is put together, what the filename
crate counts as illegal, and the one default that is pinned instead of following the crate — are in the
[function reference](https://shijianjs.github.io/duckfn-open/docs/user-guide/functions/overview).

## Documentation

| Where | What is in it |
| --- | --- |
| [User guide](https://shijianjs.github.io/duckfn-open/docs/intro) | what it adds to DuckDB, installing it, every function with examples to run |
| [Development guide](docs/docs/development/quick-start.md) | building, testing and releasing the extension; project layout (also [published](https://shijianjs.github.io/duckfn-open/docs/development/quick-start)) |
| [AGENTS.md](AGENTS.md) | repository conventions, the duckfn knowledge map and the release flow |
| [README.zh.md](README.zh.md) | this file, in Chinese |
