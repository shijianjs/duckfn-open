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

What it adds is one sentence of SQL for what your desktop already does for you: hand a path or a URL to
the program configured to open it, so a query can drop a report in front of a human.

It adds three function names, five signatures, and all of them are scalar — no aggregate functions, no
table functions, no custom types:

| Function | Kind | What it does |
| --- | --- | --- |
| [`open_path(path)`](./functions.md#open_path) | scalar | Opens the path or URL with the default program, and waits for the launcher. |
| [`open_path(path, app)`](./functions.md#open_path) | scalar | The same, with the program named in the second argument. |
| [`open_path_detached(path)`](./functions.md#open_path_detached) | scalar | Opens it and returns as soon as the launcher has been started. |
| [`open_path_detached(path, app)`](./functions.md#open_path_detached) | scalar | The same, with the program named in the second argument. |
| [`open_commands(path)`](./functions.md#open_commands) | scalar | Lists, without running them, the launch commands `open_path` would try in order. |

## Install it

```sql
INSTALL duckfn_open FROM community;
LOAD duckfn_open;
```

Then call the functions in any query:

```sql
SELECT open_path('/var/tmp/report.html');        -- the program your desktop uses for .html
SELECT open_path_detached('https://duckdb.org'); -- returns without waiting for the page to close
```

Nothing on these pages runs for you: this is documentation, and every example is plain SQL for a DuckDB that
has loaded the extension. The query below lists what got registered rather than launching anything, so it is
the safest one to try first — it needs no desktop at all. The four launch signatures do, and answer `false` on
a machine that has none (a server, a CI runner); [Functions](./functions.md) explains each answer.

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'open%'
ORDER BY 1, 2;    -- three names, five signatures
```

[Installation](./installation.md) covers the other ways to get the file.

:::note[A word about side effects]

Four of the five signatures start a process, so the questions worth asking before putting one in a loop
are "how many windows is this going to open" and "on whose machine is this query running". A call whose
arguments are all constants is folded by DuckDB into a single execution, which is why
`SELECT open_path('report.html')` opens one window and `SELECT open_path(p) FROM reports` opens one per
row — there is no deduplication. `open_commands` is the one name with no side effect at all, and it answers
"what would run here" directly.

:::

## Where to go next

- [Installation](./installation.md) — the community repository, a release file, or a local build.
- [Functions](./functions.md) — every signature, what each answer means, and what to do when nothing opens.
- [Development guide](../development/quick-start.md) — building the extension from source. That part is
  for people working on this repository; it is not needed to use the extension.
