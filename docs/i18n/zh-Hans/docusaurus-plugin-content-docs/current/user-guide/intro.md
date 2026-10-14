---
title: 简介
sidebar_position: 1
slug: /intro
description: duckfn_open 给 DuckDB 加了什么，以及怎么开始用。
---

# 简介

`duckfn_open` 是一个 [DuckDB 可加载扩展](https://duckdb.org/docs/stable/extensions/overview)：它给 DuckDB
加了一批 SQL 函数，加载之后它们和 DuckDB 自带的函数没有区别。不用编译什么，调用处也不用 import ——
CLI、Python / R 会话与其他 DuckDB 前端接受的都同一份 `.duckdb_extension` 文件。

它加的东西一句话就能说完：把一个路径或 URL 交给桌面当前配置好的程序去打开，让一条查询能把报表送到人
眼前。可查询产出的东西大多还没有路径 —— 它是一段内容 —— 所以这套 API 的另一半先把内容写成一个文件，
再打开那个文件。还有第三块这两边都不属于：它就在一个普通字符串上了结一件事 —— 这个字符串到底能不能当
一个文件名。

加载之后注册到位的是九个函数名、二十一个签名，全都是标量函数，另外还有一个装落盘选项的 STRUCT 类型。
分成三组：

- **路径与 URL** —— [打开路径与 URL](./functions/open-path.md)：`op_open_path` 把文件或链接交给桌面配好
  的那个程序，并等它返回；`op_open_path_detached` 不等；`op_open_commands` 只列出启动会依次尝试什么。
- **内容** —— [把内容落成文件](./functions/write-file.md)与[打印到 stdout](./functions/print.md)：
  `op_write_file` 把文本或字节变成一个文件并返回路径，`op_open_content` 在同一次调用里顺手打开，
  `op_open_html` 连选项都不用写，而 `op_print` 干脆绕过结果集的排版直接写 stdout，再把那段文本还回去。
- **文件名** —— [从数据造文件名](./functions/sanitize-filename.md)：`op_sanitize_filename` 把一个值改写成
  各平台都认的名字，`op_sanitize_filename_check` 只回答它本来是不是这样一个名字。两个都一步都不离开递给
  它们的那个字符串。

## 它是给谁用的

这是个做**交互式探索**的工具：人坐在那台机器前，跑着 DuckDB，想让结果出现在眼前。它调的启动器就是那台
机器上配好的那个，所以它属于「有人在看着输出」的那些地方 —— CLI、notebook、有人在敲的 Python / R 会话。

DuckDB 被别人嵌进应用里跑的场合，**不是**它要解决的问题。服务器、定时任务、CI runner 里没有桌面可以把文件
交出去，而一次库调用弹出别人机器上的窗口，也不是调用方想要的：在那里启动那一族一律答复 `false`，
`op_open_content` 与 `op_open_html` 则报一句带着它们确实写出的那个路径的错误。真要在那种机器上用这个扩展，
用的是不起进程的那一半 —— `op_write_file`、`op_print` 与文件名那两个函数。

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

这个站不会替你跑任何东西：每个例子都是给已经加载了扩展的 DuckDB 用的普通 SQL。下面这条查询只读目录、
不做任何动作，所以它最适合先试。`op_write_file`、`op_print` 和文件名那两个函数也不需要桌面；启动那件事
才需要：在没有桌面的机器上（服务器、CI runner）四个 `op_open_path` 动作一律答复 `false`，而
`op_open_content` 和 `op_open_html` 会报错，报文里带着它们确实已经写出去的那个路径。每个答复意味着什么，
都写在[函数](./functions/index.md)那一页里。

```sql
SELECT function_name, len(parameters), return_type
FROM duckdb_functions()
WHERE function_name LIKE 'op%'
ORDER BY 1, 2;    -- 九个名字，二十一个签名
```

其他获取文件的方式见[安装](./installation.md)。

::::note[关于副作用]

二十一个签名里有九个会起进程，所以把它们放进循环之前值得先问两句：这会开出多少个窗口，以及这条查询跑在
谁的机器上。这些调用**一律按行执行，一次不多一次不少**：DuckDB 既不折叠也不缓存它们，所以「参数是常量」
救不了你 —— `SELECT op_open_path('report.html')` 开一个窗口，`SELECT op_open_path(p) FROM reports` 一行
开一个，而 `SELECT op_open_path('report.html') FROM reports` 跟后一种一样。反过来也值钱：既然什么都不被
折叠，规划一条查询时也就什么都不会发生 —— `EXPLAIN SELECT op_write_file('a', 'txt')` 一个文件都不写。

其余的都不起进程：`op_open_commands` 只列出启动会依次尝试的那些命令，`op_write_file` 落完盘就停 —— 这正是
让你可以自己把路径交给 `op_open_path` 的原因 —— `op_print` 只往 stdout 打，而文件名那两个函数一步都离不开
递给它们的那个字符串。

::::

## 接下来去哪

- [安装](./installation.md) —— 社区仓、Release 文件，或者本地自己构建的产物。
- [函数](./functions/index.md) —— 每个签名、那个选项 STRUCT、每个返回值是什么意思，以及什么都没打开时该
  查什么。
- [开发指南](../development/quick-start.md) —— 从源码构建这个扩展。那是给改这个仓库的人看的；用扩展
  本身不需要它。
