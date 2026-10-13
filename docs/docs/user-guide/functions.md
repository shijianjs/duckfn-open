---
title: Functions
sidebar_position: 3
description: Every SQL function my_extension registers, with examples you can run in the browser.
---

# Functions

Loading the extension registers three functions. They behave like DuckDB's own: use them in a projection,
a `WHERE` clause or a `GROUP BY`, and they combine with built-in functions freely.

| Function | Kind | Signature | Summary |
| --- | --- | --- | --- |
| `my_greet` | scalar | `VARCHAR -> VARCHAR` | Greets a name; never NULL. |
| `my_greet_checked` | scalar | `VARCHAR -> VARCHAR` | As above, validating the name. |
| `my_sum` | aggregate | `DOUBLE -> DOUBLE` | Sums a column, skipping NULLs. |

## my_greet

```text
my_greet(name VARCHAR) -> VARCHAR
```

Greets `name`. Never returns NULL.

```sql {"type":"duckfn","show":"table"}
SELECT my_greet('world') AS greeting;
```

## my_greet_checked

```text
my_greet_checked(name VARCHAR) -> VARCHAR
```

Like `my_greet`, but it validates the name. An empty string comes back as SQL `NULL`; a name with
surrounding whitespace fails the query.

```sql {"type":"duckfn","show":"table"}
SELECT name AS input, my_greet_checked(name) AS greeting
FROM (VALUES ('world'), ('')) t(name);
```

```sql {"type":"duckfn","expect":"error"}
SELECT my_greet_checked(' x ');    -- error: the name must not have surrounding whitespace
```

## my_sum

```text
my_sum(value DOUBLE) -> DOUBLE
```

An aggregate. It ignores NULL inputs, and a group with no usable value comes back as SQL `NULL` rather
than `0`.

```sql {"type":"duckfn","show":"table"}
SELECT grp, my_sum(x) AS total
FROM (VALUES ('rows', 1.5::DOUBLE), ('rows', 2.5), ('all NULL', NULL::DOUBLE)) t(grp, x)
GROUP BY grp
ORDER BY grp;
```

## Notes

- **NULL in, NULL out.** `my_greet_checked('')` is `NULL`, not an empty string, and `my_sum` skips NULL
  rows — so a group that is entirely NULL yields `NULL`, which is not the same as `0`.
- **A rejected input fails the statement.** The error names the function, and the rest of the query is
  not evaluated.
- **Numbers print with their type.** `my_sum` returns a `DOUBLE`, so `7` shows up as `7.0`; cast it
  (`my_sum(x)::DECIMAL(10,1)`) when you want another shape.
