---
title: 简介
sidebar_position: 1
slug: /intro
description: my_extension 给 DuckDB 加了什么，以及怎么开始用。
---

# 简介

`my_extension` 是一个 [DuckDB 可加载扩展](https://duckdb.org/docs/stable/extensions/overview)：它给 DuckDB
加了一批 SQL 函数，加载之后它们和 DuckDB 自带的函数没有区别。你不需要知道它是怎么写出来的，也不用编译
任何东西 —— CLI、Python / R 会话与浏览器里的构建接受的都同一份 `.duckdb_extension` 文件。

它加了三个函数：

| 函数 | 类别 | 作用 |
| --- | --- | --- |
| [`my_greet(name)`](./functions.md#my_greet) | 标量 | 向 `name` 问好；永不为 NULL。 |
| [`my_greet_checked(name)`](./functions.md#my_greet_checked) | 标量 | 同上，但空串返回 NULL、首尾有空格则报错。 |
| [`my_sum(value)`](./functions.md#my_sum) | 聚合 | 对一列 `DOUBLE` 求和，跳过 NULL。 |

## 装上它

```sql
INSTALL my_extension FROM community;
LOAD my_extension;
```

然后在任意查询里调用：

```sql {"type":"duckfn","show":"table"}
SELECT name AS input, my_greet_checked(name) AS greeting
FROM (VALUES ('world'), ('')) t(name);
```

上面这个块就在你的浏览器里真跑：站点从项目的最新 Release 预加载了这个扩展，所以这里不用写 `LOAD`。
其他获取文件的方式见[安装](./installation.md)。

:::note[这些页面是模板的起点]

`my_extension` 是 [duckfn-extension-template](https://github.com/shijianjs/duckfn-extension-template) 的
示例扩展。站点这一部分描述的就是它的三个示例函数；从模板生成的项目会换成自己的 API，并把这些页面一并
改掉。

:::

## 接下来去哪

- [安装](./installation.md) —— 社区仓、Release 文件，或者本地自己构建的产物。
- [函数](./functions.md) —— 每个函数，配一个能就地跑的例子。
- [开发指南](../development/quick-start.md) —— 从源码构建这个扩展。那是给改这个仓库的人看的；用扩展
  本身不需要它。
