---
title: 安装
sidebar_position: 2
description: 装 my_extension 的三条路 —— DuckDB 社区仓、GitHub Release，或本地自己构建的产物 —— 以及怎么确认它加载上了。
---

# 安装

`my_extension` 需要 **DuckDB 1.3 或更新**。它是一个普通的可加载扩展，`INSTALL` 与 `LOAD` 就是全部；
没有额外的下载、库文件或配置步骤。

## 从社区仓安装

最省事的一条，也是唯一不需要任何命令行开关的：

```sql
INSTALL my_extension FROM community;
LOAD my_extension;
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
LOAD 'https://github.com/<owner>/<repo>/releases/latest/download/my_extension-windows_amd64.duckdb_extension';
```

Release 里的文件没有用 DuckDB 的发行密钥签名，所以 DuckDB 必须加 `-unsigned` 启动：

```shell
duckdb -unsigned
```

在 Release 页面上挑对应平台的资产 —— 命名规则是 `my_extension-<平台>.duckdb_extension`
（`windows_amd64`、`linux_amd64`、`osx_arm64` 等），另有一份给浏览器用的 WebAssembly 构建。

## 从本地构建的产物安装

如果你自己从源码构建了扩展，把 `LOAD` 指到产物上 —— 同样要用 `-unsigned` 启动 DuckDB：

```sql
LOAD './build/debug/my_extension.duckdb_extension';
```

怎么构建见[开发指南](../development/quick-start.md)。

## 确认它加载上了

`duckdb_extensions()` 列出可用的扩展，`duckdb_functions()` 列出可调用的函数：

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

三个函数就是三行；如果一行都没有，说明你查的这个会话里没有加载这个扩展。
