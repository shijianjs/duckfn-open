---
title: Functions
sidebar_position: 3
description: Every SQL function duckfn_open registers, with examples to run in your own DuckDB.
---

# Functions

Loading the extension registers seven function names and sixteen signatures, and every one of them is a
scalar function: there is no aggregate function and no table function here. It also registers one STRUCT
type, [`open_write_options`](#the-write-options) — that is where the optional arguments of the write
family live. The functions behave like DuckDB's own: use them in a projection or a `WHERE` clause, and
they combine with built-in functions freely.

The names fall into two groups. The first takes a path or URL that already exists and hands it to the
desktop:

| Function | Kind | Signature | Summary |
| --- | --- | --- | --- |
| [`open_path`](#open_path) | scalar | `VARCHAR -> BOOLEAN` | Opens a path or URL with the default program, and waits for the launcher. |
| [`open_path`](#open_path) | scalar | `VARCHAR, VARCHAR -> BOOLEAN` | The same, with the program named by the second argument. |
| [`open_path_detached`](#open_path_detached) | scalar | `VARCHAR -> BOOLEAN` | Opens it and returns as soon as the launcher has been started. |
| [`open_path_detached`](#open_path_detached) | scalar | `VARCHAR, VARCHAR -> BOOLEAN` | The same, with the program named by the second argument. |
| [`open_commands`](#open_commands) | scalar | `VARCHAR -> VARCHAR[]` | Lists the launch commands `open_path` would try, in order, running none of them. |

The second group starts from **content** rather than from a path, which is the shape of most of what a
query produces: a report, a chart, a page that has no file yet.

| Function | Kind | Signature | Summary |
| --- | --- | --- | --- |
| [`open_write`](#open_write) | scalar | `VARCHAR, VARCHAR -> VARCHAR` | Writes text to a file and returns its path, starting nothing. |
| [`open_write`](#open_write) | scalar | `VARCHAR, VARCHAR, open_write_options -> VARCHAR` | The same, with the directory, the file name and a name segment chosen. |
| [`open_write`](#open_write) | scalar | `BLOB, VARCHAR -> VARCHAR` | Binary content (an image, say), written out as the bytes it is. |
| [`open_write`](#open_write) | scalar | `BLOB, VARCHAR, open_write_options -> VARCHAR` | The same, with the options. |
| [`open_content`](#open_content) | scalar | the four signatures of `open_write`, `-> VARCHAR` | Writes it out and opens it in the same call: `open_write` composed with `open_path`. |
| [`open_html`](#open_html) | scalar | `VARCHAR -> VARCHAR` | The zero-configuration shortcut: a temporary `.html` opened in the default browser. |
| [`open_print`](#open_print) | scalar | `VARCHAR -> VARCHAR` | Writes to stdout, past the result set's formatting. Always returns NULL. |
| [`open_print`](#open_print) | scalar | `VARCHAR, VARCHAR -> VARCHAR` | The same, with what to append after the text instead of a newline. |

The block below is the whole API as DuckDB sees it, once the extension is loaded:

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'open%'
ORDER BY 1, 2;    -- seven names, sixteen signatures
```

Five of the names carry more than one signature under a single SQL name, so none of the extra forms has a
SQL name of its own: the calls are `SELECT open_path('a', 'b')` and
`SELECT open_write(html, 'html', opts)`, not `open_path_with('a', 'b')` or `open_write_with(…)`.

## What one call answers

The two groups answer differently, and each of those is a small contract of its own.

**The launch family** (`open_path`, `open_path_detached`) distinguishes three outcomes:

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

**The write family** returns a path, which changes where a failure can be written. `open_write` starts
nothing, so its three answers are about the file: a path, NULL for a row with no input, and an error for an
argument it cannot make into a file name. `open_content` and `open_html` add a launch to that, and since
the path is already their return value, a launcher that cannot start is reported as an **error carrying the
path** — the file is on disk, this machine just has nothing to show it with:

```sql
SELECT open_write('report body', 'md');
-- /tmp/open-20260101-120000-9Qm2Zk.md  — written, nothing started

SELECT open_content(svg, 'svg');
-- on a desktop: the path, and a viewer showing it
-- on a headless machine: an error, "the file was written to '<path>', but no launcher could be started"
```

**`open_print`** returns NULL always: the text went to stdout, and SQL has nothing left to say.

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

Both `open_content` and `open_html` take this non-blocking route: writing a chart and opening it should not
hang until the viewer is closed.

## open_commands

```text
open_commands(path VARCHAR) -> VARCHAR[]
```

The only action in the launch family that starts nothing. It returns, as a list of strings, the launch
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
  is the launch answer that is safe to put in a test or run on a server.

Because it returns a list, SQL can work over the candidates directly:

```sql
SELECT unnest(open_commands('report.html')) AS would_run;
```

## open_write

```text
open_write(content VARCHAR, extension VARCHAR) -> VARCHAR
open_write(content VARCHAR, extension VARCHAR, opts open_write_options) -> VARCHAR
open_write(content BLOB,    extension VARCHAR) -> VARCHAR
open_write(content BLOB,    extension VARCHAR, opts open_write_options) -> VARCHAR
```

Turns content into a file and returns the path it wrote. It starts no process — that is what makes it
composable: take the path to [`open_path`](#open_path) to choose the program, or to
[`open_content`](#open_content) for the default one.

```sql
SELECT open_write('{"a":1}', 'json');
SELECT open_write(svg, 'svg', {'dir':'./out', 'name':'chart', 'part':NULL});
SELECT open_write(read_blob('logo.png'), 'png');
```

The **extension is the one required setting**, because it is what decides how the system treats the file:
`html` and `.html` both mean `.html`, and an empty one — or one carrying a path separator, which would be a
path in disguise — is an error rather than a guess. For a BLOB it matters even more: without a suffix the
viewer has no idea what it is looking at.

Everything else has a default:

- **`dir`** — the system temp directory when unset. A directory that does not exist is created with its
  parents, because someone writing `file.dir = '/tmp/charts'` means "this is where it should go", not "this
  already exists". An empty string is an error.
- **`name`** — used verbatim, with `.svg`-style suffix completion when it carries no extension, and
  **an existing file at that path is overwritten**: you named it, that is the path you wanted.
- With no `name`, one is generated as `open-<time>-<part>-<random>.<extension>`:
  `open-20260101-120000-Sales_Overview-9Qm2Zk.svg`. The time comes right after the prefix, so sorting a
  directory by name sorts it by time; the random suffix is what keeps two charts made in the same second
  apart, and it also guarantees an auto-generated name never lands on an existing file.
- **`part`** — the meaningful bit that goes into a generated name (a chart title, a report name), where the
  alternative is a directory full of timestamps. It is capped at 48 characters.

All three options are treated as file-name material: illegal characters, control characters, Windows
reserved device names and trailing dots and spaces are replaced, spaces inside `part` become `_`. A `part`
carrying a slash therefore cannot escape its directory, and a title in Chinese or with punctuation still
fits in a name.

A directory and a file name are always joined with `/`, on every platform, because that string is read by a
human and usually concatenated further by a downstream query — and `std::fs` accepts it on Windows too. What
the directory part looks like is what you gave: a `dir` you wrote is used as written, and when it defaults
to the system temp directory you get that directory's own shape (`C:\Users\…\Temp/open-….json` on Windows).

### The write options

`open_write_options` is a STRUCT type the extension registers, and it is the third argument of the
 signatures that carry options:

```text
open_write_options(dir VARCHAR, "name" VARCHAR, part VARCHAR)
```

The field is displayed as `"name"` because NAME is a keyword in DuckDB's grammar; writing `name:` or
`'name':` in a struct literal both work.

**All three fields are optional, but a literal that omits one needs a cast.** DuckDB does not implicitly
widen `{'dir':'./out'}` into a three-field STRUCT, so write either every field or the cast:

```sql
SELECT open_write(svg, 'svg', {'dir':'./out', 'name':NULL, 'part':'sales'});
SELECT open_write(svg, 'svg', {'dir':'./out'}::open_write_options);
```

A whole STRUCT passed as NULL is a NULL argument, so it short-circuits like a NULL content does — it is
not "use the defaults". Omit the third argument for that.

## open_content

```text
open_content(content VARCHAR, extension VARCHAR) -> VARCHAR
open_content(content VARCHAR, extension VARCHAR, opts open_write_options) -> VARCHAR
open_content(content BLOB,    extension VARCHAR) -> VARCHAR
open_content(content BLOB,    extension VARCHAR, opts open_write_options) -> VARCHAR
```

`open_write` and `open_path` in one call, which is the single most common thing to do while exploring: the
query has content, and you want to look at it.

```sql
SELECT open_content(read_text('report.html'), 'html');
SELECT open_content(svg, 'svg', {'dir':'./out', 'name':NULL, 'part':'sales'});
SELECT open_content(logo_blob, 'png');
```

Every persistence rule above is the same one it shares with `open_write`. Two behaviours come from the
launch it adds: it does not wait for the program to exit, and a launcher that cannot start is an error
carrying the path rather than the `false` `open_path` would answer with — the return value is already spent
on the path, and a bare `false` would throw that path away.

Choosing the program is deliberately not this function's job; the options STRUCT has no `app` field for the
same reason. Compose it instead:

```sql
SELECT open_path(open_write(html, 'html'), 'firefox');
```

## open_html

```text
open_html(content VARCHAR) -> VARCHAR
```

[`open_content`](#open_content) with all three options left at their default and the extension pinned to
`html`: a temporary file, opened in the system default browser, path returned.

```sql
SELECT open_html('<h1>hello</h1>');
SELECT open_html(string_agg(row, chr(10))) FROM rendered_rows t(row);
```

It exists because what SQL renders is HTML nine times out of ten, and typing the extension plus an empty
STRUCT each time is noise. When you need a directory, a name, a recognisable segment in the name, or a
browser other than the default, that is `open_content(content, 'html', opts)` or the `open_path`
composition above. There is no BLOB signature here: the input is HTML *text*.

## open_print

```text
open_print(text VARCHAR) -> VARCHAR
open_print(text VARCHAR, tail VARCHAR) -> VARCHAR
```

Writes text straight to stdout and returns NULL, which bypasses the thing this is for: the DuckDB CLI
*lays out* its results — column widths, borders, wrapping — and character art (the ASCII or braille charts
an extension like `textplot` produces) is read line by line, so stuffing it into one VARCHAR cell destroys
it.

```sql
SELECT open_print(textplot(y)) FROM series;           -- one chart, on the console
SELECT open_print('rows: ' || count(*)) FROM t;       -- a note that does not occupy the result set
```

The two signatures split on what follows the text: with no `tail` a newline is added, so the next line of
output does not run into this one; with a `tail` exactly that is written — `''` means "keep going on this
line", `chr(10) || chr(10)` means "leave a blank line". Each call flushes, because Rust buffers stdout by
line and a tail without a newline would otherwise leave the text in the buffer.

Being a scalar function, it is evaluated per row, so a multi-row query prints a running log in execution
order rather than a laid-out report — the report is what the result set is for. One exception to plan
around: when the argument is a constant, DuckDB folds the call into a single evaluation, so
`SELECT open_print('x') FROM t` prints one line however many rows `t` has. Reference a column
(`open_print('x: ' || a)`) to get one line per row.

## Errors

An empty path or an empty program name is an error, not a `false` — a typo and an unopenable file deserve
different answers. DuckDB reports it as an invalid input error, with the function's own name in front of
the message:

```sql
SELECT open_path('');    -- error: open_path: the path or URL must not be empty
```

The write family reports the same way, one message per setting it cannot use:

```sql
SELECT open_write('a', '');                       -- error: open_write: the extension must not be empty …
SELECT open_write('a', 'txt', {'dir':'  ', 'name':NULL, 'part':NULL});
                                                  -- error: … open_write_options.dir must not be empty
SELECT open_content('a', 'txt', {'dir':'./out', 'name':'/', 'part':NULL});
                                                  -- error: … open_write_options.name is not a usable file name
```

The second argument of `open_path` has its own message (`… the application must not be empty`), and the
argument count and types are checked while the query is being bound: `open_path()`,
`open_path('a', 'b', 'c')` and `open_path(42)` all fail before anything runs.

## Notes

- **A desktop has to be behind a launch.** The launcher is desktop software, so on a machine that has none
  — a server, a CI runner — the four `open_path` actions answer `false`, `open_content` and `open_html`
  raise an error carrying the path they did write, and `open_commands` is the one call that still answers,
  listing what would be tried there. `open_write` and `open_print` need no desktop at all.
- **NULL in, NULL out — and nothing started, nothing written.** That is what makes the outcomes readable:
  NULL is a row with no input.
- **A rejected input fails the statement.** The error names the function, and the rest of the query is not
  evaluated.
- **Constants are folded into one execution.** `SELECT open_path('report.html')` runs once. As soon as the
  argument comes from a column, the call runs per row — `SELECT open_path(p) FROM reports` opens one window
  per row, with no deduplication. `open_write` per row means one file per row, which is exactly what the
  generated name is for.
- **The list is printed with its type.** `open_commands` returns `VARCHAR[]`, so its elements keep the
  quoting they are launched with; index into it (`open_commands('report.html')[1]`) or `unnest` it when you
  want one command per row.
