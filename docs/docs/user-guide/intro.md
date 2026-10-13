---
title: Introduction
sidebar_position: 1
slug: /intro
description: What my_extension adds to DuckDB, and how to start using it.
---

# Introduction

`my_extension` is a [DuckDB loadable extension](https://duckdb.org/docs/stable/extensions/overview): it
adds SQL functions to DuckDB, and once it is loaded they behave like DuckDB's own. You do not need to know
how it was written or to compile anything — the CLI, a Python or R session and the browser build all take
the same `.duckdb_extension` file.

It adds three functions:

| Function | Kind | What it does |
| --- | --- | --- |
| [`my_greet(name)`](./functions.md#my_greet) | scalar | Greets `name`; never NULL. |
| [`my_greet_checked(name)`](./functions.md#my_greet_checked) | scalar | The same, but NULL for an empty name and an error for surrounding whitespace. |
| [`my_sum(value)`](./functions.md#my_sum) | aggregate | Sums a `DOUBLE` column, skipping NULLs. |

## Install it

```sql
INSTALL my_extension FROM community;
LOAD my_extension;
```

Then call the functions in any query:

```sql {"type":"duckfn","show":"table"}
SELECT name AS input, my_greet_checked(name) AS greeting
FROM (VALUES ('world'), ('')) t(name);
```

That block runs right here, in your browser: the site preloads the extension from the project's latest
release, so there is no `LOAD` to write. [Installation](./installation.md) covers the other ways to get
the file.

:::note[These pages are the template's starting point]

`my_extension` is the sample extension of
[duckfn-extension-template](https://github.com/shijianjs/duckfn-extension-template). The pages in this
part of the site describe its three sample functions; a project built from the template replaces them
with its own API and rewrites these pages to match.

:::

## Where to go next

- [Installation](./installation.md) — the community repository, a release file, or a local build.
- [Functions](./functions.md) — every function, with an example you can run.
- [Development guide](../development/quick-start.md) — building the extension from source. That part is
  for people working on this repository; it is not needed to use the extension.
