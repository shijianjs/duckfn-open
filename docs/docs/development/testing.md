---
title: Testing
sidebar_position: 4
description: The SQLLogicTest files in test/sql, how to run them (make, just, or the test runner directly), and what a new function should cover.
---

# Testing

Tests are [SQLLogicTest](https://duckdb.org/dev/sqllogictest/intro) files under `test/sql/`, the same
format DuckDB uses for its own test suite. A file is a sequence of `query` and `statement` blocks with
the expected result inline, so a test doubles as a worked example of the function's behaviour.

| File | What it covers |
| --- | --- |
| `my_extension.test` | Smoke test: the function is missing before `LOAD`, the sample functions exist after `require`. Also the smallest proof that the extension loads at all. |
| `scalar_greet.test` | Scalars: ordinary values (non-ASCII included), a constant `NULL` folded to `NULL`, runtime `NULL` rows short-circuited, both `DuckOptionResult` paths (NULL and error), binder errors for wrong arity and type, inputs spanning several DataChunks. |
| `aggregate_sum.test` | Aggregates: the return type, NULL rows skipped, an empty group yielding `NULL`, per-group results under `GROUP BY`, `combine` across DataChunks. |

## Running them

How a file reaches the runner:

```mermaid
flowchart LR
    file["test/sql/*.test"] --> req["require my_extension<br/>loads the artifact"]
    req --> blocks["query / statement blocks<br/>expected output inline"]
    blocks --> runner["duckdb_sqllogictest<br/>compares and reports"]
```

```shell
just test                 # = make configure + make debug + make test
just ci-build             # just the official build, without the tests
```

`just test` goes through DuckDB's official Makefile flow, which is also what CI runs. Two things to
know about it:

- **It does not rebuild.** After touching Rust code run `just ci-build` (or `make debug`) first, or the
  tests run against the previous artifact.
- **On Windows `make` has to run inside Git Bash**, not PowerShell.

### The fast loop

DuckDB's test runner can drive the artifact directly, which skips `make` entirely. The recipe needs a
Python environment with `duckdb_sqllogictest` installed — `make configure` creates one under
`configure/venv`, or you can use any Python 3 environment that has it:

```bash
# Linux / macOS
./configure/venv/bin/python -m duckdb_sqllogictest \
    --test-dir test/sql \
    --external-extension build/debug/my_extension.duckdb_extension
```

```powershell
# Windows
.\configure\venv\Scripts\python.exe -m duckdb_sqllogictest `
    --test-dir test/sql `
    --external-extension build/debug/my_extension.duckdb_extension
```

`--test-dir` is required: it is also the value of `__TEST_DIR__`, the directory a test writing files is
given. To run a single file, add `--file-path test/sql/scalar_greet.test`.

## Conventions

- **Every file starts from a clean database**, so `my_extension.test` can assert that the function does
  not exist before the extension is loaded, and the other files start with `require my_extension`.
- **Expected errors are matched as substrings.** Under `statement error`, a distinctive fragment of the
  message is enough — there is no need to reproduce DuckDB's whole error string, and doing so ties the
  test to a message that may well change.
- **Watch how values print.** A `DOUBLE` renders as `7.0`; when a test is about a number rather than a
  type, cast it (`my_sum(x)::DECIMAL(10,1)`) so the expectation stays stable if the type changes.
- **Divide the files by concern**, not by function count: behaviour in one file, error paths in
  another, and a `.test` that reaches for a community extension (for HTML parsing, say) kept separate,
  because it needs the network the first time.

## What a new function should cover

At least: ordinary values, `NULL`, a boundary value, and the error path. Three more that pay off:

- the node before the `LOAD` (`statement error` … `does not exist`), if the file is the smoke test;
- a NULL input in a chunk that is *not* constant-folded, since a constant `NULL` never reaches the body;
- more rows than `STANDARD_VECTOR_SIZE` (2048), which is what exercises `combine` for an aggregate and
  the per-chunk path for a scalar.

Before committing: `just lint` (`cargo clippy --all-targets -- -D warnings`).
