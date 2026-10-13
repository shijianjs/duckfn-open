---
title: 简介
sidebar_position: 1
slug: /intro
description: duckfn_open 给 DuckDB 加了什么，以及怎么开始用。
---

# 简介

`duckfn_open` 是一个 [DuckDB 可加载扩展](https://duckdb.org/docs/stable/extensions/overview)：它给 DuckDB
加了一批 SQL 函数，加载之后它们和 DuckDB 自带的函数没有区别。你不需要知道它是怎么写出来的，也不用编译
任何东西 —— CLI、Python / R 会话与其他 DuckDB 前端接受的都同一份 `.duckdb_extension` 文件。

它加的东西一句话就能说完：把一个路径或 URL 交给桌面当前配置好的程序去打开，让一条查询能把报表送到人
眼前。

它加了三个函数名、五个签名，全都是标量函数 —— 没有聚合函数、没有表函数、也没有自定义类型：

| 函数 | 类别 | 作用 |
| --- | --- | --- |
| [`open_path(path)`](./functions.md#open_path) | 标量 | 用默认程序打开这个路径或 URL，并等启动器返回。 |
| [`open_path(path, app)`](./functions.md#open_path) | 标量 | 同上，程序由第二个参数指定。 |
| [`open_path_detached(path)`](./functions.md#open_path_detached) | 标量 | 打开它，启动器一起来就返回。 |
| [`open_path_detached(path, app)`](./functions.md#open_path_detached) | 标量 | 同上，程序由第二个参数指定。 |
| [`open_commands(path)`](./functions.md#open_commands) | 标量 | 列出 `open_path` 会依次尝试的启动命令，一条都不执行。 |

## 装上它

```sql
INSTALL duckfn_open FROM community;
LOAD duckfn_open;
```

然后在任意查询里调用：

```sql
SELECT open_path('/var/tmp/report.html');        -- 桌面给 .html 配的那个程序
SELECT open_path_detached('https://duckdb.org'); -- 不用等页面关掉就返回
```

这些页面不会替你跑任何东西：这里是文档，每个例子都是给已经加载了扩展的 DuckDB 用的普通 SQL。下面这条
查询只从目录里读出注册到位的那套 API，不去 launch 任何进程，所以它最适合先试 —— 它根本不需要桌面。那
四个启动签名需要，在没有桌面的机器上（服务器、CI runner）它们一律答复 `false`；每个答复意味着什么，都
写在[函数](./functions.md)那一页里。

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'open%'
ORDER BY 1, 2;    -- 三个名字，五个签名
```

其他获取文件的方式见[安装](./installation.md)。

:::note[关于副作用]

五个签名里有四个会起进程，所以把它们放进循环之前值得先问两句：这会开出多少个窗口，以及这条查询跑在
谁的机器上。参数全是常量的调用会被 DuckDB 折叠成一次执行，所以 `SELECT open_path('report.html')` 只开
一个窗口，而 `SELECT open_path(p) FROM reports` 一行开一个 —— 不去重。`open_commands` 是唯一完全没有
副作用的那个名字，「这里到底会跑什么」问它就行。

:::

## 接下来去哪

- [安装](./installation.md) —— 社区仓、Release 文件，或者本地自己构建的产物。
- [函数](./functions.md) —— 每个签名、每个返回值是什么意思，以及什么都没打开时该查什么。
- [开发指南](../development/quick-start.md) —— 从源码构建这个扩展。那是给改这个仓库的人看的；用扩展
  本身不需要它。
