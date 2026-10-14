---
title: Functions
sidebar_position: 3
slug: /user-guide/functions
description: Every SQL function duckfn_open registers, with examples to run in your own DuckDB.
---

# Functions

Loading the extension registers nine function names and twenty-one signatures, and every one of them is a
scalar function: no aggregate, no table function. They behave like DuckDB's own, so they go in a projection
or a `WHERE` clause and combine with the built-in functions freely.

Seven of the nine names answer to more than one signature. DuckDB sees one function with several overloads,
so the extra forms have no SQL name of their own: the call is `SELECT op_open_path('a', 'b')`, not
`op_open_path_with('a', 'b')`.

The only registered type is [`op_write_file_options`](./write-file.md#op_write_file_options), the STRUCT that
carries the optional arguments of `op_write_file` and `op_open_content`. Like any registered type it appears
in `duckdb_types()` rather than in `duckdb_functions()`; the two filename functions register no type at all.

## Paths and URLs

A file or a link that already exists, handed to the desktop. Details:
[Opening a path](./open-path.md).

| Function | Signature | What it does |
| --- | --- | --- |
| [`op_open_path`](./open-path.md#op_open_path) | `VARCHAR -> BOOLEAN` | Opens it with the default program, and waits for the launcher. |
| [`op_open_path`](./open-path.md#op_open_path) | `VARCHAR, VARCHAR -> BOOLEAN` | The same, with the program named by the second argument. |
| [`op_open_path_detached`](./open-path.md#op_open_path_detached) | `VARCHAR -> BOOLEAN` | Opens it and returns as soon as the launcher has been started. |
| [`op_open_path_detached`](./open-path.md#op_open_path_detached) | `VARCHAR, VARCHAR -> BOOLEAN` | The same, with the program named by the second argument. |
| [`op_open_commands`](./open-path.md#op_open_commands) | `VARCHAR -> VARCHAR[]` | Lists the launch commands `op_open_path` would try, in order, running none of them. |

## Content

What a query produces before it has a file: a report, a chart, a page. Details:
[Writing content out](./write-file.md) and [Printing to stdout](./print.md).

| Function | Signature | What it does |
| --- | --- | --- |
| [`op_write_file`](./write-file.md#op_write_file) | `VARCHAR, VARCHAR -> VARCHAR` | Writes text to a file and returns its path, starting nothing. |
| [`op_write_file`](./write-file.md#op_write_file) | `VARCHAR, VARCHAR, op_write_file_options -> VARCHAR` | The same, with the directory, the file name and a name segment chosen. |
| [`op_write_file`](./write-file.md#op_write_file) | `BLOB, VARCHAR -> VARCHAR` | Binary content (an image, say), written out as the bytes it is. |
| [`op_write_file`](./write-file.md#op_write_file) | `BLOB, VARCHAR, op_write_file_options -> VARCHAR` | The same, with the options. |
| [`op_open_content`](./write-file.md#op_open_content) | the four signatures of `op_write_file`, `-> VARCHAR` | Writes it out and opens it in the same call. |
| [`op_open_html`](./write-file.md#op_open_html) | `VARCHAR -> VARCHAR` | The zero-configuration shortcut: a temporary `.html`, opened in the default browser. |
| [`op_print`](./print.md#op_print) | `VARCHAR -> VARCHAR` | Writes to stdout, past the result set's formatting, and returns the text it was given. |
| [`op_print`](./print.md#op_print) | `VARCHAR, VARCHAR -> VARCHAR` | The same, with what to append after the text instead of a newline; the tail is not part of the answer. |

## File names

Pure string work: what it answers about a value is whether that value could be a file name. Details:
[File names from data](./sanitize-filename.md).

| Function | Signature | What it does |
| --- | --- | --- |
| [`op_sanitize_filename`](./sanitize-filename.md#op_sanitize_filename) | `VARCHAR -> VARCHAR` | Rewrites a string into something usable as a file name. |
| [`op_sanitize_filename`](./sanitize-filename.md#op_sanitize_filename) | `VARCHAR, VARCHAR -> VARCHAR` | The same, with what to substitute for each illegal character instead of deleting it. |
| [`op_sanitize_filename`](./sanitize-filename.md#op_sanitize_filename) | `VARCHAR, VARCHAR, BOOLEAN, BOOLEAN -> VARCHAR` | The same, plus the two switches: the 255-byte cut and the Windows rules. |
| [`op_sanitize_filename_check`](./sanitize-filename.md#op_sanitize_filename_check) | `VARCHAR -> BOOLEAN` | Rewrites nothing: is this string already a usable name? |
| [`op_sanitize_filename_check`](./sanitize-filename.md#op_sanitize_filename_check) | `VARCHAR, BOOLEAN, BOOLEAN -> BOOLEAN` | The same question, with the two switches. |

That is the whole API, and one query lists it as DuckDB sees it:

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'op%'
ORDER BY 1, 2;    -- nine names, twenty-one signatures
```

## Notes

- **A desktop has to be behind a launch.** The launcher is desktop software, so on a machine that has none —
  a server, a CI runner — the four `op_open_path` actions answer `false`, `op_open_content` and
  `op_open_html` raise an error carrying the path they did write, and `op_open_commands` is the one call
  that still answers there, listing what would be tried. `op_write_file`, `op_print` and the filename
  functions need no desktop at all.
- **NULL in, NULL out — and nothing started, nothing written.** That is what makes the answers readable:
  NULL is a row with no input. In `op_sanitize_filename` and `op_sanitize_filename_check` it means NULL in
  **any** argument, the switches included.
- **A rejected input fails the statement.** The error names the function, and the rest of the query is not
  evaluated. A value that is merely unusable is not an error anywhere here — it gets an answer.
- **Constants are folded into one execution.** `SELECT op_open_path('report.html')` runs once. As soon as
  the argument comes from a column the call runs per row, so `SELECT op_open_path(p) FROM reports` opens one
  window per row, with no deduplication. `op_write_file` per row means one file per row, which is what the
  generated name is for.
- **The list keeps its type.** `op_open_commands` returns `VARCHAR[]`, so its elements carry the quoting they
  are launched with; index into it (`op_open_commands('report.html')[1]`) or `unnest` it for one command per
  row.
- **The filename functions answer the same way on every machine.** Nothing in them depends on the
  filesystem, on a desktop, or on what a window happened to do, so a rewritten name is verbatim checkable in
  a test — which is also why `windows` is pinned to `true` instead of following the host platform.
