---
title: 快速开始
sidebar_position: 1
description: 改扩展名、用 cargo 构建出 .duckdb_extension、加载进 DuckDB，然后调用示例函数。
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

`scripts/rename.sh` 会把扩展名必须一致的五处一次改齐 —— `Cargo.toml`（`[package] name` 与
`[[example]] name`）、Makefile 的 `EXTENSION_NAME`、`src/extension/mod.rs` 里的入口点符号、Justfile、
CI 工作流 —— 以及文档里出现的每一处，并按新包名重写 `Cargo.lock`。它最后会打印还需要人工过一遍的清单，
示例函数是其中主要的一项。

### 改完之后

扩展名处处一致之后，还剩这些要人工过一遍：

1. **`AGENTS.md`** —— 填上 `{{PROJECT_GOAL}}` 占位符，并把命名约定里的 `my_` 前缀改成你自己的。
2. **示例函数**（`my_greet` / `my_greet_checked` / `my_sum`）与 `test/sql/*.test` —— 换成你自己的 API，
   见[写函数](./writing-functions.md)与[测试](./testing.md)。
3. **`community-extension/description.yml`** —— `extension.name` / `description` / `maintainers` / `repo`
   由你填，逐个字段的说明在[社区扩展](./community-extension.md)。
4. **`LICENSE`** —— 版权人，以及 `Cargo.toml` 里的 duckfn 版本。
5. **首次发版前**，确认仓库有 `main` 分支和 `origin` 远程：`just release_tag` 会同时推送这两者。

## 2. 构建

```shell
just build          # = make configure && make debug
```

产物是 `build/debug/my_extension.duckdb_extension`。没有 C++ 这一步，也不需要本地编译 DuckDB：扩展
只用到 DuckDB 的头文件，加载时通过它的 API 表分发。

## 3. 加载并调用

```shell
just repl           # 已经 LOAD 好扩展的 DuckDB REPL
```

```sql
-- 或者手动来；本地构建的产物必须加 -unsigned
duckdb -unsigned -c "LOAD './build/debug/my_extension.duckdb_extension';"
```

下面是几个示例函数，就地就能跑 —— 站点从仓库的最新 Release 预加载了这个扩展，这里不用写 `LOAD`
（本地自己构建的产物仍然要加 `-unsigned`，见下面的几个坑）。点任意块上的 **执行** 即可。

```sql {"type":"duckfn","show":"table"}
SELECT name AS input, my_greet_checked(name) AS greeting
FROM (VALUES ('world'), ('')) t(name);
```

```sql {"type":"duckfn","show":"table"}
-- my_sum 跳过 NULL；一组里一个有效值都没有时结果是 NULL 而不是 0。
SELECT grp, my_sum(x) AS total
FROM (VALUES ('rows', 1.5::DOUBLE), ('rows', 2.5), ('all NULL', NULL::DOUBLE)) t(grp, x)
GROUP BY grp
ORDER BY grp;
```

失败路径同样是个可运行块 —— 它自己声明了「应该失败」：

```sql {"type":"duckfn","expect":"error"}
SELECT my_greet_checked(' x ');    -- 报错：首尾不允许有空格
```

不进 REPL、只跑一条语句：

```shell
just sql "SELECT my_greet('world')"
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
  `win.duckdb_extension` 会报 `did not contain function "my_extension_init_c_api"`。
- **`make test` 不会自动重新构建。** 改完 Rust 先跑 `just ci-build`（或 `make debug`），否则测试跑的
  还是上一次的产物。

:::

Windows 上还有一条：如果 `make debug` 报产物被占用，说明有 DuckDB 进程正拿着
`build/debug/my_extension.duckdb_extension`（多半是没关的 `just repl`）—— 关掉那个进程重新构建即可。
`.duckdb_extension` 不是改了名的 DLL：DuckDB 的元数据在文件尾，直接 `Copy-Item` 一个 DLL 过去会报
`The metadata at the end of the file is invalid`。
