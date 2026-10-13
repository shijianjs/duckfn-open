---
title: Installation
sidebar_position: 2
description: The three ways to install my_extension — DuckDB's community repository, a GitHub release, or a local build — and how to check that it loaded.
---

# Installation

`my_extension` needs **DuckDB 1.3 or newer**. It is an ordinary loadable extension, so `INSTALL` and
`LOAD` are all it takes; there is no separate download, library or setup step.

## From the community repository

The simplest route, and the only one that needs no flags:

```sql
INSTALL my_extension FROM community;
LOAD my_extension;
```

DuckDB downloads a build that is signed and matched to your DuckDB version and platform. `INSTALL` is a
one-time step, but `LOAD` is needed again in each new session.

:::note[About this route]

`INSTALL … FROM community` works once the extension has been registered in DuckDB's community repository.
That registration is a maintainer step described in
[Community extensions](../development/community-extension.md); until it is done, load a release file or a
local build instead.

:::

## From a GitHub release

Every version is published as a GitHub Release carrying one `.duckdb_extension` per platform. Load the
one matching your platform straight from its URL:

```sql
LOAD 'https://github.com/<owner>/<repo>/releases/latest/download/my_extension-windows_amd64.duckdb_extension';
```

A release file is not signed by DuckDB's distribution key, so DuckDB has to be started with `-unsigned`:

```shell
duckdb -unsigned
```

Pick the asset for your platform from the release page — names follow
`my_extension-<platform>.duckdb_extension` (`windows_amd64`, `linux_amd64`, `osx_arm64`, and so on), plus a
WebAssembly build for the browser.

## From a local build

If you built the extension from source yourself, point `LOAD` at the artifact — and, again, start DuckDB
with `-unsigned`:

```sql
LOAD './build/debug/my_extension.duckdb_extension';
```

Building it is the subject of the [Development guide](../development/quick-start.md).

## Checking that it loaded

`duckdb_extensions()` lists what is available, and `duckdb_functions()` what is callable:

```sql
SELECT extension_name, installed, loaded
FROM duckdb_extensions()
WHERE extension_name = 'my_extension';
```

```sql
SELECT function_name, function_type, return_type
FROM duckdb_functions()
WHERE function_name LIKE 'my%';
```

Three rows for three functions; if the list is empty, the extension is not loaded in the session you are
querying.
