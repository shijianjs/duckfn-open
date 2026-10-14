---
title: File names from data
sidebar_position: 4
description: op_sanitize_filename and op_sanitize_filename_check — turning a value into a usable file name.
---

# File names from data

These two touch neither the desktop nor the filesystem. They answer, about a plain string, the question the
other groups run into as soon as a name comes out of the data: can this value be a file name?

What they answer with is the value itself — a rewritten string, or `true`/`false` about the string they were
given. No process starts and no file is written, so no machine can be too bare for them, and **an unusable
name is not an error**: a rewrite that leaves nothing behind is a legal answer rather than a rejected
argument.

## op_sanitize_filename

```text
op_sanitize_filename(value VARCHAR) -> VARCHAR
op_sanitize_filename(value VARCHAR, replacement VARCHAR) -> VARCHAR
op_sanitize_filename(value VARCHAR, replacement VARCHAR, truncate BOOLEAN, windows BOOLEAN) -> VARCHAR
```

Rewrites a string into something usable as a file name. Nothing else happens — no process, no file, no
directory: the value goes in, a value comes out. What the rules cover is everything that makes a string
unusable as a name: the illegal characters `/ \ ? < > : * | "`, the control characters, a name made only of
dots (`.` and `..`), the Windows reserved device names (`CON`, `PRN`, `AUX`, `NUL`, `COM0`–`COM9`,
`LPT0`–`LPT9`, and those names with a suffix on them), trailing dots and spaces, and the 255-**byte** cut.

A query wants this because its file names usually come from the data itself — a customer name, a report
title, the `path` column that was crawled in. Turning a value into a legal name is what makes the
directory, the URL and the next extension composable.

```sql
SELECT op_sanitize_filename('quarterly/report:v2.csv');   -- 'quarterlyreportv2.csv'
SELECT op_sanitize_filename('  the report.  ');           -- '  the report' — the tail goes, the head stays
SELECT op_sanitize_filename('CON.txt');                   -- '' — a reserved device name is not a name
SELECT op_sanitize_filename('..');                        -- ''
```

The defaults are that library's own, with **one deliberate exception**. `replacement` is the empty string,
so illegal characters are **deleted**, not substituted; `truncate` is on. `windows`, though, is **pinned to
`true` whatever the host platform is**: its default in the library is the platform it was compiled for, and
that would let one and the same SQL statement answer differently on different machines. Pass `false` there
for the Unix rules — under which `CON.txt` is just a file name:

```sql
SELECT op_sanitize_filename('a/b:c?.txt', '_');    -- 'a_b_c_.txt' — the substitution shows where it edited
SELECT op_sanitize_filename('sales|east', '-');    -- 'sales-east'; any string works, including ''
SELECT op_sanitize_filename('CON.txt', '_', true, false);   -- 'CON.txt'
SELECT length(op_sanitize_filename(repeat('a', 300)));                 -- 255
SELECT length(op_sanitize_filename(repeat('a', 300), '_', false, true));  -- 300
```

The substitution is only about display: `'_'` keeps the shape of the name readable, `''` (the default, or
written out) is the same rewrite with the holes closed.

**`truncate` and `windows` can only be given together**, as the third and fourth arguments of the four
argument form; there is deliberately **no three-argument form**, so `op_sanitize_filename('a.txt', '_', true)`
is a binder error:

```sql
SELECT op_sanitize_filename('a.txt', '_', true);          -- Binder Error: truncate and windows are a pair
```

This is the one compression in the API rather than a missing case: two BOOLEANs in optional positions are
five combinations, and of those only the two the library defaults to deserve a shorter form.

NULL in **any** argument makes that row NULL. There is nothing to guess a missing option from — a truncated
name and a name that was never asked about are different things.

These are not helpers of [`op_write_file`](./write-file.md#op_write_file). That family's own naming policy
is a second set of defaults living inside the writer — `name` substitutes `_` and never truncates, `part`
collapses spaces to `_` and caps at 48 characters — and it was never a SQL name of its own. Neither set is a
stand-in for the other.

## op_sanitize_filename_check

```text
op_sanitize_filename_check(value VARCHAR) -> BOOLEAN
op_sanitize_filename_check(value VARCHAR, truncate BOOLEAN, windows BOOLEAN) -> BOOLEAN
```

The half that rewrites nothing: `true` means `op_sanitize_filename` with the same switches would hand back
exactly the value it was given.

```sql
SELECT op_sanitize_filename_check('report.csv');   -- true
SELECT op_sanitize_filename_check('a/b.txt');      -- false
SELECT op_sanitize_filename_check('CON.txt');      -- false
SELECT op_sanitize_filename_check('trail. ');      -- false
SELECT count(*) FROM files WHERE NOT op_sanitize_filename_check(name);   -- how many would be touched
```

That last line is the reason the pair exists: a query can count what a rewrite would move without moving any
of it, which is the audit and filter step before anyone decides what to do about the data.

There is no `replacement` argument here, because a question that rewrites nothing has nothing to substitute.
Everything else carries over from [`op_sanitize_filename`](#op_sanitize_filename): the same two switches,
only available together (so the options form takes three arguments, not two); `windows` pinned to `true`, so
`op_sanitize_filename_check('CON.txt')` is `false` even on Linux, where nothing reserves that name —
`op_sanitize_filename_check('CON.txt', true, false)` answers `true`; and NULL in any argument, NULL out.

Here, too, the options form has no shorter variant, and one shape is worth tripping over on purpose:

```sql
SELECT op_sanitize_filename_check('a.txt', true);         -- Binder Error: the options form takes three arguments
```

Nothing in these two functions is an error about the *value*: an illegal character, a reserved device name,
a 300-character title all get an answer, not a message.
