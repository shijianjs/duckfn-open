---
title: Opening a path
sidebar_position: 2
description: op_open_path, op_open_path_detached and op_open_commands — handing a path or a URL to the desktop.
---

# Opening a path

These three take something that already exists — a file, a directory, a URL — and hand it to the desktop.
They are the only calls in the extension that start a process.

A launch answers one of three things:

| Result | Meaning |
| --- | --- |
| `NULL` | The row had no input. A NULL never reaches a launcher, so nothing was asked of the desktop. |
| `true` | A launcher was started. |
| `false` | No launcher could be started: this machine has none configured, or the program named in the second argument does not exist. |

```sql
SELECT op_open_path(NULL) AS opened;    -- NULL: no input on this row, so nothing was launched
```

NULL is answered per row, not only for a literal: a column full of NULLs launches nothing at all.

```sql
SELECT count(*) FROM reports WHERE op_open_path(path);    -- true keeps the row, false and NULL drop it
```

A failure is only a `false` because the launcher's own problem report has nowhere to go in SQL.
[op_open_commands](#op_open_commands) is the answer to "why did nothing open".

## op_open_path

```text
op_open_path(path VARCHAR) -> BOOLEAN
op_open_path(path VARCHAR, app VARCHAR) -> BOOLEAN
```

Opens a local path or a URL with whatever the desktop is configured to open it with — the same program a
double-click hands the file to, which for a URL is the default browser. The second signature names the
program instead.

```sql
SELECT op_open_path('report.html');
SELECT op_open_path('https://duckdb.org', 'firefox');
SELECT op_open_path('/data/report.csv', 'LibreOffice Calc');
```

**This call waits.** It comes back when the launcher does, and on Linux the usual launcher only returns once
the program you opened the file with has exited — which is the one place where a query can look like it has
hung. That is the difference between this name and [`op_open_path_detached`](#op_open_path_detached). On
Windows it goes the other way: there `op_open_path_detached` is the more direct of the two, because its
launcher step is a single native call rather than a helper process with a fallback behind it.

The `app` argument is passed to the platform exactly as you wrote it: an executable on your `PATH` on Linux,
on Windows a name looked up among the installed applications first and handed to the shell after that. No
aliases are resolved and no existence check is performed, so a program that cannot be started is simply a
`false`.

An empty path, or an empty program name, is an error rather than a `false` — a typo and an unopenable file
deserve different answers:

```sql
SELECT op_open_path('');    -- error: op_open_path: the path or URL must not be empty
```

The second argument has its own message (`… the application must not be empty`). The argument count and the
argument types are checked while the query is being bound: `op_open_path()`, `op_open_path('a', 'b', 'c')`
and `op_open_path(42)` all fail before anything runs.

## op_open_path_detached

```text
op_open_path_detached(path VARCHAR) -> BOOLEAN
op_open_path_detached(path VARCHAR, app VARCHAR) -> BOOLEAN
```

The non-blocking pair of [`op_open_path`](#op_open_path): it starts the launcher and returns immediately.

```sql
SELECT op_open_path_detached('https://duckdb.org');
SELECT count(*) FROM reports WHERE op_open_path_detached(url);
```

The price is a looser `true`. It says a process was started, not that the file ended up on screen — with
nothing waited on, there is no outcome left to report. Use `op_open_path` when you need the launcher's own
answer, and this one when a query should not sit there until a window is closed.

[`op_open_content`](./write-file.md#op_open_content) and `op_open_html` take this non-blocking route too:
writing a chart and opening it should not hang until the viewer is closed.

## op_open_commands

```text
op_open_commands(path VARCHAR) -> VARCHAR[]
```

The only call in this group that starts nothing. It returns, as a list of strings, the launch commands
`op_open_path` would try on this machine, in the order it would try them: the next candidate only runs when
the previous one could not be started.

```sql
SELECT op_open_commands('report.html');
-- on Linux:   ['"xdg-open" "report.html"', '"/usr/bin/xdg-open" "report.html"', …]
-- on Windows: ['"powershell.exe" "-NoProfile" …', '"explorer.exe" "report.html"']
```

Two things follow from that shape:

- It answers "what command would run here", not "will the program succeed". The exit status of a command is
  only visible to something that actually runs it, which is `op_open_path`'s job.
- Its result depends only on the machine's configuration, never on whether a window happened to open, so it
  is the launch answer that is safe to put in a test or run on a server.

Because it returns a list, SQL can work over the candidates directly:

```sql
SELECT unnest(op_open_commands('report.html')) AS would_run;
```
