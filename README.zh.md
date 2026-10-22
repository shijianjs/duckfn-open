[English](README.md) | [简体中文](README.zh.md)

# duckfn_open

[![DuckDB 1.4+](https://img.shields.io/badge/DuckDB-1.4%2B-fff100?logo=duckdb&style=flat)](https://duckdb.org/)
[![周下载](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fcommunity-extensions.duckdb.org%2Fdownloads-last-week.json&query=%24.duckfn_open&label=%E5%91%A8%E4%B8%8B%E8%BD%BD&color=blue&logo=duckdb&logoColor=a899ff&style=flat)](https://duckdb.org/community_extensions/extensions/duckfn_open)
[![版本](https://img.shields.io/github/v/release/shijianjs/duckfn-open?style=flat&logo=github&label=%E7%89%88%E6%9C%AC)](https://github.com/shijianjs/duckfn-open/releases)
[![许可证](https://img.shields.io/badge/%E8%AE%B8%E5%8F%AF%E8%AF%81-MIT-14459b.svg?style=flat&logo=github)](https://github.com/shijianjs/duckfn-open/blob/main/LICENSE)

`duckfn_open` 把两个成熟的 Rust 库接到 DuckDB SQL 上：[open](https://crates.io/crates/open) 负责把一个路径
或 URL 交给桌面当前配置好的程序，[sanitize-filename](https://crates.io/crates/sanitize-filename) 负责在一个
字符串上了结「它能不能当文件名」。两者都不是重写的 —— 各平台的启动方式与参数引用、文件名合法性的那套规则
都是库自己的，这里只是逐个接口接到 SQL 上。

它给 SQL 加的是两步，两步都为了「把东西送到人眼前」：open 那一件事，以及它前面的那一步，因为查询手里躺着的
大多是**内容**（渲染好的页面、一张图、一份 CSV、BLOB 里的字节），而启动器只认路径：

```sql
SELECT op_open_path('/var/tmp/report.html');           -- 已经存在的路径
SELECT op_open_html('<h1>hello</h1>');                 -- 内容：临时 .html，进浏览器
SELECT op_write_file(csv, 'csv');                      -- 内容：落成文件，回路径，不起进程
SELECT op_print(chart);                                -- 内容：直接进 stdout，绕开结果集的排版
```

还有第三族站在这两步之外：`op_sanitize_filename` 与 `op_sanitize_filename_check` 把一个字符串改写成能当
文件名用的样子，或者回答它本来是不是。不起进程、不碰文件、连路径都不要。

它的方向是**交互式探索**：人坐在那台机器前，跑着 DuckDB，想让结果出现在看得见的地方。它调的启动器就是
那台机器上配好的那个，这也正是它不适合被嵌进一个应用里的原因 —— DuckDB 无头跑的地方（服务器、定时任务、
CI runner）没有桌面可以把文件交出去，而一次库调用弹出别人机器上的窗口，不是调用方想要的东西。两半里，
不起进程的那一半（`op_write_file`、`op_print` 与两个 `op_sanitize_filename` 名字）才是那边还讲得通的。

## 安装

`duckfn_open` 需要 **DuckDB 1.4 或更新**，以及一台有桌面的机器 —— Windows、Linux 或 macOS。它就是一个
普通的可加载扩展：

```sql
INSTALL duckfn_open FROM community;   -- 装一次
LOAD duckfn_open;                     -- 每个会话各一次
```

GitHub Release 上的文件或本地自己构建的产物也行，这两条路都要给 DuckDB 加 `-unsigned`；
[安装](https://shijianjs.github.io/duckfn-open/docs/user-guide/installation)一页写了这三种方式。

有五个名字不起任何进程，所以在任何机器上都能答，也因此最适合先试：`op_open_commands` 回答「在这里到底会
拿什么去打开」，`op_write_file` 只写一个文件、把路径回给你，`op_print` 只往 stdout 写，而两个
`op_sanitize_filename` 名字连桌面与文件都不要 —— 进一个字符串，出一个可用的名字（或者一个是/否）。

```sql
SELECT op_open_commands('report.html');
-- 这台机器上会依次尝试的启动命令，一条都不执行
```

## 函数

九个 SQL 名字、二十一个签名，全是标量函数 —— 没有聚合函数、没有表函数 —— 另外注册一个 STRUCT 类型
`op_write_file_options(dir VARCHAR, "name" VARCHAR, part VARCHAR)`，落盘这一族的三个可选项就装在它里面。
九个名字里有七个在同一个 SQL 名字下带多个签名，所以调用点写的是 `op_write_file(html, 'html', opts)` 与
`op_open_path('a', 'firefox')`，不会变成 `op_write_file_with(…)` 或 `op_open_path_with(…)`。

第一组接的是一个已经存在的路径或 URL，把它交给桌面：

| 函数 | 输入 → 输出 | 对应的 open 调用 |
| --- | --- | --- |
| `op_open_path(path)` | `VARCHAR` → `BOOLEAN` | `open::that` |
| `op_open_path(path, app)` | `VARCHAR, VARCHAR` → `BOOLEAN` | `open::with` |
| `op_open_path_detached(path)` | `VARCHAR` → `BOOLEAN` | `open::that_detached` |
| `op_open_path_detached(path, app)` | `VARCHAR, VARCHAR` → `BOOLEAN` | `open::with_detached` |
| `op_open_commands(path)` | `VARCHAR` → `VARCHAR[]` | `open::commands` |

第二组从**内容**出发，那才是一次查询产出的东西的形状：

| 函数 | 签名 | 输出 |
| --- | --- | --- |
| `op_write_file` | `(content, ext)`、`(content, ext, opts)`，`BLOB` 同样两个形式 | `VARCHAR` —— 写好的路径；不起任何进程 |
| `op_open_content` | 同样的四个 | `VARCHAR` —— 那个路径，此时默认程序已经把它显示出来了 |
| `op_open_html` | `(content)` | `VARCHAR` —— 零配置的那一种：临时 `.html` 进默认浏览器 |
| `op_print` | `(text)`、`(text, tail)` | `VARCHAR` —— 递进去的那段文本；顺手打到了 stdout |

第三族是**纯字符串**，与上面两组都不相干：它里面没有任何一样东西会起进程、会碰文件。

| 函数 | 签名 | 输出 |
| --- | --- | --- |
| `op_sanitize_filename` | `(value)`、`(value, replacement)`、`(value, replacement, truncate, windows)` | `VARCHAR` —— 把同一个字符串改写成能当文件名用的样子 |
| `op_sanitize_filename_check` | `(value)`、`(value, truncate, windows)` | `BOOLEAN` —— 那一次改写会不会什么都不动 |

### 一次调用答的是什么

- **启动这一族** —— 这一行没有输入时是 `NULL`，启动器被拉起时是 `true`，这台机器上一个都起不来时是
  `false`。空的路径或程序名是错误而不是 `false`，因为「写错了」和「打不开」该有不同的答复；
- **落盘这一族** —— 回它写好的那个路径；一个成不了文件名的参数则报错。`op_open_content` 与
  `op_open_html` 在前面那一步之后再加一次启动，启动器起不来时给的**是带着路径的报错**：文件确实在磁盘上，
  只是这台机器没有东西能把它显示出来；
- **文件名这一族** —— 回一个字符串或一个 `BOOLEAN`，NULL 进 NULL 出，而「值不对」在这里不是错误：
  一个什么都留不下的名字照样有答复。

凡是会碰外面世界的名字都注册成 **volatile**，所以一次调用就是一行：常量参数和列参数的表现完全一样，
规划一条查询时也不会执行到它们（`EXPLAIN SELECT op_write_file('a', 'txt')` 一个文件都不写）。

这些答复背后的规则 —— 那个选项 STRUCT、自动名字是怎么拼的、文件名这个库把什么算作非法，以及唯一一处
不跟库走而钉住的缺省 —— 都在[函数参考](https://shijianjs.github.io/duckfn-open/docs/category/functions)里。

## 文档

| 位置 | 内容 |
| --- | --- |
| [用户指南](https://shijianjs.github.io/duckfn-open/docs/intro) | 它给 DuckDB 加了什么、怎么装、每个函数的签名与示例 |
| [开发指南](docs/docs/development/quick-start.md) | 构建、测试与发布这个扩展；目录结构（也有[线上版](https://shijianjs.github.io/duckfn-open/docs/development/quick-start)） |
| [AGENTS.md](AGENTS.md) | 仓库约定、duckfn 知识地图与发版流程 |
| [README.md](README.md) | 本文件，英文 |
