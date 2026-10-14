---
title: Printing to stdout
sidebar_position: 3
description: op_print — text written straight to stdout, past the result set's formatting, and handed back.
---

# Printing to stdout

## op_print

```text
op_print(text VARCHAR) -> VARCHAR
op_print(text VARCHAR, tail VARCHAR) -> VARCHAR
```

Writes text straight to stdout, and returns that same text. The console gets it as it is, which bypasses the
thing this is for: the DuckDB CLI *lays out* its results — column widths, borders, wrapping — and character
art (the ASCII or braille charts an extension like `textplot` produces) is read line by line, so stuffing it
into one VARCHAR cell destroys it. The value is not spent by the trip: the call can stay in a projection, or
be concatenated further, with the text still there.

```sql
SELECT op_print(textplot(y)) FROM series;           -- the chart on the console, the same string in the column
SELECT op_print('rows: ' || count(*)) FROM t;       -- a note that is also the value of that column
```

Because the text comes back, a call in a projection puts it in the result set too, laid out as always. That
is the trade of returning the value. When only the console should see it, keep the call out of the SELECT
list — `SELECT count(*) FROM series WHERE op_print(textplot(y)) IS NOT NULL` prints every chart and leaves a
count in the result set.

The two signatures split on what follows the text: with no `tail` a newline is added, so the next line of
output does not run into this one; with a `tail` exactly that is written — `''` means "keep going on this
line", `chr(10) || chr(10)` means "leave a blank line". The tail is not part of the return value. Each call
flushes, because Rust buffers stdout by line and a tail without a newline would otherwise leave the text in
the buffer.

Being a scalar function, it is evaluated per row, so a multi-row query prints a running log in execution
order rather than a laid-out report — the report is what the result set is for. One exception to plan
around: when the argument is a constant, DuckDB folds the call into a single evaluation, so
`SELECT op_print('x') FROM t` prints one line however many rows `t` has. Reference a column
(`op_print('x: ' || a)`) to get one line per row.

NULL in, NULL out: a row with no input prints nothing and answers NULL.
