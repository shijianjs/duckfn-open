---
title: 函数
sidebar_position: 3
description: duckfn_open 注册的每一个 SQL 函数，以及可以在你自己的 DuckDB 里跑的例子。
---

# 函数

加载扩展会注册三个函数名、五个签名，而且它们全都是标量函数：这里没有聚合函数、没有表函数，也没有
自定义类型。它们和 DuckDB 自带的函数一样用：可以放进投影或 `WHERE` 条件，也能和内置函数随意组合。

| 函数 | 类别 | 签名 | 说明 |
| --- | --- | --- | --- |
| [`open_path`](#open_path) | 标量 | `VARCHAR -> BOOLEAN` | 用默认程序打开路径或 URL，并等启动器返回。 |
| [`open_path`](#open_path) | 标量 | `VARCHAR, VARCHAR -> BOOLEAN` | 同上，程序由第二个参数指定。 |
| [`open_path_detached`](#open_path_detached) | 标量 | `VARCHAR -> BOOLEAN` | 打开它，启动器一起来就返回。 |
| [`open_path_detached`](#open_path_detached) | 标量 | `VARCHAR, VARCHAR -> BOOLEAN` | 同上，程序由第二个参数指定。 |
| [`open_commands`](#open_commands) | 标量 | `VARCHAR -> VARCHAR[]` | 列出 `open_path` 会依次尝试的启动命令，一条都不执行。 |

扩展加载后，DuckDB 眼中的这套 API 就是下面这些：

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'open%'
ORDER BY 1, 2;    -- 三个名字，五个签名
```

`open_path` 与 `open_path_detached` 各自以一个名字覆盖两种参数个数，所以两参数形式没有自己的 SQL
名字：调用是 `SELECT open_path('a', 'b')`，不是 `open_path_with('a', 'b')`。

## 一次调用会答复什么

SQL 里能分辨三种结果，它们合起来就是全部约定：

| 结果 | 含义 |
| --- | --- |
| `NULL` | 这一行没有输入。NULL 永远到不了启动器，所以它不是失败 —— 它表示这台机器的桌面没被问任何东西。 |
| `true` | 启动器被拉起了。 |
| `false` | 一个启动器都没能起来：这台机器没有配上任何一个，或者第二个参数指定的程序不存在。 |

```sql
SELECT open_path(NULL) AS opened;    -- NULL：这一行没有输入，所以什么都没启动
```

NULL 跟着行走，不是只跟着常量走：整列都是 NULL 时，一次 launch 也不会发生。

```sql
SELECT count(*) FROM reports WHERE open_path(path);    -- true 留下这一行，false 与 NULL 把它丢掉
```

失败只能答复 `false`，是因为启动器自己那份问题报告在 SQL 里没有地方放。「为什么什么都没打开」要问
[open_commands](#open_commands)。

## open_path

```text
open_path(path VARCHAR) -> BOOLEAN
open_path(path VARCHAR, app VARCHAR) -> BOOLEAN
```

把一个本地路径或 URL 交给桌面当前配置好的程序 —— 也就是双击这个文件时接住它的那个程序；面对一个 URL
时，那就是默认浏览器。第二个签名改为由你指定程序。

```sql
SELECT open_path('report.html');
SELECT open_path('https://duckdb.org', 'firefox');
SELECT open_path('/data/report.csv', 'LibreOffice Calc');
```

**这个调用会等。** 它在启动器返回时才返回，而 Linux 上常见的启动器通常要到打开这个文件的那个程序退出
时才返回。这就是这个名字与 [`open_path_detached`](#open_path_detached) 之间的差别 —— 一条查询要跑多久，
可见的差别就在这一件事上。Windows 上方向是反的：`open_path_detached` 反而是两者里更直接的一条路，因为
它启动的那一步只是一次原生调用，而不是「一个辅助进程，后面还挂着一个兜底的启动器」。

`app` 参数会原样交给平台：Linux 上是能在 `PATH` 里找到的可执行名，Windows 上先在已安装应用程序里查
这个名、再交给 shell。既不解析别名，也不校验它存在与否 —— 起不来的程序就是一个 `false`。

## open_path_detached

```text
open_path_detached(path VARCHAR) -> BOOLEAN
open_path_detached(path VARCHAR, app VARCHAR) -> BOOLEAN
```

`open_path` 的非阻塞那一对：起启动器，然后立刻返回。

```sql
SELECT open_path_detached('https://duckdb.org');
SELECT count(*) FROM reports WHERE open_path_detached(url);
```

代价是一个含义更松的 `true`。它说的是有个进程起来了，不是文件最后落到屏幕上了 —— 既然什么都没等，
就没有结果可报告。需要拿到启动器自己的答复就用 `open_path`；不希望查询干等着一个窗口被关掉就用这个。

## open_commands

```text
open_commands(path VARCHAR) -> VARCHAR[]
```

这个扩展里唯一一个什么都不启动的动作。它按 `open_path` 在这台机器上的尝试顺序，把它会依次执行的启动
命令列成一个字符串列表返回，一条都不真的跑：前一个起不来，才轮到下一个。

```sql
SELECT open_commands('report.html');
-- Linux：  ['"xdg-open" "report.html"', '"/usr/bin/xdg-open" "report.html"', …]
-- Windows：['"powershell.exe" "-NoProfile" …', '"explorer.exe" "report.html"']
```

这个形状带来两件事：

- 它回答的是「这台机器上会跑什么命令」，不是「目标程序会不会成功」。命令跑出来的退出码只有真去执行的
  那一方看得见，那是 `open_path` 的事。
- 它的结果只取决于这台机器的配置，不取决于某个窗口有没有真的打开，所以它是这套 API 里唯一可以放心写进
  测试、也可以在服务器上跑的那个调用。

返回的是列表，所以 SQL 能直接接着处理这份候选序列：

```sql
SELECT unnest(open_commands('report.html')) AS would_run;
```

## 报错

空的路径、空的程序名都是错误，而不是一个 `false` —— 「写错了」和「打不开」该有不同的答复。DuckDB 把它
报成 invalid input error，报错信息里带函数自己的名字：

```sql
SELECT open_path('');    -- 报错：open_path: the path or URL must not be empty
```

第二个参数有它自己那句文案（`… the application must not be empty`），而参数的个数与类型在绑定查询时
就把关：`open_path()`、`open_path('a', 'b', 'c')`、`open_path(42)` 全都在任何东西跑起来之前失败。

## 几点说明

- **调用背后得有一台桌面。** 启动器是桌面软件，所以在没有桌面的机器上 —— 服务器、CI runner —— 四个启动
  动作一律答复 `false`，而 `open_commands` 是那个照样能答的调用，它列出这台机器上会依次尝试什么。
- **NULL 进、NULL 出 —— 而且什么都没启动。** 三种结果因此读得出来：NULL 是这一行没有输入，`false` 是
  这台机器起不动启动器，`true` 是启动器被起来了。
- **被拒绝的输入会让整条语句失败。** 报错信息里带函数名，查询剩下的部分不会再执行。
- **常量参数会被折叠成一次执行。** `SELECT open_path('report.html')` 只跑一次。参数一旦来自某一列，调用
  就按行跑 —— `SELECT open_path(p) FROM reports` 一行开一个窗口，不去重。
- **列表按类型打印。** `open_commands` 返回 `VARCHAR[]`，所以里面的每一项都保留它被启动时那副引用样子；
  要一条命令一行就取下标（`open_commands('report.html')[1]`）或者 `unnest`。
