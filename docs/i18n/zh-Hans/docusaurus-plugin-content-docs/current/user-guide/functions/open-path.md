---
title: 打开路径与 URL
sidebar_position: 1
description: op_open_path、op_open_path_detached 与 op_open_commands —— 把一个路径或 URL 交给桌面。
---

# 打开路径与 URL

这三个接住一个已经存在的东西 —— 文件、目录、URL —— 把它交给桌面。它们也是这个扩展里唯一会起进程的
调用。

一次启动答复三种结果之一：

| 结果 | 含义 |
| --- | --- |
| `NULL` | 这一行没有输入。NULL 永远到不了启动器，所以没让这台机器的桌面做任何事。 |
| `true` | 启动器被拉起了。 |
| `false` | 一个启动器都没能起来：这台机器没有配上任何一个，或者第二个参数指定的程序不存在。 |

```sql
SELECT op_open_path(NULL) AS opened;    -- NULL：这一行没有输入，所以什么都没启动
```

NULL 跟着行走，不是只跟着常量走：整列都是 NULL 时，一次启动也不会发生。

```sql
SELECT count(*) FROM reports WHERE op_open_path(path);    -- true 留下这一行，false 与 NULL 把它丢掉
```

失败只能答复 `false`，是因为启动器自己那份问题报告在 SQL 里没有地方放。「为什么什么都没打开」要问
[op_open_commands](#op_open_commands)。

## op_open_path

```text
op_open_path(path VARCHAR) -> BOOLEAN
op_open_path(path VARCHAR, app VARCHAR) -> BOOLEAN
```

把一个本地路径或 URL 交给桌面当前配置好的程序 —— 也就是双击这个文件时接住它的那个程序；面对一个 URL
时，那就是默认浏览器。第二个签名改为由你指定程序。

```sql
SELECT op_open_path('report.html');
SELECT op_open_path('https://duckdb.org', 'firefox');
SELECT op_open_path('/data/report.csv', 'LibreOffice Calc');
```

**这个调用会等。** 它在启动器返回时才返回，而 Linux 上常见的启动器通常要到打开这个文件的那个程序退出
时才返回 —— 这是一条查询唯一可能看起来像卡住的地方，也正是这个名字与
[`op_open_path_detached`](#op_open_path_detached) 的差别。Windows 上方向是反的：在那里
`op_open_path_detached` 反而是两者里更直接的一条路，因为它启动的那一步只是一次原生调用，而不是「一个
辅助进程，后面还挂着一个兜底的启动器」。

`app` 参数会原样交给平台：Linux 上是能在 `PATH` 里找到的可执行名，Windows 上先在已安装应用程序里查
这个名、再交给 shell。既不解析别名，也不校验它存在与否 —— 起不来的程序就是一个 `false`。

空的路径、空的程序名是一句错误，而不是一个 `false` ——「写错了」和「打不开」该有不同的答复：

```sql
SELECT op_open_path('');    -- 报错：op_open_path: the path or URL must not be empty
```

第二个参数有它自己那句文案（`… the application must not be empty`）。参数的个数与类型在绑定查询时就把
关：`op_open_path()`、`op_open_path('a', 'b', 'c')`、`op_open_path(42)` 全都在任何东西跑起来之前失败。

## op_open_path_detached

```text
op_open_path_detached(path VARCHAR) -> BOOLEAN
op_open_path_detached(path VARCHAR, app VARCHAR) -> BOOLEAN
```

[`op_open_path`](#op_open_path) 的非阻塞那一对：起启动器，然后立刻返回。

```sql
SELECT op_open_path_detached('https://duckdb.org');
SELECT count(*) FROM reports WHERE op_open_path_detached(url);
```

代价是一个含义更松的 `true`。它说的是有个进程起来了，不是文件最后落到屏幕上了 —— 既然什么都没等，
就没有结果可报告。需要拿到启动器自己的答复就用 `op_open_path`；不希望查询干等着一个窗口被关掉就用这个。

[`op_open_content`](./write-file.md#op_open_content) 与 `op_open_html` 走的也是这条非阻塞的路：画完一张图
再打开它，不该一直挂到查看器被关掉。

## op_open_commands

```text
op_open_commands(path VARCHAR) -> VARCHAR[]
```

这一组里唯一一个什么都不起的调用。它按 `op_open_path` 在这台机器上的尝试顺序，把它会依次执行的启动
命令列成一个字符串列表返回，一条都不真的跑：前一个起不来，才轮到下一个。

```sql
SELECT op_open_commands('report.html');
-- Linux：  ['"xdg-open" "report.html"', '"/usr/bin/xdg-open" "report.html"', …]
-- Windows：['"powershell.exe" "-NoProfile" …', '"explorer.exe" "report.html"']
```

这个形状带来两件事：

- 它回答的是「这台机器上会跑什么命令」，不是「目标程序会不会成功」。命令跑出来的退出码只有真去执行它的
  那一方看得见，那是 `op_open_path` 的事。
- 它的结果只取决于这台机器的配置，不取决于某个窗口有没有真的打开，所以这些启动答复里，它才是可以放心写
  进测试、也可以在服务器上跑的那一个。

返回的是列表，所以 SQL 能直接接着处理这份候选序列：

```sql
SELECT unnest(op_open_commands('report.html')) AS would_run;
```
