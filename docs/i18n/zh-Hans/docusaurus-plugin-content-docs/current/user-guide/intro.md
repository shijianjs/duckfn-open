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
眼前。可查询产出的东西大多还没有路径 —— 它是一段内容 —— 所以这套 API 的另一半先把内容写成一个文件，
再打开那个文件。还有第三块这两边都不属于：它就在一个普通字符串上了结一件事 —— 这个字符串到底能不能当
一个文件名。

加载之后注册到位的是九个函数名、二十一个签名，而且全都是标量函数 —— 没有聚合函数，也没有表函数。它另外
注册一个 STRUCT 类型 `op_write_file_options(dir VARCHAR, "name" VARCHAR, part VARCHAR)`，落盘那三个可选项就
放在这里；它和所有注册类型一样，出现在 `duckdb_types()` 里，而不是 `duckdb_functions()` 里。新加的这两个
名字不注册任何类型。

这些名字分成三组。第一组接手一个已经存在的路径或 URL，把它交给桌面：

| 函数 | 类别 | 作用 |
| --- | --- | --- |
| [`op_open_path(path)`](./functions.md#op_open_path) | 标量 | 用默认程序打开这个路径或 URL，并等启动器返回。 |
| [`op_open_path(path, app)`](./functions.md#op_open_path) | 标量 | 同上，程序由第二个参数指定。 |
| [`op_open_path_detached(path)`](./functions.md#op_open_path_detached) | 标量 | 打开它，启动器一起来就返回。 |
| [`op_open_path_detached(path, app)`](./functions.md#op_open_path_detached) | 标量 | 同上，程序由第二个参数指定。 |
| [`op_open_commands(path)`](./functions.md#op_open_commands) | 标量 | 列出 `op_open_path` 会依次尝试的启动命令，一条都不执行。 |

第二组从**内容**出发：一段 VARCHAR 或 BLOB 先变成一个文件，之后它就是一条普通的路径，第一组能照常接手。

| 函数 | 类别 | 作用 |
| --- | --- | --- |
| [`op_write_file(content, ext)`](./functions.md#op_write_file) | 标量 | 把内容写出去并返回路径，不起任何进程。 |
| [`op_write_file(content, ext, opts)`](./functions.md#op_write_file) | 标量 | 同上，目录、文件名与名字里的那一段由 `op_write_file_options` 指定。 |
| [`op_open_content(…)`](./functions.md#op_open_content) | 标量 | `op_write_file` 的四个签名再补一次启动：落盘、用默认程序打开、返回路径。 |
| [`op_open_html(content)`](./functions.md#op_open_html) | 标量 | HTML 的零配置快捷方式：所有选项都按缺省的 `op_open_content(content, 'html')`。 |
| [`op_print(text)`](./functions.md#op_print) | 标量 | 把文本直接打到 stdout，绕开结果集的排版；恒返回 NULL。 |
| [`op_print(text, tail)`](./functions.md#op_print) | 标量 | 同上，指定打完之后追加什么来代替那个换行。 |

第三组做的是纯字符串的事：它不起程序、不碰文件系统，回答的是另外两组一旦有名字来自数据本身就会
撞上的那个问题 —— 这个值能当一个文件名吗？

| 函数 | 类别 | 作用 |
| --- | --- | --- |
| [`op_sanitize_filename(value)`](./functions.md#op_sanitize_filename) | 标量 | 把字符串改写成一个可用的文件名，不能出现在名字里的东西被删掉。 |
| [`op_sanitize_filename(value, replacement)`](./functions.md#op_sanitize_filename) | 标量 | 同上，非法字符所在的位置换成你给的那一段，改动因此看得见。 |
| [`op_sanitize_filename(value, replacement, truncate, windows)`](./functions.md#op_sanitize_filename) | 标量 | 同上，把 255 字节这一刀与 Windows 那套规则做成两个开关。 |
| [`op_sanitize_filename_check(value)`](./functions.md#op_sanitize_filename_check) | 标量 | 什么都不改写：这个值本来就是一个可用的名字时为 `true`。 |
| [`op_sanitize_filename_check(value, truncate, windows)`](./functions.md#op_sanitize_filename_check) | 标量 | 同一个问题，带上那两个开关。 |

## 装上它

```sql
INSTALL duckfn_open FROM community;
LOAD duckfn_open;
```

然后在任意查询里调用：

```sql
SELECT op_open_path('/var/tmp/report.html');        -- 桌面给 .html 配的那个程序
SELECT op_open_path_detached('https://duckdb.org'); -- 不用等页面关掉就返回
SELECT op_write_file('{"a": 1}', 'json');           -- 只落一个文件、回一条路径，不起进程
SELECT op_open_html('<h1>hello</h1>');              -- 落一个临时 .html，并用默认浏览器打开
SELECT op_sanitize_filename('quarterly/report:v2.csv'); -- 'quarterlyreportv2.csv' —— 只是一个字符串
```

这些页面不会替你跑任何东西：这里是文档，每个例子都是给已经加载了扩展的 DuckDB 用的普通 SQL。下面这条
查询只读目录、不做任何动作，所以它最适合先试：它不需要桌面，`op_write_file`、`op_print` 和文件名那一族也
不需要 —— 一个只写文件，一个只打 stdout，另外两个改写一个字符串就停在那里。启动那件事才需要桌面：在
没有桌面的机器上（服务器、CI runner）四个 `op_open_path` 动作一律答复 `false`，而 `op_open_content` 和
`op_open_html` 会报错，报文里带着它们确实已经写出去的那个路径。每个答复意味着什么，都写在
[函数](./functions.md)那一页里。

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'op%'
ORDER BY 1, 2;    -- 九个名字，二十一个签名
```

其他获取文件的方式见[安装](./installation.md)。

:::note[关于副作用]

二十一个签名里有九个会起进程，所以把它们放进循环之前值得先问两句：这会开出多少个窗口，以及这条查询跑在
谁的机器上。参数全是常量的调用会被 DuckDB 折叠成一次执行，所以 `SELECT op_open_path('report.html')` 只开
一个窗口，而 `SELECT op_open_path(p) FROM reports` 一行开一个 —— 不去重。其余的都不起进程：`op_open_commands`
只列出启动会依次尝试的那些命令，`op_write_file` 落完盘就停 —— 这正是让你可以自己把路径交给 `op_open_path` 的
原因 —— `op_print` 只往 stdout 打，而 `op_sanitize_filename` 与 `op_sanitize_filename_check` 一步都离不开递给它们的
那个字符串。

:::

## 接下来去哪

- [安装](./installation.md) —— 社区仓、Release 文件，或者本地自己构建的产物。
- [函数](./functions.md) —— 每个签名、那个选项 STRUCT、每个返回值是什么意思，以及什么都没打开时该查
  什么。
- [开发指南](../development/quick-start.md) —— 从源码构建这个扩展。那是给改这个仓库的人看的；用扩展
  本身不需要它。
