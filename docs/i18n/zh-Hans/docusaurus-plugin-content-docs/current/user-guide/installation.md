---
title: 安装
sidebar_position: 2
description: 装 duckfn_open 的三条路 —— DuckDB 社区仓、GitHub Release，或本地自己构建的产物 —— 以及怎么确认它加载上了。
---

# 安装

`duckfn_open` 需要 **DuckDB 1.3 或更新**。它是一个普通的可加载扩展，`INSTALL` 与 `LOAD` 就是全部；
没有额外的下载、库文件或配置步骤。它跑在一台有桌面的机器上 —— Windows、Linux 或 macOS —— 因为启动器
就在那里。

## 从社区仓安装

最省事的一条，也是唯一不需要任何命令行开关的：

```sql
INSTALL duckfn_open FROM community;
LOAD duckfn_open;
```

DuckDB 会下载一份签过名、且与你本机 DuckDB 版本和平台匹配的构建。`INSTALL` 只需做一次，但每个新会话
都要重新 `LOAD`。

:::note[关于这条路]

`INSTALL … FROM community` 要等扩展注册进 DuckDB 社区仓之后才可用。注册是维护者做的一步，见
[社区扩展](../development/community-extension.md)；在那之前，用 Release 文件或本地构建的产物。

:::

## 从 GitHub Release 安装

每个版本都会发成一个 GitHub Release，里面每个平台各一份 `.duckdb_extension`。直接按 URL 加载你平台的
那一份：

```sql
LOAD 'https://github.com/<owner>/<repo>/releases/latest/download/duckfn_open-windows_amd64.duckdb_extension';
```

Release 里的文件没有用 DuckDB 的发行密钥签名，所以 DuckDB 必须加 `-unsigned` 启动：

```shell
duckdb -unsigned
```

在 Release 页面上挑对应平台的资产 —— 命名规则是 `duckfn_open-<平台>.duckdb_extension`
（`windows_amd64`、`linux_amd64`、`osx_arm64` 之类）。发布的只有桌面平台，也只有这些平台装载这个扩展
有意义：Windows、Linux 与 macOS 各有一个可以把文件交出去的启动器，而 WebAssembly 构建一个都没有，所以
也不存在那样的资产。

## 从本地构建的产物安装

如果你自己从源码构建了扩展，把 `LOAD` 指到产物上 —— 同样要用 `-unsigned` 启动 DuckDB：

```sql
LOAD './build/debug/duckfn_open.duckdb_extension';
```

怎么构建见[开发指南](../development/quick-start.md)。

## 确认它加载上了

`duckdb_extensions()` 列出可用的扩展，`duckdb_functions()` 列出可调用的函数：

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

这个扩展注册九个名字，对应二十一行（`op_write` 与 `op_open_content` 各四个签名 —— 文本与二进制 × 带选项与
不带选项；`op_open_path`、`op_open_path_detached`、`op_print` 与 `op_filename_check` 各两个；`op_filename`
三个；`op_open_commands` 与 `op_open_html` 各一个）。落盘签名要传的那个选项 STRUCT 是注册的类型而不是函数，所以
它出现在 `duckdb_types()` 里，不在这里。如果一行都没有，说明你查的这个会话里没有加载这个扩展。
