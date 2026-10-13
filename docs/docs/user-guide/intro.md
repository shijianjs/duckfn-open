---
title: Introduction
sidebar_position: 1
slug: /intro
description: What duckfn_open adds to DuckDB, and how to start using it.
---

# Introduction

`duckfn_open` is a [DuckDB loadable extension](https://duckdb.org/docs/stable/extensions/overview): it
adds SQL functions to DuckDB, and once it is loaded they behave like DuckDB's own. You do not need to know
how it was written or to compile anything — the CLI, a Python or R session and the other DuckDB front ends
all take the same `.duckdb_extension` file.

What it adds is one sentence of SQL for what your desktop already does for you: a path or a URL goes to
the program configured to open it, so a query can drop a report in front of a human. Most of what a query
produces has no path yet, though — it is content — so the other half of the API writes the content out as
a file first, and opens that. A third piece belongs to neither of those: it settles, on a plain string,
whether that string can be used as a file name at all.

Loading the extension registers nine function names and twenty-one signatures, and every one of them is
scalar: no aggregate functions, no table functions. It also registers one STRUCT type,
`open_write_options(dir VARCHAR, "name" VARCHAR, part VARCHAR)`, which carries the three optional
settings of the write family; like any registered type it shows in `duckdb_types()` rather than in
`duckdb_functions()`. The two filename functions register no type of their own.

The names fall into three groups. The first takes a path or URL that already exists and hands it to the
desktop:

| Function | Kind | What it does |
| --- | --- | --- |
| [`open_path(path)`](./functions.md#open_path) | scalar | Opens the path or URL with the default program, and waits for the launcher. |
| [`open_path(path, app)`](./functions.md#open_path) | scalar | The same, with the program named in the second argument. |
| [`open_path_detached(path)`](./functions.md#open_path_detached) | scalar | Opens it and returns as soon as the launcher has been started. |
| [`open_path_detached(path, app)`](./functions.md#open_path_detached) | scalar | The same, with the program named in the second argument. |
| [`open_commands(path)`](./functions.md#open_commands) | scalar | Lists, without running them, the launch commands `open_path` would try in order. |

The second starts from content: a VARCHAR or a BLOB becomes a file, and from there it is an ordinary
path that the first group can work with.

| Function | Kind | What it does |
| --- | --- | --- |
| [`open_write(content, ext)`](./functions.md#open_write) | scalar | Writes the content out and returns its path, starting nothing. |
| [`open_write(content, ext, opts)`](./functions.md#open_write) | scalar | The same, with the directory, the file name and a name segment chosen through `open_write_options`. |
| [`open_content(…)`](./functions.md#open_content) | scalar | The four signatures of `open_write` plus a launch: writes it out, opens it with the default program, returns the path. |
| [`open_html(content)`](./functions.md#open_html) | scalar | The zero-configuration shortcut for HTML: `open_content(content, 'html')` with every option left at its default. |
| [`open_print(text)`](./functions.md#open_print) | scalar | Writes the text straight to stdout, past the result set's formatting; always returns NULL. |
| [`open_print(text, tail)`](./functions.md#open_print) | scalar | The same, with what to append after the text instead of a newline. |

The third group is pure string work: it neither starts a program nor touches the filesystem, and it answers
the question the other two run into as soon as a name comes out of the data — can this value be a file
name?

| Function | Kind | What it does |
| --- | --- | --- |
| [`open_filename(value)`](./functions.md#open_filename) | scalar | Rewrites a string into something usable as a file name, deleting what cannot be in one. |
| [`open_filename(value, replacement)`](./functions.md#open_filename) | scalar | The same, with what to put where an illegal character was, so the edits show. |
| [`open_filename(value, replacement, truncate, windows)`](./functions.md#open_filename) | scalar | The same, plus the 255-byte cut and the Windows rules as two switches. |
| [`open_filename_check(value)`](./functions.md#open_filename_check) | scalar | Rewrites nothing: `true` when the value is already a usable name. |
| [`open_filename_check(value, truncate, windows)`](./functions.md#open_filename_check) | scalar | The same question, with the two switches. |

## Install it

```sql
INSTALL duckfn_open FROM community;
LOAD duckfn_open;
```

Then call the functions in any query:

```sql
SELECT open_path('/var/tmp/report.html');        -- the program your desktop uses for .html
SELECT open_path_detached('https://duckdb.org'); -- returns without waiting for the page to close
SELECT open_write('{"a": 1}', 'json');           -- a file and its path; nothing is started
SELECT open_html('<h1>hello</h1>');              -- a temporary .html, opened in the default browser
SELECT open_filename('quarterly/report:v2.csv'); -- 'quarterlyreportv2.csv' — a string, nothing else
```

Nothing on these pages runs for you: this is documentation, and every example is plain SQL for a DuckDB
that has loaded the extension. The query below only reads the catalog, so it is the safest one to try
first: it needs no desktop, and neither do `open_write`, `open_print` or the filename family — one writes a
file, one writes to stdout, and the other two rewrite a string and stop there. Launching does need a
desktop: on a machine that has none (a server, a CI runner) the four `open_path` actions answer `false`,
while `open_content` and `open_html` raise an error carrying the path they did write.
[Functions](./functions.md) explains each answer.

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'open%'
ORDER BY 1, 2;    -- nine names, twenty-one signatures
```

[Installation](./installation.md) covers the other ways to get the file.

:::note[A word about side effects]

Nine of the twenty-one signatures start a process, so the questions worth asking before putting one in a
loop are "how many windows is this going to open" and "on whose machine is this query running". A call
whose arguments are all constants is folded by DuckDB into a single execution, which is why
`SELECT open_path('report.html')` opens one window and `SELECT open_path(p) FROM reports` opens one per
row — there is no deduplication. Nothing else starts a process: `open_commands` only lists the commands a
launch would try, `open_write` writes a file and stops there — which is what lets you hand the path to
`open_path` yourself — `open_print` only writes to stdout, and the two filename names never leave the
string they were given.

:::

## Where to go next

- [Installation](./installation.md) — the community repository, a release file, or a local build.
- [Functions](./functions.md) — every signature, the options STRUCT, what each answer means, and what to
  do when nothing opens.
- [Development guide](../development/quick-start.md) — building the extension from source. That part is
  for people working on this repository; it is not needed to use the extension.
