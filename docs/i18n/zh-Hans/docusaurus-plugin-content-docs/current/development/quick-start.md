---
title: 快速开始
sidebar_position: 1
description: 改扩展名、用官方 make 工具链构建出 .duckdb_extension、加载进 DuckDB，然后在 SQL 里调用这些函数。
---

# 快速开始

整页就是四步：

```mermaid
flowchart LR
    rename["just rename"] --> build["just build"]
    build --> load["用 -unsigned<br/>LOAD 产物"]
    load --> call["在 SQL 里<br/>调用函数"]
```

## 前置条件

- **Rust** 1.86 或更新（`Cargo.toml` 里的 `rust-version`）。
- **[just](https://github.com/casey/just)** —— recipe 会调用它：

  ```shell
  cargo install just
  ```

- **make**（Windows 上要在 Git Bash 里跑）与 **Python 3** —— 官方 DuckDB
  `extension-ci-tools` 的构建 / 测试流程，`just build` 底层就是它（`make configure` + `make debug`），
  `just test` 走的也是同一套。
- 一个 **DuckDB** 1.3 或更新的可执行文件（`duckdb` 在 `PATH` 里，或者用
  `just DUCKDB=/path/to/duckdb …` 指定）。

## 1. 改扩展名

```shell
just rename csv_stats
```

`scripts/rename.sh` 会把扩展名必须一致的五处一次改齐 —— `Cargo.toml`（`[package] name`）、Makefile 的
`EXTENSION_NAME`、`src/extension/mod.rs` 里的入口点符号、Justfile、CI 工作流 —— 以及文档里出现的每一处，
并按新包名重写 `Cargo.lock`。它最后会打印还需要人工过一遍的清单，函数与它们的测试是其中主要的一项。

### 改完之后

扩展名处处一致之后，还剩这些要人工过一遍：

1. **`AGENTS.md`** —— 填上 `{{PROJECT_GOAL}}` 占位符，并把命名约定里的函数名前缀改成你自己的（这个
   扩展的前缀是 `open_`）。
2. **模板自带的示例函数与 `test/sql/*.test`** —— 换成你自己的 API，见[编写函数](./writing-functions.md)
   与[测试](./testing.md)。本仓库已经走完这一步：`src/extension/functions/` 里的
   `open_path` / `open_path_detached` / `open_commands`、`src/extension/launcher.rs` 这层平台代码，以及
   `test/sql/` 下按函数文件各一份的用例，就是它做完之后的样子。
3. **`community-extension/description.yml`** —— `extension.name` / `description` / `maintainers` / `repo`
   由你填，逐个字段的说明在[社区扩展](./community-extension.md)。
4. **`LICENSE`** —— 版权人，以及 `Cargo.toml` 里的 duckfn 版本。
5. **首次发版前**，确认仓库有 `main` 分支和 `origin` 远程：`just release_tag` 会同时推送这两者。

## 2. 构建

```shell
just build          # = make configure && make debug
```

产物是 `build/debug/duckfn_open.duckdb_extension`。没有 C++ 这一步，也不需要本地编译 DuckDB：扩展
只用到 DuckDB 的头文件，加载时通过它的 API 表分发。

## 3. 加载并调用

```shell
just repl           # 已经 LOAD 好扩展的 DuckDB REPL
```

```sql
-- 或者手动来；本地构建的产物必须加 -unsigned
duckdb -unsigned -c "LOAD './build/debug/duckfn_open.duckdb_extension';"
```

下面把扩展注册了什么直接从目录里读出来。本页每个 `sql` 围栏都是静态代码块 —— 站点不在浏览器里
跑任何东西 —— 所以把它们贴进上面那个 REPL 就能跑（或者先在你自己的 `duckdb -unsigned` 里 `LOAD` 产物）。

```sql
SELECT function_name, len(parameters) AS args, return_type
FROM duckdb_functions()
WHERE function_name IN ('open_path', 'open_path_detached', 'open_commands')
ORDER BY function_name, args;
```

三个 SQL 名字、五个签名：两个启动器名字各有 `(path)` 与 `(path, app)` 两个形式，靠 `overloads_name`
并成一个函数集。

不起进程的那条调用在哪台机器上答案都一样，所以可移植的演示用的是 NULL：

```sql
SELECT open_path(NULL) AS opened;   -- NULL 输入被短路，一次启动器都不会被叫起来
```

失败路径也值得一跑：空参数是报错，不是 `false`：

```sql
SELECT open_path('');    -- 报错：路径或 URL 不能为空
```

真去 launch 会答什么都取决于这台机器的桌面：装了哪些启动器、`.html` 关联到了谁。也没有第二个可以跑它的
地方 —— 这个扩展只构建桌面平台，见[构建与发版](./build-and-release.md)。

```sql
SELECT open_path('report.html');                    -- 默认程序，等启动器返回
SELECT open_path('https://duckdb.org', 'firefox');  -- 指定程序
SELECT open_path_detached('report.html');           -- 起进程即返回
SELECT open_commands('report.html');                -- 会依次尝试的命令，一条都不执行
```

不进 REPL、只跑一条语句：

```shell
just sql "SELECT open_path(NULL)"
```

## 4. 跑测试

```shell
just test           # make configure + make debug + make test
```

更快的迭代方式见[测试](./testing.md)。

## 几个坑

:::warning[三个看着像 bug、其实不是的]

- **本地构建的产物加载时必须加 `-unsigned`**，不加 DuckDB 会直接拒绝这个文件。
- **产物文件名必须保持 `<扩展名>.duckdb_extension`。** DuckDB 是按文件名去找入口点符号的，复制成
  `win.duckdb_extension` 会报 `did not contain function "duckfn_open_init_c_api"`。
- **`make test` 不会自动重新构建。** 改完 Rust 先跑 `just ci-build`（或 `make debug`），否则测试跑的
  还是上一次的产物。

:::

Windows 上还有一条：如果 `make debug` 报产物被占用，说明有 DuckDB 进程正拿着
`build/debug/duckfn_open.duckdb_extension`（多半是没关的 `just repl`）—— 关掉那个进程重新构建即可。
`.duckdb_extension` 不是改了名的 DLL：DuckDB 的元数据在文件尾，直接 `Copy-Item` 一个 DLL 过去会报
`The metadata at the end of the file is invalid`。
