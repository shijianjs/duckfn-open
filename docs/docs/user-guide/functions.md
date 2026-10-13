---
title: Functions
sidebar_position: 3
description: Every SQL function duckfn_open registers, with examples to run in your own DuckDB.
---

# Functions

Loading the extension registers three function names, five signatures, and every one of them is a scalar
function: there is no aggregate function, no table function and no custom type here. They behave like
DuckDB's own: use them in a projection or a `WHERE` clause, and they combine with built-in functions
freely.

| Function | Kind | Signature | Summary |
| --- | --- | --- | --- |
| [`open_path`](#open_path) | scalar | `VARCHAR -> BOOLEAN` | Opens a path or URL with the default program, and waits for the launcher. |
| [`open_path`](#open_path) | scalar | `VARCHAR, VARCHAR -> BOOLEAN` | The same, with the program named by the second argument. |
| [`open_path_detached`](#open_path_detached) | scalar | `VARCHAR -> BOOLEAN` | Opens it and returns as soon as the launcher has been started. |
| [`open_path_detached`](#open_path_detached) | scalar | `VARCHAR, VARCHAR -> BOOLEAN` | The same, with the program named by the second argument. |
| [`open_commands`](#open_commands) | scalar | `VARCHAR -> VARCHAR[]` | Lists the launch commands `open_path` would try, in order, running none of them. |

The block below is the whole API as DuckDB sees it, once the extension is loaded:

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'open%'
ORDER BY 1, 2;    -- three names, five signatures
```

`open_path` and `open_path_detached` each answer for two argument counts under one name, so the
two-argument form has no SQL name of its own: the call is `SELECT open_path('a', 'b')`, not
`open_path_with('a', 'b')`.

## What one call answers

Three outcomes are distinguishable from SQL, and together they are the whole contract:

| Result | Meaning |
| --- | --- |
| `NULL` | The row had no input. A NULL never reaches a launcher, so it is not a failure — it means nothing was asked of the desktop. |
| `true` | A launcher was started. |
| `false` | No launcher could be started: this machine has none configured, or the program named in the second argument does not exist. |

```sql
SELECT open_path(NULL) AS opened;    -- NULL: no input on this row, so nothing was launched
```

NULL is answered per row, not only for a literal: a column full of NULLs launches nothing at all.

```sql
SELECT count(*) FROM reports WHERE open_path(path);    -- true keeps the row, false and NULL drop it
```

The reason a failure is only `false` is that the launcher's own problem report has nowhere to go in SQL.
[open_commands](#open_commands) is the answer to "why did nothing open".

## open_path

```text
open_path(path VARCHAR) -> BOOLEAN
open_path(path VARCHAR, app VARCHAR) -> BOOLEAN
```

Opens a local path or a URL with whatever the desktop is configured to open it with — the same program a
double-click hands the file to, which for a URL is the default browser. The second signature names the
program instead.

```sql
SELECT open_path('report.html');
SELECT open_path('https://duckdb.org', 'firefox');
SELECT open_path('/data/report.csv', 'LibreOffice Calc');
```

**This call waits.** It comes back when the launcher does, and on Linux the usual launcher only returns
once the program you opened the file with has exited. That is the difference between this name and
[`open_path_detached`](#open_path_detached) — the one thing that visibly changes how long a query takes.
On Windows it goes the other way: `open_path_detached` is the more direct of the two, because its launcher
step is a single native call rather than a helper process with a fallback behind it.

The `app` argument is passed to the platform exactly as you wrote it: an executable on your `PATH` on
Linux, on Windows a name looked up among the installed applications first and handed to the shell
after that. No aliases are resolved and no existence check is performed, so a program that cannot be
started is simply a `false`.

## open_path_detached

```text
open_path_detached(path VARCHAR) -> BOOLEAN
open_path_detached(path VARCHAR, app VARCHAR) -> BOOLEAN
```

The non-blocking pair of `open_path`: it starts the launcher and returns immediately.

```sql
SELECT open_path_detached('https://duckdb.org');
SELECT count(*) FROM reports WHERE open_path_detached(url);
```

The price is a looser `true`. It says a process was started, not that the file ended up on screen — with
nothing waited on, there is no outcome left to report. Use `open_path` when you need the launcher's own
answer, and this one when a query should not sit there until a window is closed.

## open_commands

```text
open_commands(path VARCHAR) -> VARCHAR[]
```

The only action in this extension that starts nothing. It returns, as a list of strings, the launch
commands `open_path` would try on this machine, in the order it would try them: the next candidate only
runs when the previous one could not be started.

```sql
SELECT open_commands('report.html');
-- on Linux:   ['"xdg-open" "report.html"', '"/usr/bin/xdg-open" "report.html"', …]
-- on Windows: ['"powershell.exe" "-NoProfile" …', '"explorer.exe" "report.html"']
```

Two things follow from that shape:

- It answers "what command would run here", not "will the program succeed". The exit status of a command
  is only visible to something that actually runs it, which is `open_path`'s job.
- Its result depends only on the machine's configuration, never on whether a window happened to open, so it
  is the one call that is safe to put in a test or run on a server.

Because it returns a list, SQL can work over the candidates directly:

```sql
SELECT unnest(open_commands('report.html')) AS would_run;
```

## Errors

An empty path or an empty program name is an error, not a `false` — a typo and an unopenable file deserve
different answers. DuckDB reports it as an invalid input error, with the function's own name in front of the
message:

```sql
SELECT open_path('');    -- error: open_path: the path or URL must not be empty
```

The second argument has its own message (`… the application must not be empty`), and the argument count
and types are checked while the query is being bound: `open_path()`, `open_path('a', 'b', 'c')` and
`open_path(42)` all fail before anything runs.

## Notes

- **A desktop has to be behind the call.** The launcher is desktop software, so on a machine that has none —
  a server, a CI runner — the four launch actions answer `false`, and `open_commands` is the one call that
  still answers, listing what would be tried there.
- **NULL in, NULL out — and nothing launched.** That is what makes the three outcomes readable: NULL is a
  row with no input, `false` is a machine that could not start a launcher, `true` is a launcher that was
  started.
- **A rejected input fails the statement.** The error names the function, and the rest of the query is not
  evaluated.
- **Constants are folded into one execution.** `SELECT open_path('report.html')` runs once. As soon as the
  argument comes from a column, the call runs per row — `SELECT open_path(p) FROM reports` opens one
  window per row, with no deduplication.
- **The list is printed with its type.** `open_commands` returns `VARCHAR[]`, so its elements keep the
  quoting they are launched with; index into it (`open_commands('report.html')[1]`) or `unnest` it when you
  want one command per row.
