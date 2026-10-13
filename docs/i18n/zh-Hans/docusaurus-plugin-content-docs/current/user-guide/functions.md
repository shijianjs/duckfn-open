---
title: 函数
sidebar_position: 3
description: duckfn_open 注册的每一个 SQL 函数，以及可以在你自己的 DuckDB 里跑的例子。
---

# 函数

加载扩展会注册七个函数名、十六个签名，而且它们全都是标量函数：这里没有聚合函数、没有表函数。它还注册
一个 STRUCT 类型 [`open_write_options`](#open_write_options) —— 落盘那一族的可选项就住在这里。这些函数
和 DuckDB 自带的一样用：可以放进投影或 `WHERE` 条件，也能和内置函数随意组合。

名字分成两组。第一组接住一个已经存在的路径或 URL，把它交给桌面：

| 函数 | 类别 | 签名 | 说明 |
| --- | --- | --- | --- |
| [`open_path`](#open_path) | 标量 | `VARCHAR -> BOOLEAN` | 用默认程序打开路径或 URL，并等启动器返回。 |
| [`open_path`](#open_path) | 标量 | `VARCHAR, VARCHAR -> BOOLEAN` | 同上，程序由第二个参数指定。 |
| [`open_path_detached`](#open_path_detached) | 标量 | `VARCHAR -> BOOLEAN` | 打开它，启动器一起来就返回。 |
| [`open_path_detached`](#open_path_detached) | 标量 | `VARCHAR, VARCHAR -> BOOLEAN` | 同上，程序由第二个参数指定。 |
| [`open_commands`](#open_commands) | 标量 | `VARCHAR -> VARCHAR[]` | 列出 `open_path` 会依次尝试的启动命令，一条都不执行。 |

第二组从**内容**出发，而不是从路径出发 —— 这正是一条查询产出的东西通常的形状：一份报表、一张图、一个
还没有文件的页面。

| 函数 | 类别 | 签名 | 说明 |
| --- | --- | --- | --- |
| [`open_write`](#open_write) | 标量 | `VARCHAR, VARCHAR -> VARCHAR` | 把文本写成一个文件并返回它的路径，什么都不启动。 |
| [`open_write`](#open_write) | 标量 | `VARCHAR, VARCHAR, open_write_options -> VARCHAR` | 同上，目录、文件名和名字里的那一段都由你选。 |
| [`open_write`](#open_write) | 标量 | `BLOB, VARCHAR -> VARCHAR` | 二进制内容（比如一张图），按它本来的那些字节写出去。 |
| [`open_write`](#open_write) | 标量 | `BLOB, VARCHAR, open_write_options -> VARCHAR` | 同上，带上那几个选项。 |
| [`open_content`](#open_content) | 标量 | `open_write` 的四个签名，返回 `VARCHAR` | 在同一次调用里写出去再打开：就是 `open_write` 与 `open_path` 的合成。 |
| [`open_html`](#open_html) | 标量 | `VARCHAR -> VARCHAR` | 零配置的捷径：一个临时 `.html`，在默认浏览器里打开。 |
| [`open_print`](#open_print) | 标量 | `VARCHAR -> VARCHAR` | 越过结果集的排版直接写到 stdout。永远返回 NULL。 |
| [`open_print`](#open_print) | 标量 | `VARCHAR, VARCHAR -> VARCHAR` | 同上，第二个参数指定文本后面接什么，取代那个换行。 |

下面这块就是扩展加载之后，DuckDB 眼中的整套 API：

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'open%'
ORDER BY 1, 2;    -- 七个名字，十六个签名
```

其中五个名字各自在一个 SQL 名字下并了多个签名，所以多出来的那些形式都没有自己的 SQL 名字：调用是
`SELECT open_path('a', 'b')` 和 `SELECT open_write(html, 'html', opts)`，不是
`open_path_with('a', 'b')` 或 `open_write_with(…)`。

## 一次调用会答复什么

两组答复的方式不同，而每一种自成一个小小的约定。

**启动那一族**（`open_path`、`open_path_detached`）分辨三种结果：

| 结果 | 含义 |
| --- | --- |
| `NULL` | 这一行没有输入。NULL 永远到不了启动器，所以它不是失败 —— 它表示没让这台机器的桌面做任何事。 |
| `true` | 启动器被拉起了。 |
| `false` | 一个启动器都没能起来：这台机器没有配上任何一个，或者第二个参数指定的程序不存在。 |

```sql
SELECT open_path(NULL) AS opened;    -- NULL：这一行没有输入，所以什么都没启动
```

NULL 跟着行走，不是只跟着常量走：整列都是 NULL 时，一次启动也不会发生。

```sql
SELECT count(*) FROM reports WHERE open_path(path);    -- true 留下这一行，false 与 NULL 把它丢掉
```

失败只能答复 `false`，是因为启动器自己那份问题报告在 SQL 里没有地方放。「为什么什么都没打开」要问
[open_commands](#open_commands)。

**落盘那一族**返回的是一个路径，这就改变了失败能被写在什么地方。`open_write` 什么都不启动，所以它的
三种答复说的都是文件：一个路径、没有输入的那一行是 NULL、以及一个它没能做成文件名的参数是一句错误。
`open_content` 和 `open_html` 在其上又加了一次启动，而路径早就是它们的返回值了 —— 于是起不来的启动器
报成**一句带着路径的错误**：文件已经落在盘上，只是这台机器没有东西能把它显示出来：

```sql
SELECT open_write('report body', 'md');
-- /tmp/open-20260101-120000-9Qm2Zk.md  — 写好了，什么都没启动

SELECT open_content(svg, 'svg');
-- 有桌面的机器：路径，外加一个正在显示它的查看器
-- 没有桌面的机器：一句错误，"the file was written to '<path>', but no launcher could be started"
```

**`open_print`** 永远返回 NULL：文本已经走到 stdout，SQL 这边没有话要说了。

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
看得见的差别就在这一件事上。Windows 上方向是反的：`open_path_detached` 反而是两者里更直接的一条路，
因为它启动的那一步只是一次原生调用，而不是「一个辅助进程，后面还挂着一个兜底的启动器」。

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

`open_content` 和 `open_html` 走的都是这条非阻塞的路：画完一张图再打开它，不该一直挂到查看器被关掉。

## open_commands

```text
open_commands(path VARCHAR) -> VARCHAR[]
```

启动这一族里唯一一个什么都不起的动作。它按 `open_path` 在这台机器上的尝试顺序，把它会依次执行的启动
命令列成一个字符串列表返回，一条都不真的跑：前一个起不来，才轮到下一个。

```sql
SELECT open_commands('report.html');
-- Linux：  ['"xdg-open" "report.html"', '"/usr/bin/xdg-open" "report.html"', …]
-- Windows：['"powershell.exe" "-NoProfile" …', '"explorer.exe" "report.html"']
```

这个形状带来两件事：

- 它回答的是「这台机器上会跑什么命令」，不是「目标程序会不会成功」。命令跑出来的退出码只有真去执行它的
  那一方看得见，那是 `open_path` 的事。
- 它的结果只取决于这台机器的配置，不取决于某个窗口有没有真的打开，所以这些启动答复里，它才是可以放心写
  进测试、也可以在服务器上跑的那一个。

返回的是列表，所以 SQL 能直接接着处理这份候选序列：

```sql
SELECT unnest(open_commands('report.html')) AS would_run;
```

## open_write

```text
open_write(content VARCHAR, extension VARCHAR) -> VARCHAR
open_write(content VARCHAR, extension VARCHAR, opts open_write_options) -> VARCHAR
open_write(content BLOB,    extension VARCHAR) -> VARCHAR
open_write(content BLOB,    extension VARCHAR, opts open_write_options) -> VARCHAR
```

把一段内容变成一个文件，返回它写到的那个路径。它不起任何进程 —— 这正是它能拼起来的原因：拿着这个路径去
[`open_path`](#open_path) 就能挑程序，去 [`open_content`](#open_content) 就用默认的那个。

```sql
SELECT open_write('{"a":1}', 'json');
SELECT open_write(svg, 'svg', {'dir':'./out', 'name':'chart', 'part':NULL});
SELECT open_write(read_blob('logo.png'), 'png');
```

**扩展名是唯一必填的设置**，因为系统就是照它决定怎么对待这个文件：`html` 与 `.html` 都指 `.html`；
空的扩展名 —— 或者带路径分隔符的（那其实是一个伪装成扩展名的路径）—— 是一句错误，而不是一次猜测。
对 BLOB 它更要紧：没有后缀，查看器根本不知道自己看到的是什么。

其余的都有默认值：

- **`dir`** —— 不填就是系统临时目录。不存在的目录会连同它的父目录一起建出来，因为写下
  `file.dir = '/tmp/charts'` 的人意思是「就该放在这儿」，不是「这儿已经有了」。空字符串是一句错误。
- **`name`** —— 原样使用；它自己不带扩展名时按 `.svg` 这种方式补齐后缀，而**这个路径上已有的文件会被
  覆盖**：名字是你起的，你要的就是这个路径。
- 没有 `name` 时，名字按 `open-<time>-<part>-<random>.<extension>` 生成：
  `open-20260101-120000-Sales_Overview-9Qm2Zk.svg`。时间紧跟在前缀后面，所以按名字排一个目录就是按时间
  排；随机后缀是用来分开同一秒里做出的两张图的，它同时也保证自动生成的名字永远不会落在一个已有的文件上。
- **`part`** —— 塞进生成名字里的那段有意义的东西（图表标题、报表名），不这么做的下场就是一目录的时间戳。
  它最长 48 个字符。

这三个选项都被当成文件名的材料：非法字符、控制字符、Windows 保留的设备名、结尾的点与空格都会被替换掉，
`part` 里的空格变成 `_`。因此一个带斜杠的 `part` 逃不出它所在的目录，而中文标题、带标点的标题也照样塞得
进一个名字。

目录与文件名在任何平台上都一律用 `/` 拼接，因为这个字符串是人来读的，而且下游的查询通常还会继续把它拼
下去 —— 而 `std::fs` 在 Windows 上也接受它。目录那一段长什么样，取决于你给了什么：你写的 `dir` 原样
使用；它默认到系统临时目录时，你得到的就是那个目录自己的形状（Windows 上是
`C:\Users\…\Temp/open-….json`）。

### open_write_options

`open_write_options` 是扩展注册的一个 STRUCT 类型，那几个带选项的签名就是拿它当第三个参数：

```text
open_write_options(dir VARCHAR, "name" VARCHAR, part VARCHAR)
```

这个字段显示成 `"name"`，因为 NAME 是 DuckDB 语法里的关键字；结构体字面量里写 `name:` 或 `'name':`
都能用。

**三个字段都可省，但省掉其中任何一个的字面量都需要一次转换。** DuckDB 不会把 `{'dir':'./out'}` 隐式
加宽成三字段的 STRUCT，所以要么每个字段都写，要么带上转换：

```sql
SELECT open_write(svg, 'svg', {'dir':'./out', 'name':NULL, 'part':'sales'});
SELECT open_write(svg, 'svg', {'dir':'./out'}::open_write_options);
```

整个 STRUCT 作为 NULL 传进来就是一个 NULL 参数，所以它会像内容为 NULL 那样短路 —— 它不是「用默认值」。
想要默认值就别写第三个参数。

## open_content

```text
open_content(content VARCHAR, extension VARCHAR) -> VARCHAR
open_content(content VARCHAR, extension VARCHAR, opts open_write_options) -> VARCHAR
open_content(content BLOB,    extension VARCHAR) -> VARCHAR
open_content(content BLOB,    extension VARCHAR, opts open_write_options) -> VARCHAR
```

一次调用同时做完 `open_write` 和 `open_path`，这是探索过程中最常见的那件事：查询手里有一段内容，你想
看一眼。

```sql
SELECT open_content(read_text('report.html'), 'html');
SELECT open_content(svg, 'svg', {'dir':'./out', 'name':NULL, 'part':'sales'});
SELECT open_content(logo_blob, 'png');
```

上面那套落盘规则，它和 `open_write` 用的是同一条。它加上的那次启动带来两个行为：它不等程序退出；而起不
来的启动器是一句带着路径的错误，不是 `open_path` 会答的那个 `false` —— 返回值已经花在路径上了，一个光秃
秃的 `false` 会把这个路径扔掉。

挑哪个程序，有意不归这个函数管；选项 STRUCT 里没有 `app` 字段，理由也是这个。要指定程序就把调用拼出来：

```sql
SELECT open_path(open_write(html, 'html'), 'firefox');
```

## open_html

```text
open_html(content VARCHAR) -> VARCHAR
```

[`open_content`](#open_content)，三个选项全都留在默认值上、扩展名钉死成 `html`：一个临时文件，在系统
默认浏览器里打开，返回路径。

```sql
SELECT open_html('<h1>hello</h1>');
SELECT open_html(string_agg(row, chr(10))) FROM rendered_rows t(row);
```

它之所以存在，是因为 SQL 渲染出来的东西十有八九是 HTML，而每次都要敲上扩展名再加一个空 STRUCT 是噪音。
当你需要一个目录、一个名字、名字里一段认得出来的字，或者默认浏览器以外的浏览器，那就是
`open_content(content, 'html', opts)`，或者上面那个 `open_path` 组合。这里没有 BLOB 签名：它的输入是
HTML *文本*。

## open_print

```text
open_print(text VARCHAR) -> VARCHAR
open_print(text VARCHAR, tail VARCHAR) -> VARCHAR
```

把文本直接写到 stdout 并返回 NULL，为的是绕开这件事：DuckDB CLI 会对结果*排版* —— 列宽、边框、折行 ——
而字符画（`textplot` 这类扩展产出的 ASCII 图或盲文图）是一行一行读的，把它塞进一个 VARCHAR 单元格里就
把它毁掉了。

```sql
SELECT open_print(textplot(y)) FROM series;           -- 一张图，打在控制台上
SELECT open_print('rows: ' || count(*)) FROM t;       -- 一句不占用结果集的备注
```

两个签名按「文本后面跟着什么」分开：没有 `tail` 时补一个换行，好让下一行输出不跟在这一行后面；给了
`tail` 就原样写它 —— `''` 表示「接着这一行往下写」，`chr(10) || chr(10)` 表示「留一个空行」。每次调用
都会 flush，因为 Rust 按行缓冲 stdout，一个不带换行的 tail 否则会把文本留在缓冲区里。

它是个标量函数，所以按行求值：一个多行的查询打出来的是一份按执行顺序滚动的日志，而不是一份排好版的
报表 —— 报表那是结果集的活。有一个例外要提前算进去：参数是常量时，DuckDB 会把调用折叠成一次求值，所以
`SELECT open_print('x') FROM t` 无论 `t` 有多少行都只打一行。想要一行打一行，就得引用一个列
（`open_print('x: ' || a)`）。

## 报错

空的路径、空的程序名都是错误，而不是一个 `false` —— 「写错了」和「打不开」该有不同的答复。DuckDB 把它
报成 invalid input error，报错信息里带函数自己的名字：

```sql
SELECT open_path('');    -- 报错：open_path: the path or URL must not be empty
```

落盘那一族报的方式一样，一个用不了的设置一句：

```sql
SELECT open_write('a', '');                       -- 报错：open_write: the extension must not be empty …
SELECT open_write('a', 'txt', {'dir':'  ', 'name':NULL, 'part':NULL});
                                                  -- 报错：… open_write_options.dir must not be empty
SELECT open_content('a', 'txt', {'dir':'./out', 'name':'/', 'part':NULL});
                                                  -- 报错：… open_write_options.name is not a usable file name
```

`open_path` 的第二个参数有它自己那句文案（`… the application must not be empty`），而参数的个数与类型
在绑定查询时就把关：`open_path()`、`open_path('a', 'b', 'c')`、`open_path(42)` 全都在任何东西跑起来
之前失败。

## 几点说明

- **调用背后得有一台桌面。** 启动器是桌面软件，所以在没有桌面的机器上 —— 服务器、CI runner —— 四个
  `open_path` 动作一律答复 `false`，`open_content` 和 `open_html` 报一句带着它们确实写出的那个路径的
  错误，而 `open_commands` 是那个照样能答的调用，它列出这台机器上会依次尝试什么。`open_write` 和
  `open_print` 根本不需要桌面。
- **NULL 进、NULL 出 —— 而且什么都没启动、什么都没落盘。** 三种结果因此读得出来：NULL 就是这一行没有
  输入。
- **被拒绝的输入会让整条语句失败。** 报错信息里带函数名，查询剩下的部分不会再执行。
- **常量会被折叠成一次执行。** `SELECT open_path('report.html')` 只跑一次。参数一旦来自某一列，调用就
  按行跑 —— `SELECT open_path(p) FROM reports` 一行开一个窗口，不去重。按行跑 `open_write` 就是一行一个
  文件，这正是那套生成名字要应付的。
- **列表按类型打印。** `open_commands` 返回 `VARCHAR[]`，所以里面的每一项都保留它被启动时那副引用样子；
  要一条命令一行就取下标（`open_commands('report.html')[1]`）或者 `unnest`。
