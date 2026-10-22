---
title: Installation
sidebar_position: 2
description: The three ways to install duckfn_open — DuckDB's community repository, a GitHub release, or a local build — and how to check that it loaded.
---

# Installation

`duckfn_open` needs **DuckDB 1.4 or newer**. The release binaries are built against DuckDB 2.0.0 and
also load into 1.4.x and 1.5.x (the C API is binary compatible across these versions), so there is
no separate 1.5 build: on an older DuckDB, install the same 2.0 release asset. It is an ordinary loadable extension, so `INSTALL` and
`LOAD` are all it takes; there is no separate download, library or setup step. It runs on a machine with a
desktop — Windows, Linux or macOS — because that is where a launcher lives. That is also its scope: it is for
interactive work, not for DuckDB embedded in an application.

## From the community repository

The simplest route, and the only one that needs no flags:

```sql
INSTALL duckfn_open FROM community;
LOAD duckfn_open;
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

Every version is published as a GitHub Release carrying one `.duckdb_extension` per desktop platform, named
`duckfn_open-<platform>.duckdb_extension` (`windows_amd64`, `linux_amd64`, `osx_arm64`, and so on). Download
the one for your platform and **rename it to `duckfn_open.duckdb_extension`**: DuckDB takes the entry-point
symbol from the file name, so loading the release name fails with
`did not contain function "duckfn_open-windows_amd64_init_c_api"` — and `LOAD` fetches no URLs anyway, it takes
an extension name or a local path:

```shell
curl -LO https://github.com/shijianjs/duckfn-open/releases/latest/download/duckfn_open-windows_amd64.duckdb_extension
mv duckfn_open-windows_amd64.duckdb_extension duckfn_open.duckdb_extension
duckdb -unsigned -c "LOAD './duckfn_open.duckdb_extension';"
```

A release file is not signed by DuckDB's distribution key, hence `-unsigned`. Only the desktop platforms are
published, and only those can load this extension usefully: Windows, Linux and macOS each have a launcher to
hand a file to, and a WebAssembly build would have none, so there is no such asset.

## From a local build

If you built the extension from source yourself, point `LOAD` at the artifact — and, again, start DuckDB
with `-unsigned`:

```sql
LOAD './build/debug/duckfn_open.duckdb_extension';
```

Building it is the subject of the [Development guide](../development/quick-start.md).

## Checking that it loaded

`duckdb_extensions()` lists what is available, and `duckdb_functions()` what is callable:

```sql
SELECT extension_name, installed, loaded
FROM duckdb_extensions()
WHERE extension_name = 'duckfn_open';
```

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'op%';
```

Twenty-one rows for the nine names this extension registers (`op_write_file` and `op_open_content` carry four
signatures each — text and binary, with and without options; `op_open_path`, `op_open_path_detached`,
`op_print` and `op_sanitize_filename_check` carry two each; `op_sanitize_filename` carries three; `op_open_commands` and
`op_open_html` one). The options STRUCT the write signatures take is a registered type rather than a
function, so it shows up in `duckdb_types()`, not here. If the list is empty, the extension is not loaded
in the session you are querying.
