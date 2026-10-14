---
title: Introduction
sidebar_position: 1
slug: /intro
description: What duckfn_open adds to DuckDB, and how to start using it.
---

# Introduction

`duckfn_open` is a [DuckDB loadable extension](https://duckdb.org/docs/stable/extensions/overview): it adds
SQL functions, and once it is loaded they behave like DuckDB's own. Nothing to compile and nothing to import
at the call site — the CLI, a Python or R session and the other front ends all take the same
`.duckdb_extension` file.

What it adds is one sentence of SQL for what your desktop already does: a path or a URL goes to the program
configured to open it, so a query can put a report in front of a human. Most of what a query produces has no
path yet, though — it is content — so the other half of the API writes the content out as a file first, and
opens that. A third group belongs to neither: it settles, on a plain string, whether that string can be a
file name at all.

Loading it registers nine function names and twenty-one signatures, every one of them scalar, plus one
STRUCT type for the write options. The groups:

- **Paths and URLs** — [Opening a path](./functions/open-path.md): `op_open_path` hands a file or a link to
  the program the desktop has configured, and waits for it; `op_open_path_detached` returns without waiting;
  `op_open_commands` only lists the commands a launch would try.
- **Content** — [Writing content out](./functions/write-file.md) and
  [Printing to stdout](./functions/print.md):   `op_write_file` turns text or bytes into a file and returns the
  path, `op_open_content` opens it in the same call, `op_open_html` does that with no options at all, and
  `op_print` writes to stdout instead, past the result set's formatting, and hands the text back.
- **File names** — [File names from data](./functions/sanitize-filename.md): `op_sanitize_filename` rewrites
  a value into a name every platform accepts, and `op_sanitize_filename_check` answers whether it already is
  one. Neither leaves the string it was given.

## Install it

```sql
INSTALL duckfn_open FROM community;
LOAD duckfn_open;
```

Then call the functions in any query:

```sql
SELECT op_open_path('/var/tmp/report.html');        -- the program your desktop uses for .html
SELECT op_open_path_detached('https://duckdb.org'); -- returns without waiting for the page to close
SELECT op_write_file('{"a": 1}', 'json');           -- a file and its path; nothing is started
SELECT op_open_html('<h1>hello</h1>');              -- a temporary .html, opened in the default browser
SELECT op_sanitize_filename('quarterly/report:v2.csv'); -- 'quarterlyreportv2.csv' — a string, nothing else
```

Nothing on this site runs for you: every example is plain SQL for a DuckDB that has loaded the extension.
The query below only reads the catalog, so it is the safest one to try first. `op_write_file`, `op_print`
and the two filename functions need no desktop either; launching does. On a machine without one — a server,
a CI runner — the four `op_open_path` actions answer `false`, while `op_open_content` and `op_open_html`
raise an error carrying the path they did write. [Functions](./functions/index.md) explains each answer.

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'op%'
ORDER BY 1, 2;    -- nine names, twenty-one signatures
```

[Installation](./installation.md) covers the other ways to get the file.

::::note[A word about side effects]

Nine of the twenty-one signatures start a process, so the questions worth asking before putting one in a
loop are "how many windows is this going to open" and "on whose machine is this query running". A call whose
arguments are all constants is folded by DuckDB into a single execution, which is why
`SELECT op_open_path('report.html')` opens one window and `SELECT op_open_path(p) FROM reports` opens one per
row — there is no deduplication. Nothing else starts a process: `op_open_commands` only lists the commands a
launch would try, `op_write_file` writes a file and stops there — which is what lets you hand the path to
`op_open_path` yourself — `op_print` only writes to stdout, and the two filename functions never leave the
string they were given.

::::

## Where to go next

- [Installation](./installation.md) — the community repository, a release file, or a local build.
- [Functions](./functions/index.md) — every signature, the options STRUCT, what each answer means, and what
  to do when nothing opens.
- [Development guide](../development/quick-start.md) — building the extension from source. That part is for
  people working on this repository; it is not needed to use the extension.
