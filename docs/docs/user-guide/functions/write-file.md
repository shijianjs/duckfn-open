---
title: Writing content out
sidebar_position: 2
description: op_write_file, op_open_content and op_open_html — turning content into a file, and maybe opening it.
---

# Writing content out

These three start from **content** rather than from a path, which is the shape of most of what a query
produces. The content becomes a file, and the function returns the path it wrote: from there it is an
ordinary path, and the [launch functions](./open-path.md) can take it.

Because the path is the return value, a failure has to be written somewhere else than a return value.
`op_write_file` starts nothing, so its answers are about the file: a path, NULL for a row with no input, and
an error for an argument it cannot turn into a file name. `op_open_content` and `op_open_html` add a launch
to that, and a launcher that cannot start is reported as an **error carrying the path** — the file is on
disk, this machine just has nothing to show it with:

```sql
SELECT op_write_file('report body', 'md');
-- /tmp/open-20260101-120000-9Qm2Zk.md  — written, nothing started

SELECT op_open_content(svg, 'svg');
-- on a desktop: the path, and a viewer showing it
-- on a headless machine: an error, "the file was written to '<path>', but no launcher could be started"
```

An argument that cannot become a file name is an error too, one message per setting:

```sql
SELECT op_write_file('a', '');                       -- error: op_write_file: the extension must not be empty …
SELECT op_write_file('a', 'txt', {'dir':'  ', 'name':NULL, 'part':NULL});
                                                  -- error: … op_write_file_options.dir must not be empty
SELECT op_open_content('a', 'txt', {'dir':'./out', 'name':'/', 'part':NULL});
                                                  -- error: … op_write_file_options.name is not a usable file name
```

## op_write_file

```text
op_write_file(content VARCHAR, extension VARCHAR) -> VARCHAR
op_write_file(content VARCHAR, extension VARCHAR, opts op_write_file_options) -> VARCHAR
op_write_file(content BLOB,    extension VARCHAR) -> VARCHAR
op_write_file(content BLOB,    extension VARCHAR, opts op_write_file_options) -> VARCHAR
```

Turns content into a file and returns the path it wrote. It starts no process, and that is what makes it
composable: hand the path to [`op_open_path`](./open-path.md#op_open_path) to choose the program, or to
[`op_open_content`](#op_open_content) for the default one.

```sql
SELECT op_write_file('{"a":1}', 'json');
SELECT op_write_file(svg, 'svg', {'dir':'./out', 'name':'chart', 'part':NULL});
SELECT op_write_file(read_blob('logo.png'), 'png');
```

The **extension is the one required setting**, because it is what decides how the system treats the file:
`html` and `.html` both mean `.html`, and an empty one — or one carrying a path separator, which would be a
path in disguise — is an error rather than a guess. For a BLOB it matters even more: without a suffix the
viewer has no idea what it is looking at.

Everything else has a default:

- **`dir`** — the system temp directory when unset. A directory that does not exist is created with its
  parents, because someone writing `file.dir = '/tmp/charts'` means "this is where it should go", not "this
  already exists". An empty string is an error.
- **`name`** — used verbatim, with `.svg`-style suffix completion when it carries no extension, and **an
  existing file at that path is overwritten**: you named it, that is the path you wanted.
- With no `name`, one is generated as `open-<time>-<part>-<random>.<extension>`:
  `open-20260101-120000-Sales_Overview-9Qm2Zk.svg`. The time comes right after the prefix, so sorting a
  directory by name sorts it by time; the random suffix is what keeps two charts made in the same second
  apart, and it also guarantees an auto-generated name never lands on an existing file.
- **`part`** — the meaningful bit that goes into a generated name (a chart title, a report name), where the
  alternative is a directory full of timestamps. It is capped at 48 characters.

All three options are treated as file-name material: illegal characters, control characters, Windows
reserved device names and trailing dots and spaces are replaced, and spaces inside `part` become `_`. A
`part` carrying a slash therefore cannot escape its directory, and a title in Chinese or with punctuation
still fits in a name.

A directory and a file name are always joined with `/`, on every platform, because that string is read by a
human and usually concatenated further by a downstream query — and `std::fs` accepts it on Windows too. What
the directory part looks like is what you gave: a `dir` you wrote is used as written, and when it defaults
to the system temp directory you get that directory's own shape
(`C:\Users\…\Temp/open-….json` on Windows).

### op_write_file_options

`op_write_file_options` is a STRUCT type the extension registers, and it is the third argument of the
signatures that carry options:

```text
op_write_file_options(dir VARCHAR, "name" VARCHAR, part VARCHAR)
```

The field is displayed as `"name"` because NAME is a keyword in DuckDB's grammar; writing `name:` or
`'name':` in a struct literal both work.

**All three fields are optional, but a literal that omits one needs a cast.** DuckDB does not implicitly
widen `{'dir':'./out'}` into a three-field STRUCT, so write either every field or the cast:

```sql
SELECT op_write_file(svg, 'svg', {'dir':'./out', 'name':NULL, 'part':'sales'});
SELECT op_write_file(svg, 'svg', {'dir':'./out'}::op_write_file_options);
```

A whole STRUCT passed as NULL is a NULL argument, so it short-circuits like a NULL content does — it is not
"use the defaults". Omit the third argument for that.

## op_open_content

```text
op_open_content(content VARCHAR, extension VARCHAR) -> VARCHAR
op_open_content(content VARCHAR, extension VARCHAR, opts op_write_file_options) -> VARCHAR
op_open_content(content BLOB,    extension VARCHAR) -> VARCHAR
op_open_content(content BLOB,    extension VARCHAR, opts op_write_file_options) -> VARCHAR
```

[`op_write_file`](#op_write_file) and a launch in one call, which is the single most common thing to do
while exploring: the query has content, and you want to look at it.

```sql
SELECT op_open_content(read_text('report.html'), 'html');
SELECT op_open_content(svg, 'svg', {'dir':'./out', 'name':NULL, 'part':'sales'});
SELECT op_open_content(logo_blob, 'png');
```

Every persistence rule above is the same one it shares with `op_write_file`. The launch it adds brings two
behaviours: it does not wait for the program to exit, and a launcher that cannot start is an error carrying
the path rather than the `false` [`op_open_path`](./open-path.md#op_open_path) would answer with.

Choosing the program is deliberately not this function's job; the options STRUCT has no `app` field for the
same reason. Compose it instead:

```sql
SELECT op_open_path(op_write_file(html, 'html'), 'firefox');
```

## op_open_html

```text
op_open_html(content VARCHAR) -> VARCHAR
```

[`op_open_content`](#op_open_content) with all three options left at their default and the extension pinned
to `html`: a temporary file, opened in the system default browser, path returned.

```sql
SELECT op_open_html('<h1>hello</h1>');
SELECT op_open_html(string_agg(row, chr(10))) FROM rendered_rows t(row);
```

It exists because what SQL renders is HTML nine times out of ten, and typing the extension plus an empty
STRUCT each time is noise. When you need a directory, a name, a recognisable segment in the name, or a
browser other than the default, that is `op_open_content(content, 'html', opts)` or the `op_open_path`
composition above. There is no BLOB signature here: the input is HTML *text*.
