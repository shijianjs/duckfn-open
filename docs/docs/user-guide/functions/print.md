---
title: Printing to stdout
sidebar_position: 4
description: op_print — text written straight to stdout, past the result set's formatting, and handed back.
---

# Printing to stdout

## op_print

```text
op_print(text VARCHAR) -> VARCHAR
op_print(text VARCHAR, tail VARCHAR) -> VARCHAR
```

Writes text straight to stdout, and returns that same text. The console gets it as it is: the DuckDB CLI
*lays out* its results — column widths, borders, wrapping — and character art is read line by line, so one
VARCHAR cell destroys it. A QR code makes that concrete:

```sql
INSTALL textplot FROM community;
LOAD textplot;

SELECT op_print(tp_qr('https://query.farm'));
-- the code, one row of blocks per line, as it was rendered
```

The console copy is the point; that is the output this function exists for. The text also comes back, so the
same call puts a second copy in the result set, where the CLI lays it out like any other VARCHAR — a QR code
there is wrapped across lines and no longer scans. That copy is easy to be rid of, because the call is an
ordinary VARCHAR expression and anything that consumes a string consumes it:

```sql
-- the code on the console, one number in the result set
SELECT length(op_print(tp_qr('https://query.farm')));

-- keep the value instead, when the note is the column you wanted
SELECT op_print('rows: ' || count(*)) AS note FROM t;

-- or keep the call out of the SELECT list altogether
SELECT count(*) FROM (VALUES ('https://query.farm')) t(url) WHERE op_print(tp_qr(url)) IS NOT NULL;
```

The two signatures split on what follows the text: with no `tail` a newline is added, so the next line of
output does not run into this one; with a `tail` exactly that is written — `''` means "keep going on this
line", `chr(10) || chr(10)` means "leave a blank line". The tail is not part of the return value. Each call
flushes, because Rust buffers stdout by line and a tail without a newline would otherwise leave the text in
the buffer.

Being a scalar function, it is evaluated per row, so a multi-row query prints a running log in execution
order rather than a laid-out report — the report is what the result set is for. Nothing is folded or cached
either: `op_print` is registered volatile, so `SELECT op_print('x') FROM t` prints one line per row, constant
argument or not. The same flag is why a call inside a larger expression is evaluated exactly once —
`length(op_print(…))` prints one copy, not two — and why `EXPLAIN` prints nothing at all: a query that is only
being planned never runs it.

NULL in, NULL out: a row with no input prints nothing and answers NULL.
