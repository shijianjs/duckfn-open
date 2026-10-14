---
title: 函数
sidebar_position: 3
slug: /user-guide/functions
description: duckfn_open 注册的每一个 SQL 函数，以及可以在你自己的 DuckDB 里跑的例子。
---

# 函数

加载扩展会注册九个函数名、二十一个签名，而且它们全都是标量函数：没有聚合函数、没有表函数。它们和
DuckDB 自带的一样用，可以放进投影或 `WHERE` 条件，也能和内置函数随意组合。

九个名字里有七个在一个名字下并了多个签名。DuckDB 看到的是一个带若干重载的函数，所以多出来的那些形式
没有自己的 SQL 名字：调用是 `SELECT op_open_path('a', 'b')`，不是 `op_open_path_with('a', 'b')`。

唯一注册的类型是 [`op_write_file_options`](./write-file.md#op_write_file_options)，`op_write_file` 与
`op_open_content` 那几个可选项就装在这个 STRUCT 里。它和所有注册类型一样出现在 `duckdb_types()` 里，不在
`duckdb_functions()` 里；文件名那两个函数不注册任何类型。

## 路径与 URL

把已经存在的文件或链接交给桌面。详见[打开路径与 URL](./open-path.md)。

| 函数 | 签名 | 作用 |
| --- | --- | --- |
| [`op_open_path`](./open-path.md#op_open_path) | `VARCHAR -> BOOLEAN` | 用默认程序打开它，并等启动器返回。 |
| [`op_open_path`](./open-path.md#op_open_path) | `VARCHAR, VARCHAR -> BOOLEAN` | 同上，程序由第二个参数指定。 |
| [`op_open_path_detached`](./open-path.md#op_open_path_detached) | `VARCHAR -> BOOLEAN` | 打开它，启动器一起来就返回。 |
| [`op_open_path_detached`](./open-path.md#op_open_path_detached) | `VARCHAR, VARCHAR -> BOOLEAN` | 同上，程序由第二个参数指定。 |
| [`op_open_commands`](./open-path.md#op_open_commands) | `VARCHAR -> VARCHAR[]` | 列出 `op_open_path` 会依次尝试的启动命令，一条都不执行。 |

## 内容

一条查询产出但还没有文件的东西：一份报表、一张图、一个页面。详见[把内容落成文件](./write-file.md)与
[打印到 stdout](./print.md)。

| 函数 | 签名 | 作用 |
| --- | --- | --- |
| [`op_write_file`](./write-file.md#op_write_file) | `VARCHAR, VARCHAR -> VARCHAR` | 把文本写成一个文件并返回它的路径，什么都不启动。 |
| [`op_write_file`](./write-file.md#op_write_file) | `VARCHAR, VARCHAR, op_write_file_options -> VARCHAR` | 同上，目录、文件名和名字里的那一段都由你选。 |
| [`op_write_file`](./write-file.md#op_write_file) | `BLOB, VARCHAR -> VARCHAR` | 二进制内容（比如一张图），按它本来的那些字节写出去。 |
| [`op_write_file`](./write-file.md#op_write_file) | `BLOB, VARCHAR, op_write_file_options -> VARCHAR` | 同上，带上那几个选项。 |
| [`op_open_content`](./write-file.md#op_open_content) | `op_write_file` 的四个签名，返回 `VARCHAR` | 在同一次调用里写出去再打开。 |
| [`op_open_html`](./write-file.md#op_open_html) | `VARCHAR -> VARCHAR` | 零配置的捷径：一个临时 `.html`，在默认浏览器里打开。 |
| [`op_print`](./print.md#op_print) | `VARCHAR -> VARCHAR` | 越过结果集的排版直接写到 stdout，并把那段文本原样返回。 |
| [`op_print`](./print.md#op_print) | `VARCHAR, VARCHAR -> VARCHAR` | 同上，第二个参数指定文本后面接什么，取代那个换行；这个 tail 不进返回值。 |

## 文件名

纯字符串的事：关于一个值，它回答的是这个值能不能当文件名。详见[从数据造文件名](./sanitize-filename.md)。

| 函数 | 签名 | 作用 |
| --- | --- | --- |
| [`op_sanitize_filename`](./sanitize-filename.md#op_sanitize_filename) | `VARCHAR -> VARCHAR` | 把一个字符串改写成一个可用的文件名。 |
| [`op_sanitize_filename`](./sanitize-filename.md#op_sanitize_filename) | `VARCHAR, VARCHAR -> VARCHAR` | 同上，第二个参数指定每个非法字符换成什么，而不是直接删掉。 |
| [`op_sanitize_filename`](./sanitize-filename.md#op_sanitize_filename) | `VARCHAR, VARCHAR, BOOLEAN, BOOLEAN -> VARCHAR` | 同上，再加上那两个开关：255 字节这一刀，以及 Windows 那套规则。 |
| [`op_sanitize_filename_check`](./sanitize-filename.md#op_sanitize_filename_check) | `VARCHAR -> BOOLEAN` | 什么都不改写：这个字符串本来就是一个可用的名字吗？ |
| [`op_sanitize_filename_check`](./sanitize-filename.md#op_sanitize_filename_check) | `VARCHAR, BOOLEAN, BOOLEAN -> BOOLEAN` | 同一个问题，带上那两个开关。 |

整套 API 就是这些，一条查询就能按 DuckDB 的视角把它列出来：

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'op%'
ORDER BY 1, 2;    -- 九个名字，二十一个签名
```

## 几点说明

- **调用背后得有一台桌面。** 启动器是桌面软件，所以在没有桌面的机器上 —— 服务器、CI runner —— 四个
  `op_open_path` 动作一律答复 `false`，`op_open_content` 和 `op_open_html` 报一句带着它们确实写出的那个
  路径的错误，而 `op_open_commands` 是那个照样能答的调用，它列出这台机器上会依次尝试什么。
  `op_write_file`、`op_print` 和文件名那两个函数根本不需要桌面。这是这个扩展本来的形状，不是管道层的偶然 ——
  它是给交互式探索用的，DuckDB 被别人嵌进应用里跑的那种情形不在它服务的范围内（见[简介](../intro.md)）。
- **NULL 进、NULL 出 —— 而且什么都没启动、什么都没落盘。** 三种结果因此读得出来：NULL 就是这一行没有
  输入。对 `op_sanitize_filename` 与 `op_sanitize_filename_check` 来说，它指的是**任何**一个参数是 NULL，那两个
  开关也算。
- **被拒绝的输入会让整条语句失败。** 报错信息里带函数名，查询剩下的部分不会再执行。而一个「只是不能用」
  的值在这里从来不算错 —— 它会得到一个答案。
- **一次调用就是一行，永远如此。** 这里凡是会碰外面世界的函数都注册成 volatile，所以 DuckDB 既不会把常量
  参数的调用折叠成一次执行，也不会缓存它：`SELECT op_open_path('report.html')` 开一个窗口，加上 `FROM reports`
  就变成一行一个 —— 常量参数和列参数的表现完全一样。按行跑 `op_write_file` 就是一行一个文件，这正是那套生成
  名字要应付的。规划一条查询时也什么都不会发生，所以 `EXPLAIN SELECT op_write_file(…)` 一个文件都不写。
- **列表带着类型走。** `op_open_commands` 返回 `VARCHAR[]`，所以里面的每一项都保留它被启动时那副引用
  样子；要一条命令一行就取下标（`op_open_commands('report.html')[1]`）或者 `unnest`。
- **文件名那两个函数在每台机器上答得一样。** 它们里面没有任何东西取决于文件系统、桌面，或者某个窗口
  干了什么，所以改写出来的名字可以在测试里逐字断言 —— 这也正是 `windows` 被钉成 `true`、而不是跟着宿主
  平台走的原因。
