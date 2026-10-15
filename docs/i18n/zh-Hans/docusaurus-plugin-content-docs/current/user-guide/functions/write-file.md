---
title: 把内容落成文件
sidebar_position: 3
description: op_write_file、op_open_content 与 op_open_html —— 把内容变成文件，顺便可能打开它。
---

# 把内容落成文件

这三个从**内容**出发，而不是从路径出发 —— 这正是一条查询产出的东西通常的形状。内容变成一个文件，函数
返回它写到的那个路径；从那里起它就是一条普通的路径，[打开路径](./open-path.md)那三个接得住。

因为路径就是返回值，失败就没法写在返回值里。`op_write_file` 什么都不启动，所以它的答复说的都是文件：
一个路径、没有输入的那一行是 NULL、以及一个它没能做成文件名的参数是一句错误。`op_open_content` 与
`op_open_html` 在其上又加了一次启动，而起不来的启动器报成**一句带着路径的错误**：文件已经落在盘上，
只是这台机器没有东西能把它显示出来：

```sql
SELECT op_write_file('report body', 'md');
-- /tmp/open-20260101-120000-9Qm2Zk.md  — 写好了，什么都没启动

SELECT op_open_content(svg, 'svg');
-- 有桌面的机器：路径，外加一个正在显示它的查看器
-- 没有桌面的机器：一句错误，"the file was written to '<path>', but no launcher could be started"
```

一个做不成文件名的参数同样是错误，一个用不了的设置一句：

```sql
SELECT op_write_file('a', '');                       -- 报错：op_write_file: the extension must not be empty …
SELECT op_write_file('a', 'txt', {'dir':'  ', 'name':NULL, 'part':NULL});
                                                  -- 报错：… op_write_file_options.dir must not be empty
SELECT op_open_content('a', 'txt', {'dir':'./out', 'name':'/', 'part':NULL});
                                                  -- 报错：… op_write_file_options.name is not a usable file name
```

## op_write_file

```text
op_write_file(content VARCHAR, extension VARCHAR) -> VARCHAR
op_write_file(content VARCHAR, extension VARCHAR, opts op_write_file_options) -> VARCHAR
op_write_file(content BLOB,    extension VARCHAR) -> VARCHAR
op_write_file(content BLOB,    extension VARCHAR, opts op_write_file_options) -> VARCHAR
```

把一段内容变成一个文件，返回它写到的那个路径。它不起任何进程，这正是它能拼起来的原因：拿着这个路径去
[`op_open_path`](./open-path.md#op_open_path) 就能挑程序，去 [`op_open_content`](#op_open_content) 就用
默认的那个。

```sql
SELECT op_write_file('{"a":1}', 'json');
SELECT op_write_file(svg, 'svg', {'dir':'./out', 'name':'chart', 'part':NULL});
SELECT op_write_file(read_blob('logo.png'), 'png');
```

**扩展名是唯一必填的设置**，因为系统就是照它决定怎么对待这个文件：`html` 与 `.html` 都指 `.html`；空的
扩展名 —— 或者带路径分隔符的（那其实是一个伪装成扩展名的路径）—— 是一句错误，而不是一次猜测。对 BLOB
它更要紧：没有后缀，查看器根本不知道自己看到的是什么。

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

### op_write_file_options

`op_write_file_options` 是扩展注册的一个 STRUCT 类型，那几个带选项的签名就是拿它当第三个参数：

```text
op_write_file_options(dir VARCHAR, "name" VARCHAR, part VARCHAR)
```

这个字段显示成 `"name"`，因为 NAME 是 DuckDB 语法里的关键字；结构体字面量里写 `name:` 或 `'name':`
都能用。

**三个字段都可省，但省掉其中任何一个的字面量都需要一次转换。** DuckDB 不会把 `{'dir':'./out'}` 隐式
加宽成三字段的 STRUCT，所以要么每个字段都写，要么带上转换：

```sql
SELECT op_write_file(svg, 'svg', {'dir':'./out', 'name':NULL, 'part':'sales'});
SELECT op_write_file(svg, 'svg', {'dir':'./out'}::op_write_file_options);
```

整个 STRUCT 作为 NULL 传进来就是一个 NULL 参数，所以它会像内容为 NULL 那样短路 —— 它不是「用默认值」。
想要默认值就别写第三个参数。

## op_open_content

```text
op_open_content(content VARCHAR, extension VARCHAR) -> VARCHAR
op_open_content(content VARCHAR, extension VARCHAR, opts op_write_file_options) -> VARCHAR
op_open_content(content BLOB,    extension VARCHAR) -> VARCHAR
op_open_content(content BLOB,    extension VARCHAR, opts op_write_file_options) -> VARCHAR
```

[`op_write_file`](#op_write_file) 加一次启动，一次调用做完，这是探索过程中最常见的那件事：查询手里有一段
内容，你想看一眼。

```sql
SELECT op_open_content(read_text('report.html'), 'html');
SELECT op_open_content(svg, 'svg', {'dir':'./out', 'name':NULL, 'part':'sales'});
SELECT op_open_content(logo_blob, 'png');
```

上面那套落盘规则，它和 `op_write_file` 用的是同一条。它加上的那次启动带来两个行为：它不等程序退出；
而起不来的启动器是一句带着路径的错误，不是 [`op_open_path`](./open-path.md#op_open_path) 会答的那个
`false`。

挑哪个程序，有意不归这个函数管；选项 STRUCT 里没有 `app` 字段，理由也是这个。要指定程序就把调用拼出来：

```sql
SELECT op_open_path(op_write_file(html, 'html'), 'firefox');
```

## op_open_html

```text
op_open_html(content VARCHAR) -> VARCHAR
```

[`op_open_content`](#op_open_content)，三个选项全都留在默认值上、扩展名钉死成 `html`：一个临时文件，在
系统默认浏览器里打开，返回路径。

```sql
SELECT op_open_html('<h1>hello</h1>');
SELECT op_open_html(string_agg(row, chr(10))) FROM rendered_rows t(row);
```

它之所以存在，是因为 SQL 渲染出来的东西十有八九是 HTML，而每次都要敲上扩展名再加一个空 STRUCT 是噪音。
当你需要一个目录、一个名字、名字里一段认得出来的字，或者默认浏览器以外的浏览器，那就是
`op_open_content(content, 'html', opts)`，或者上面那个 `op_open_path` 组合。这里没有 BLOB 签名：它的输入
是 HTML *文本*。
