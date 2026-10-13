---
title: 函数
sidebar_position: 3
description: duckfn_open 注册的每一个 SQL 函数，以及可以在你自己的 DuckDB 里跑的例子。
---

# 函数

加载扩展会注册九个函数名、二十一个签名，而且它们全都是标量函数：这里没有聚合函数、没有表函数。它还注册
一个 STRUCT 类型 [`op_write_options`](#op_write_options) —— 落盘那一族的可选项就住在这里；新加的这两个
名字不注册任何类型。这些函数和 DuckDB 自带的一样用：可以放进投影或 `WHERE` 条件，也能和内置函数随意组合。

名字分成三组。第一组接住一个已经存在的路径或 URL，把它交给桌面：

| 函数 | 类别 | 签名 | 说明 |
| --- | --- | --- | --- |
| [`op_open_path`](#op_open_path) | 标量 | `VARCHAR -> BOOLEAN` | 用默认程序打开路径或 URL，并等启动器返回。 |
| [`op_open_path`](#op_open_path) | 标量 | `VARCHAR, VARCHAR -> BOOLEAN` | 同上，程序由第二个参数指定。 |
| [`op_open_path_detached`](#op_open_path_detached) | 标量 | `VARCHAR -> BOOLEAN` | 打开它，启动器一起来就返回。 |
| [`op_open_path_detached`](#op_open_path_detached) | 标量 | `VARCHAR, VARCHAR -> BOOLEAN` | 同上，程序由第二个参数指定。 |
| [`op_open_commands`](#op_open_commands) | 标量 | `VARCHAR -> VARCHAR[]` | 列出 `op_open_path` 会依次尝试的启动命令，一条都不执行。 |

第二组从**内容**出发，而不是从路径出发 —— 这正是一条查询产出的东西通常的形状：一份报表、一张图、一个
还没有文件的页面。

| 函数 | 类别 | 签名 | 说明 |
| --- | --- | --- | --- |
| [`op_write`](#op_write) | 标量 | `VARCHAR, VARCHAR -> VARCHAR` | 把文本写成一个文件并返回它的路径，什么都不启动。 |
| [`op_write`](#op_write) | 标量 | `VARCHAR, VARCHAR, op_write_options -> VARCHAR` | 同上，目录、文件名和名字里的那一段都由你选。 |
| [`op_write`](#op_write) | 标量 | `BLOB, VARCHAR -> VARCHAR` | 二进制内容（比如一张图），按它本来的那些字节写出去。 |
| [`op_write`](#op_write) | 标量 | `BLOB, VARCHAR, op_write_options -> VARCHAR` | 同上，带上那几个选项。 |
| [`op_open_content`](#op_open_content) | 标量 | `op_write` 的四个签名，返回 `VARCHAR` | 在同一次调用里写出去再打开：就是 `op_write` 与 `op_open_path` 的合成。 |
| [`op_open_html`](#op_open_html) | 标量 | `VARCHAR -> VARCHAR` | 零配置的捷径：一个临时 `.html`，在默认浏览器里打开。 |
| [`op_print`](#op_print) | 标量 | `VARCHAR -> VARCHAR` | 越过结果集的排版直接写到 stdout。永远返回 NULL。 |
| [`op_print`](#op_print) | 标量 | `VARCHAR, VARCHAR -> VARCHAR` | 同上，第二个参数指定文本后面接什么，取代那个换行。 |

第三组既不碰桌面，也不碰文件系统。它做的是纯字符串的事，而它维护的是「这个值到底能不能当一个**文件名**」：

| 函数 | 类别 | 签名 | 说明 |
| --- | --- | --- | --- |
| [`op_filename`](#op_filename) | 标量 | `VARCHAR -> VARCHAR` | 把一个字符串改写成一个可用的文件名。 |
| [`op_filename`](#op_filename) | 标量 | `VARCHAR, VARCHAR -> VARCHAR` | 同上，第二个参数指定每个非法字符换成什么，而不是直接删掉。 |
| [`op_filename`](#op_filename) | 标量 | `VARCHAR, VARCHAR, BOOLEAN, BOOLEAN -> VARCHAR` | 同上，再加上那两个开关：255 字节这一刀，以及 Windows 那套规则。 |
| [`op_filename_check`](#op_filename_check) | 标量 | `VARCHAR -> BOOLEAN` | 什么都不改写：这个字符串本来就是一个可用的名字吗？ |
| [`op_filename_check`](#op_filename_check) | 标量 | `VARCHAR, BOOLEAN, BOOLEAN -> BOOLEAN` | 同一个问题，带上那两个开关。 |

下面这块就是扩展加载之后，DuckDB 眼中的整套 API：

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'op%'
ORDER BY 1, 2;    -- 九个名字，二十一个签名
```

其中七个名字各自在一个 SQL 名字下并了多个签名，所以多出来的那些形式都没有自己的 SQL 名字：调用是
`SELECT op_open_path('a', 'b')` 和 `SELECT op_write(html, 'html', opts)`，不是
`op_open_path_with('a', 'b')` 或 `op_write_with(…)`。

## 一次调用会答复什么

三组答复的方式不同，而每一种自成一个小小的约定。

**启动那一族**（`op_open_path`、`op_open_path_detached`）分辨三种结果：

| 结果 | 含义 |
| --- | --- |
| `NULL` | 这一行没有输入。NULL 永远到不了启动器，所以它不是失败 —— 它表示没让这台机器的桌面做任何事。 |
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

**落盘那一族**返回的是一个路径，这就改变了失败能被写在什么地方。`op_write` 什么都不启动，所以它的
三种答复说的都是文件：一个路径、没有输入的那一行是 NULL、以及一个它没能做成文件名的参数是一句错误。
`op_open_content` 和 `op_open_html` 在其上又加了一次启动，而路径早就是它们的返回值了 —— 于是起不来的启动器
报成**一句带着路径的错误**：文件已经落在盘上，只是这台机器没有东西能把它显示出来：

```sql
SELECT op_write('report body', 'md');
-- /tmp/open-20260101-120000-9Qm2Zk.md  — 写好了，什么都没启动

SELECT op_open_content(svg, 'svg');
-- 有桌面的机器：路径，外加一个正在显示它的查看器
-- 没有桌面的机器：一句错误，"the file was written to '<path>', but no launcher could be started"
```

**`op_print`** 永远返回 NULL：文本已经走到 stdout，SQL 这边没有话要说了。

**文件名那一族**答复的就是那个值本身 —— 一个改写好的字符串，或者关于递给它的那个字符串的
`true`/`false`。它不起进程、不写文件，所以再空的机器也容得下它；而**一个不能用的名字并不是一句错误**：
`op_filename('CON.txt')` 答复 `''`，因为「改写之后什么都不剩」是一个合法答案，不是被拒绝的参数。在这里
算错的，还是这个扩展里到处都算错的那两件事 —— 参数的个数不对、类型不对，在绑定查询时就被拦下。

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
时才返回。这就是这个名字与 [`op_open_path_detached`](#op_open_path_detached) 之间的差别 —— 一条查询要跑多久，
看得见的差别就在这一件事上。Windows 上方向是反的：`op_open_path_detached` 反而是两者里更直接的一条路，
因为它启动的那一步只是一次原生调用，而不是「一个辅助进程，后面还挂着一个兜底的启动器」。

`app` 参数会原样交给平台：Linux 上是能在 `PATH` 里找到的可执行名，Windows 上先在已安装应用程序里查
这个名、再交给 shell。既不解析别名，也不校验它存在与否 —— 起不来的程序就是一个 `false`。

## op_open_path_detached

```text
op_open_path_detached(path VARCHAR) -> BOOLEAN
op_open_path_detached(path VARCHAR, app VARCHAR) -> BOOLEAN
```

`op_open_path` 的非阻塞那一对：起启动器，然后立刻返回。

```sql
SELECT op_open_path_detached('https://duckdb.org');
SELECT count(*) FROM reports WHERE op_open_path_detached(url);
```

代价是一个含义更松的 `true`。它说的是有个进程起来了，不是文件最后落到屏幕上了 —— 既然什么都没等，
就没有结果可报告。需要拿到启动器自己的答复就用 `op_open_path`；不希望查询干等着一个窗口被关掉就用这个。

`op_open_content` 和 `op_open_html` 走的都是这条非阻塞的路：画完一张图再打开它，不该一直挂到查看器被关掉。

## op_open_commands

```text
op_open_commands(path VARCHAR) -> VARCHAR[]
```

启动这一族里唯一一个什么都不起的动作。它按 `op_open_path` 在这台机器上的尝试顺序，把它会依次执行的启动
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

## op_write

```text
op_write(content VARCHAR, extension VARCHAR) -> VARCHAR
op_write(content VARCHAR, extension VARCHAR, opts op_write_options) -> VARCHAR
op_write(content BLOB,    extension VARCHAR) -> VARCHAR
op_write(content BLOB,    extension VARCHAR, opts op_write_options) -> VARCHAR
```

把一段内容变成一个文件，返回它写到的那个路径。它不起任何进程 —— 这正是它能拼起来的原因：拿着这个路径去
[`op_open_path`](#op_open_path) 就能挑程序，去 [`op_open_content`](#op_open_content) 就用默认的那个。

```sql
SELECT op_write('{"a":1}', 'json');
SELECT op_write(svg, 'svg', {'dir':'./out', 'name':'chart', 'part':NULL});
SELECT op_write(read_blob('logo.png'), 'png');
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

### op_write_options

`op_write_options` 是扩展注册的一个 STRUCT 类型，那几个带选项的签名就是拿它当第三个参数：

```text
op_write_options(dir VARCHAR, "name" VARCHAR, part VARCHAR)
```

这个字段显示成 `"name"`，因为 NAME 是 DuckDB 语法里的关键字；结构体字面量里写 `name:` 或 `'name':`
都能用。

**三个字段都可省，但省掉其中任何一个的字面量都需要一次转换。** DuckDB 不会把 `{'dir':'./out'}` 隐式
加宽成三字段的 STRUCT，所以要么每个字段都写，要么带上转换：

```sql
SELECT op_write(svg, 'svg', {'dir':'./out', 'name':NULL, 'part':'sales'});
SELECT op_write(svg, 'svg', {'dir':'./out'}::op_write_options);
```

整个 STRUCT 作为 NULL 传进来就是一个 NULL 参数，所以它会像内容为 NULL 那样短路 —— 它不是「用默认值」。
想要默认值就别写第三个参数。

## op_open_content

```text
op_open_content(content VARCHAR, extension VARCHAR) -> VARCHAR
op_open_content(content VARCHAR, extension VARCHAR, opts op_write_options) -> VARCHAR
op_open_content(content BLOB,    extension VARCHAR) -> VARCHAR
op_open_content(content BLOB,    extension VARCHAR, opts op_write_options) -> VARCHAR
```

一次调用同时做完 `op_write` 和 `op_open_path`，这是探索过程中最常见的那件事：查询手里有一段内容，你想
看一眼。

```sql
SELECT op_open_content(read_text('report.html'), 'html');
SELECT op_open_content(svg, 'svg', {'dir':'./out', 'name':NULL, 'part':'sales'});
SELECT op_open_content(logo_blob, 'png');
```

上面那套落盘规则，它和 `op_write` 用的是同一条。它加上的那次启动带来两个行为：它不等程序退出；而起不
来的启动器是一句带着路径的错误，不是 `op_open_path` 会答的那个 `false` —— 返回值已经花在路径上了，一个光秃
秃的 `false` 会把这个路径扔掉。

挑哪个程序，有意不归这个函数管；选项 STRUCT 里没有 `app` 字段，理由也是这个。要指定程序就把调用拼出来：

```sql
SELECT op_open_path(op_write(html, 'html'), 'firefox');
```

## op_open_html

```text
op_open_html(content VARCHAR) -> VARCHAR
```

[`op_open_content`](#op_open_content)，三个选项全都留在默认值上、扩展名钉死成 `html`：一个临时文件，在系统
默认浏览器里打开，返回路径。

```sql
SELECT op_open_html('<h1>hello</h1>');
SELECT op_open_html(string_agg(row, chr(10))) FROM rendered_rows t(row);
```

它之所以存在，是因为 SQL 渲染出来的东西十有八九是 HTML，而每次都要敲上扩展名再加一个空 STRUCT 是噪音。
当你需要一个目录、一个名字、名字里一段认得出来的字，或者默认浏览器以外的浏览器，那就是
`op_open_content(content, 'html', opts)`，或者上面那个 `op_open_path` 组合。这里没有 BLOB 签名：它的输入是
HTML *文本*。

## op_print

```text
op_print(text VARCHAR) -> VARCHAR
op_print(text VARCHAR, tail VARCHAR) -> VARCHAR
```

把文本直接写到 stdout 并返回 NULL，为的是绕开这件事：DuckDB CLI 会对结果*排版* —— 列宽、边框、折行 ——
而字符画（`textplot` 这类扩展产出的 ASCII 图或盲文图）是一行一行读的，把它塞进一个 VARCHAR 单元格里就
把它毁掉了。

```sql
SELECT op_print(textplot(y)) FROM series;           -- 一张图，打在控制台上
SELECT op_print('rows: ' || count(*)) FROM t;       -- 一句不占用结果集的备注
```

两个签名按「文本后面跟着什么」分开：没有 `tail` 时补一个换行，好让下一行输出不跟在这一行后面；给了
`tail` 就原样写它 —— `''` 表示「接着这一行往下写」，`chr(10) || chr(10)` 表示「留一个空行」。每次调用
都会 flush，因为 Rust 按行缓冲 stdout，一个不带换行的 tail 否则会把文本留在缓冲区里。

它是个标量函数，所以按行求值：一个多行的查询打出来的是一份按执行顺序滚动的日志，而不是一份排好版的
报表 —— 报表那是结果集的活。有一个例外要提前算进去：参数是常量时，DuckDB 会把调用折叠成一次求值，所以
`SELECT op_print('x') FROM t` 无论 `t` 有多少行都只打一行。想要一行打一行，就得引用一个列
（`op_print('x: ' || a)`）。

## op_filename

```text
op_filename(value VARCHAR) -> VARCHAR
op_filename(value VARCHAR, replacement VARCHAR) -> VARCHAR
op_filename(value VARCHAR, replacement VARCHAR, truncate BOOLEAN, windows BOOLEAN) -> VARCHAR
```

把一个字符串改写成一个能当文件名用的东西。除此之外什么都不发生 —— 没有进程、没有文件、没有目录：值进去，
值出来。这套规则管的是所有让一个字符串当不了名字的东西：非法字符 `/ \ ? < > : * | "`、控制字符、整段只是
点的名字（`.` 和 `..`）、Windows 保留的设备名（`CON`、`PRN`、`AUX`、`NUL`、`COM0`–`COM9`、`LPT0`–`LPT9`，
以及这些名字后面再挂上后缀的）、结尾的点与空格，还有 255 **字节**这一刀。

一条查询会要它，是因为查询里的文件名常常来自数据本身 —— 客户名、报表标题、抓回来的那一列 `path`。先把
一个值变成一个合法的名字，才谈得上拼目录、拼 URL、交给下一个扩展。

```sql
SELECT op_filename('quarterly/report:v2.csv');   -- 'quarterlyreportv2.csv'
SELECT op_filename('  the report.  ');           -- '  the report' —— 结尾被去掉，开头不动
SELECT op_filename('CON.txt');                   -- '' —— 保留设备名不是一个名字
SELECT op_filename('..');                        -- ''
```

缺省跟着这个库自己那一套，只有**一处是有意的例外**。`replacement` 是空串，所以非法字符是被**删掉**的，
不是被替换的；`truncate` 是开的。`windows` 却不跟 —— 它**无论宿主平台是什么都钉死成 `true`**：这个库里它的
缺省是编译时所在的那个平台，而那会让同一条 SQL 在不同机器上给出不同的答案。想要 Unix 那套规则就把这一项
传成 `false` —— 在那套规则下 `CON.txt` 就是一个普通文件名：

```sql
SELECT op_filename('a/b:c?.txt', '_');    -- 'a_b_c_.txt' —— 替换符让改过的地方看得见
SELECT op_filename('sales|east', '-');    -- 'sales-east'；替换符可以是任意字符串，也包括 ''
SELECT op_filename('CON.txt', '_', true, false);   -- 'CON.txt'
SELECT length(op_filename(repeat('a', 300)));                 -- 255
SELECT length(op_filename(repeat('a', 300), '_', false, true));  -- 300
```

替换符只管显示：`'_'` 让名字的轮廓还看得出来，`''`（缺省那一个，也可以显式写出来）是同一次改写，只是把
窟窿合上了。

**`truncate` 与 `windows` 只能一起给**，也就是四个参数那一档的第三、第四项；这里**有意没有三个参数的
形式**，所以 `op_filename('a.txt', '_', true)` 是一句 binder 错误。这是这份 API 里唯一的一处压缩，不是
少了一个用例：两个 BOOLEAN 摆在可选位置上是五种组合，而其中只有这个库自己缺省的那两种值得单独给一个更短
的形式。

**任何**一个参数是 NULL，这一行就是 NULL。缺掉的那个开关没有东西可以猜 —— 一个被截断的名字，和一个根本
没被问过要不要截断的名字，是两件事。

这两个函数不是 [`op_write`](#op_write) 的助手。那一族自己的命名策略是住在写入器内部的第二套缺省 ——
`name` 把非法字符换成 `_`、不截断，`part` 把空格并成 `_`、最长 48 个字符 —— 它从来不是一个 SQL 名字，
两套也都不能顶替对方。

## op_filename_check

```text
op_filename_check(value VARCHAR) -> BOOLEAN
op_filename_check(value VARCHAR, truncate BOOLEAN, windows BOOLEAN) -> BOOLEAN
```

什么都不改写的那一半：`true` 的意思是，在同样的开关下 `op_filename` 会把它拿到的那个值原样还回来。

```sql
SELECT op_filename_check('report.csv');   -- true
SELECT op_filename_check('a/b.txt');      -- false
SELECT op_filename_check('CON.txt');      -- false
SELECT op_filename_check('trail. ');      -- false
SELECT count(*) FROM files WHERE NOT op_filename_check(name);   -- 有几条会被改动
```

最后那一行就是这一对存在的原因：一条查询能数出改写会动到多少条，而在这层一条都不动 —— 这是有人要决定
怎么处置这批数据之前的体检与过滤那一步。

这里没有 `replacement` 参数，因为一个什么都不改写的问题，没有「换成什么」这一项。其余全都从
[`op_filename`](#op_filename) 沿用过来：同样那两个开关，而且只能一起给（所以带选项那一档是三个参数，
不是两个）；`windows` 钉成 `true`，所以 `op_filename_check('CON.txt')` 在 Linux 上也是 `false`，尽管那个
平台什么都不保留 —— `op_filename_check('CON.txt', true, false)` 答复 `true`；还有任何一个参数是 NULL，
这一行就是 NULL。

## 报错

空的路径、空的程序名都是错误，而不是一个 `false` —— 「写错了」和「打不开」该有不同的答复。DuckDB 把它
报成 invalid input error，报错信息里带函数自己的名字：

```sql
SELECT op_open_path('');    -- 报错：op_open_path: the path or URL must not be empty
```

落盘那一族报的方式一样，一个用不了的设置一句：

```sql
SELECT op_write('a', '');                       -- 报错：op_write: the extension must not be empty …
SELECT op_write('a', 'txt', {'dir':'  ', 'name':NULL, 'part':NULL});
                                                  -- 报错：… op_write_options.dir must not be empty
SELECT op_open_content('a', 'txt', {'dir':'./out', 'name':'/', 'part':NULL});
                                                  -- 报错：… op_write_options.name is not a usable file name
```

`op_open_path` 的第二个参数有它自己那句文案（`… the application must not be empty`），而参数的个数与类型
在绑定查询时就把关：`op_open_path()`、`op_open_path('a', 'b', 'c')`、`op_open_path(42)` 全都在任何东西跑起来
之前失败。文件名那一族绑的方式一样，而它有一处形状值得绊一下 —— 就是那个并不存在的三参数调用：

```sql
SELECT op_filename('a.txt', '_', true);          -- Binder Error：truncate 与 windows 是一对
SELECT op_filename_check('a.txt', true);         -- Binder Error：带选项的形式是三个参数
```

`op_filename` / `op_filename_check` 里没有任何一条是关于*值*本身的错误：非法字符、保留设备名、三百个
字符的标题，都得到一个答案，而不是一句报错。

## 几点说明

- **调用背后得有一台桌面。** 启动器是桌面软件，所以在没有桌面的机器上 —— 服务器、CI runner —— 四个
  `op_open_path` 动作一律答复 `false`，`op_open_content` 和 `op_open_html` 报一句带着它们确实写出的那个路径的
  错误，而 `op_open_commands` 是那个照样能答的调用，它列出这台机器上会依次尝试什么。`op_write`、
  `op_print` 和文件名那一族根本不需要桌面。
- **NULL 进、NULL 出 —— 而且什么都没启动、什么都没落盘。** 三种结果因此读得出来：NULL 就是这一行没有
  输入。对 `op_filename` 与 `op_filename_check` 来说，它指的是**任何**一个参数是 NULL，那两个开关也算。
- **被拒绝的输入会让整条语句失败。** 报错信息里带函数名，查询剩下的部分不会再执行。
- **常量会被折叠成一次执行。** `SELECT op_open_path('report.html')` 只跑一次。参数一旦来自某一列，调用就
  按行跑 —— `SELECT op_open_path(p) FROM reports` 一行开一个窗口，不去重。按行跑 `op_write` 就是一行一个
  文件，这正是那套生成名字要应付的。
- **列表按类型打印。** `op_open_commands` 返回 `VARCHAR[]`，所以里面的每一项都保留它被启动时那副引用样子；
  要一条命令一行就取下标（`op_open_commands('report.html')[1]`）或者 `unnest`。
- **文件名那一族在每台机器上答得一样。** `op_filename` 与 `op_filename_check` 里没有任何东西取决于
  文件系统、桌面，或者某个窗口干了什么，所以改写出来的名字可以在测试里逐字断言 —— 这也正是 `windows` 被
  钉成 `true`、而不是跟着宿主平台走的原因。
