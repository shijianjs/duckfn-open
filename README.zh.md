[English](README.md) | [简体中文](README.zh.md)

# duckfn_open

用 [duckfn](https://crates.io/crates/duckfn) 写的 DuckDB [可加载扩展](https://duckdb.org/docs/stable/extensions/extension_development)：
属性宏把普通 Rust 函数变成 DuckDB 的标量函数，全程不碰 C++ 构建（C API 只用头文件，走 DuckDB 的 API 函数表分发）。

它给 SQL 加的是两步，两步都为了「把东西送到人眼前」：[open](https://crates.io/crates/open) 这一件事 ——
把一个路径或 URL 交给桌面当前配置好的程序；以及它前面的那一步，因为查询手里躺着的大多是**内容**
（渲染好的页面、一张图、一份 CSV、BLOB 里的字节），而启动器只认路径：

```sql
SELECT open_path('/var/tmp/report.html');           -- 已经存在的路径
SELECT open_html('<h1>hello</h1>');                 -- 内容：临时 .html，进浏览器
SELECT open_write(csv, 'csv');                      -- 内容：落成文件，回路径，不起进程
SELECT open_print(chart);                           -- 内容：直接进 stdout，绕开结果集的排版
```

两半都面向在 DuckDB 会话里探索，不是把启动器嵌进生产程序；内容这一半是把那段落盘的活儿只写一遍，
做成通用的「内容 → 一个能看它的地方」，而不是每个扩展各抄一份。

本仓库来自 [duckfn-extension-template](https://github.com/shijianjs/duckfn-extension-template)，
它带来整套可运行的工作循环——构建、sqllogictest、文档导出、发版。同样写法的另一个扩展：
[duckfn_quantstats](https://github.com/shijianjs/duckfn-quantstats)。

## 快速开始

```shell
make configure   # 一次即可：建 configure/venv（Python + sqllogictest 运行器）
make debug       # -> build/debug/duckfn_open.duckdb_extension
```

本地构建的扩展没有签名，所以 DuckDB 要带 `-unsigned` 启动：

```shell
duckdb -unsigned -c "
LOAD './build/debug/duckfn_open.duckdb_extension';
SELECT open_commands('report.html');
-- 这台机器上会依次尝试的启动命令，一条都不执行
"
```

有三个名字不起任何进程，所以在任何机器上都能答，也因此最适合先试：`open_commands` 回答「在这里到底会
拿什么去打开」，`open_write` 只写一个文件、把路径回给你，`open_print` 只往 stdout 写。

`Justfile` 包住同一批命令：`just build`、`just sql "SELECT open_commands('report.html')"`、
`just repl`（扩展已经 LOAD 好的 REPL）。

## 函数

七个 SQL 名字、十六个签名，全是标量函数 —— 没有聚合函数、没有表函数 —— 另外注册一个 STRUCT 类型
`open_write_options(dir VARCHAR, "name" VARCHAR, part VARCHAR)`，落盘这一族的三个可选项就装在它里面。
七个名字里有五个在同一个 SQL 名字下带多个签名（靠 duckfn 的 `overloads_name` 并成一个函数集），所以
调用点写的是 `open_write(html, 'html', opts)` 与 `open_path('a', 'firefox')`，不会变成
`open_write_with(…)` 或 `open_path_with(…)`。实现在 `src/extension/functions/`，落盘与命名规则集中在
`src/extension/writer.rs`，平台差异集中在 `src/extension/launcher.rs`。

第一组接的是一个已经存在的路径或 URL，把它交给桌面：

| 函数 | 输入 → 输出 | 对应的 open 调用 |
| --- | --- | --- |
| `open_path(path)` | `VARCHAR` → `BOOLEAN` | `open::that` |
| `open_path(path, app)` | `VARCHAR, VARCHAR` → `BOOLEAN` | `open::with` |
| `open_path_detached(path)` | `VARCHAR` → `BOOLEAN` | `open::that_detached` |
| `open_path_detached(path, app)` | `VARCHAR, VARCHAR` → `BOOLEAN` | `open::with_detached` |
| `open_commands(path)` | `VARCHAR` → `VARCHAR[]` | `open::commands` |

第二组从**内容**出发，那才是一次查询产出的东西的形状：

| 函数 | 签名 | 输出 |
| --- | --- | --- |
| `open_write` | `(content, ext)`、`(content, ext, opts)`，`BLOB` 同样两个形式 | `VARCHAR` —— 写好的路径；不起任何进程 |
| `open_content` | 同样的四个 | `VARCHAR` —— 那个路径，此时默认程序已经把它显示出来了 |
| `open_html` | `(content)` | `VARCHAR` —— 零配置的那一种：临时 `.html` 进默认浏览器 |
| `open_print` | `(text)`、`(text, tail)` | `VARCHAR` —— 恒为 NULL；内容已经打到 stdout 了 |

`open_content` 就是 `open_write` 与 `open_path` 的合成，之所以给它一个名字，是因为「写完就打开」是探索时
最常做的事；`open_html` 则是 `open_content(content, 'html')` 把三个可选项全按缺省走 —— 因为 SQL 渲染出来的
东西十有八九是 HTML。

### 一次调用答的是什么

两组的答法不一样，而各自都有自己的约定。

**启动这一族** —— SQL 里能分辨三种结果，这就是全部约定：

- **`NULL`** —— 这一行没有输入。入参都不是 `Option`，所以 NULL 由 duckfn 的参数读取层短路成 SQL NULL，
  启动器一次都不会被叫起来（`SELECT open_path(NULL); -- NULL`）；
- **`true`** —— 启动器被拉起了。不带后缀的那一组拿得到启动器自己的返回，所以 Linux 上它意味着目标程序跑过
  并退出；Windows 上 `_detached` 那一组走一次 ShellExecuteW，它只说明进程起来了；
- **`false`** —— 一个启动器都没能起来（这台机器没有 `xdg-open`、指定的程序不存在）。open 那边的真实原因是
  一个 `io::Error`，而 SQL 里没有地方放它，所以要知道「这里到底会跑什么」请用 `open_commands`。

**落盘这一族**回的是一个路径，这改变了失败能写在哪儿：

- `open_write` 不起进程，所以它的三种答复都关于文件：一个路径、没有输入的那一行给 NULL、
  一个成不了文件名的参数给错误；
- `open_content` 与 `open_html` 在前面那一步之后再加一次启动。它们的返回值已经是那个路径了，所以启动器
  起不来时给的**不是** `open_path` 会答的那个 `false`，而是**带着路径的报错** —— 文件确实在磁盘上，
  只是这台机器没有东西能把它显示出来。这一步走的是**不等**的那条路（`open::that_detached`）：
  写完一张图再打开它，不该被「等查看器关掉」挂住；
- `open_print` 永远回 NULL：内容已经在控制台上了，SQL 这一侧没有值可给。

### 落盘这一族守的规则

`open_write` 的四个签名与 `open_content` 的四个签名共用同一层落盘代码，所以一份清单讲完两个名字：

- **后缀是必填的那一项** —— 它决定系统怎么看待这个文件。`html` 与 `.html` 是一回事；空后缀，或者带路径
  分隔符的后缀（那其实是一条路径换了个地方写）是错误，不是猜测。对 BLOB 更要紧：没有后缀，查看器根本
  不知道自己在看什么；
- **`dir`** 缺省是系统临时目录；目录不存在时连父级一起建出来 —— 会写 `dir` 的人说的是「就该放这儿」，
  不是「这儿已经在了」；
- **`name`** 原样使用（名字里没带后缀时把必填的那个补上），而且**同名的已有文件会被覆盖**：你点了名，
  这就是你要的路径；
- 没给 `name` 就自动命名成 `open-<%Y%m%d-%H%M%S>-<part>-<随机尾缀>.<后缀>`，例如
  `open-20260101-120000-Sales_Overview-9Qm2Zk.svg`。时间紧跟在前缀后面，所以一个目录按名字排序就是
  按时间排序；随机尾缀、以及「自动名字绝不会落到已有文件上」这一条都由 `tempfile` 保证；
- **`part`** 是自动名字里「这是哪一份」的那一段（图表标题、报表名），限 48 个字符 —— 不这么办的话目录里
  就只剩一串时间戳；
- `dir`、`name`、`part` 三项一律当文件名材料对待，都先过 `sanitize-filename`（非法字符、控制字符、
  Windows 保留设备名、结尾的点与空格），所以带斜杠的 `part` 逃不出它的目录，中文标题也塞得进名字；
  `part` 里的空格并成 `_`；
- 目录与文件名**一律用 `/` 拼**，三个平台都一样：这个字符串要给人读，也通常会被下游的查询接着拼，
  而 `std::fs` 在 Windows 上也认 `/`。目录那一段保持你给它的形状，所以缺省目录在 Windows 上长成
  `C:\Users\…\Temp/open-….json` 这样。

挑哪个程序刻意不在这两个函数的事上，选项 STRUCT 里也正因为这个才没有 `app` 字段：
`open_path(open_write(html, 'html'), 'firefox')` 已经写得出同一件事，而且它还留着路径，动手打开之前
先看得一眼。

`open_print` 存在的理由是结果集做不到的一件事。DuckDB 的 CLI 会把结果**排版** —— 列宽、边框、折行 ——
而字符画（`textplot` 那类扩展返回成一个 VARCHAR 的 ASCII/盲文图）靠逐行对齐来读，塞进一个单元格就全散了。
内容原样写到 stdout，每次调用都 flush，因为 Rust 的 stdout 是按行缓冲的；`tail` 取代默认追加的那个换行 ——
`''` 是「接着同一行打」，`chr(10) || chr(10)` 是「空一行再打」。

### 两个 DuckDB 的坑

- 省略了字段的 struct 字面量**不会**被隐式加宽成三字段的 STRUCT，所以要么把三个字段写全，要么加转换：
  `SELECT open_write(svg, 'svg', {'dir':'./out'}::open_write_options);`
- 整个 STRUCT 传 NULL 就是一个 NULL 参数，它和内容传 NULL 一样**短路成 NULL、什么都不写**。它不是
  「其余项按缺省」；要缺省就**省略第 3 个参数**。

### 放进循环之前

有两件事适用于每一个名字：

- 等不等是启动那一组里两个名字唯一的可见差别：`open::that` 在启动器返回时才返回，而 Linux 上 `xdg-open`
  通常要到目标程序退出才返回。Windows 上（依赖里那个 `shellexecute-on-windows` feature）
  `open_path_detached` 反而是更直接的一条路：一次原生 API 调用，而不是「PowerShell 辅助进程 +
  explorer.exe 后备」；
- 参数全是常量的调用可能被 DuckDB 折叠成一次执行，所以 `SELECT open_path('report.html')` 只开一个窗口、
  `SELECT open_print('x')` 只打一行，而 `SELECT open_path(p) FROM reports` 一行开一个 —— 不去重。
  真要每行一次就在参数里带上列引用（`open_print('x: ' || a)`）；对 `open_write` 来说「一行一个文件」正是
  自动命名要解决的问题。

空的路径、空的程序名、空的后缀、用不了的选项值都是错误，而不是一个 `false` 或者一次静默的缺省 ——
因为「写错了」和「打不开」该有不同的答复。DuckDB 报的是 invalid input error，报文最前面是函数自己的名字：

```sql
SELECT open_path('');    -- open_path: the path or URL must not be empty
SELECT open_write('a', '');
-- open_write: the extension must not be empty (pass e.g. 'html', not '')
SELECT open_write('a', 'txt', {'dir':'   ', 'name':NULL, 'part':NULL});
-- open_write: open_write_options.dir must not be empty
```

启动器是桌面软件，所以这个扩展只构建桌面平台（见下面「平台」一节）。在没有桌面的机器上，四个
`open_path` 动作一律答复 `false`，`open_content` 与 `open_html` 报一个带着它们确实写出来的路径的错误，
而 `open_commands` 照样能答 —— 它列出这台机器上会依次尝试什么。`open_write` 与 `open_print` 根本不需要
桌面。见 `src/extension/launcher.rs`。

## 从源码构建

构建路径是官方 DuckDB `extension-ci-tools` 的 makefile：

```shell
make configure           # 一次即可：建 configure/venv（Python + sqllogictest 运行器）
make debug               # -> build/debug/duckfn_open.duckdb_extension
```

`make release` 是同一套流程的优化版。Windows 上 `make` 必须在 Git Bash 里跑。`Justfile` 把它包好了
（`just build` = `make configure && make debug`，`just ci-build`、`just test`、`just ci-release`）。
首次构建之前要先 `git submodule update --init --recursive` 把 `extension-ci-tools` 拉下来。

## 测试

测试是 `test/sql/` 下的 SQLLogicTest 文件：

```shell
just test          # make configure + make debug + make test，再把测试写出来的东西清掉
just ci-build      # 只跑官方构建流程，不跑测试
```

`make test` 不会重新构建，改了 Rust 代码要先 `just ci-build`（或 `make debug`）。能不能打开取决于这台机器
的桌面配置，而 CI 的 runner 根本没有桌面，所以测试里**一次都不会去调会 launch 的东西**。真跑的是不依赖
桌面的那部分：`duckdb_functions()` 里注册的签名形状、NULL 短路、参数校验（包括那些在落盘**之前**就返回
错误的检查）、不起任何进程的 `open_commands` 预览，以及整套落盘本身 —— `open_write` 写出去的文件用
`read_text` / `read_blob` 读回来逐个验（命名、建目录、BLOB 字节原样往返、非法字符替换、同名覆盖），
`open_print` 是真的被调用、断言它答 NULL。`open_content` 与 `open_html` 走的是同一个 `writer::write`，
所以它们的测试留在 catalog 一侧，只跑那些到不了 launch 那一步的调用。

这些测试会往 `test/sql/`（运行器的 `__TEST_DIR__`）里写产物，所以 `.gitignore` 在那儿只留 `*.test`，
而 `just test` 末尾会再跑一次 `git clean -fdX -- test/sql`。开发指南的
[测试](docs/docs/development/testing.md)一页写了迭代循环（直接对着 `build/debug/*.duckdb_extension`
跑运行器）以及每个测试文件覆盖到什么。

## 平台

只有 Windows、Linux 与 macOS。启动器是桌面软件，所以这个扩展不构建 WebAssembly 目标：CI 用
`.github/workflows/MainDistributionPipeline.yml` 里的 `exclude_archs` 排掉三条 wasm 架构，而
`just build_wasm` / `build_wasm_eh` / `test_wasm` 在本项目里被覆盖成一句说明，不再构建任何东西。
在没有桌面的机器上，四个 `open_path` 动作一律答复 `false`，`open_content` / `open_html` 报一个带着路径的
错误，而 `open_commands`、`open_write`、`open_print` 照常工作。

## 文档站

仓库在 `docs/` 里带了一份 [Docusaurus](https://docusaurus.io/) 站点（英文 + 简体中文），并由一个工作流在每个版本
tag 上把它发布到 GitHub Pages：

```shell
just docs_install    # 一次即可
just docs_start      # 开发服务器 http://localhost:3000
just docs_build      # 真正要紧的检查：onBrokenLinks 设成了 throw
```

页面里的 SQL 都是静态代码块 —— 要让读者在浏览器里就地跑，得有本扩展的 wasm 构建，而它一个都不产出。
约定（布局、命令、翻译流程、部署、`{{EXTENSION_VERSION}}` 版本占位符，以及静态块要额外交代清楚什么）
分别在 [`docs/README.md`](docs/README.md) 与 [`docs/AGENTS.md`](docs/AGENTS.md)。

## 安装已发布的扩展

Release 就是 GitHub Release，上面挂着各平台构建出的 `.duckdb_extension`：

```shell
duckdb -unsigned -c "
LOAD 'https://github.com/shijianjs/duckfn-open/releases/latest/download/duckfn_open-windows_amd64.duckdb_extension';
"
```

进了 DuckDB 的[社区扩展](https://duckdb.org/community_extensions/list_of_extensions)之后就是
`INSTALL duckfn_open FROM community`；它需要的两个文件准备在 [`community-extension/`](community-extension/AGENTS.md)。

## 相关文档

| 文件 | 内容 |
| --- | --- |
| [AGENTS.md](AGENTS.md) | 仓库约定、duckfn 知识地图、发版流程 |
| [docs/docs/development/](docs/docs/development/quick-start.md) | 开发指南：目录结构、写函数与测试、构建与发版（中英各一份） |
| [docs/README.md](docs/README.md) | 文档站：布局、命令、翻译、部署 |
| [docs/docs/user-guide/functions.md](docs/docs/user-guide/functions.md) | 逐个名字的签名与示例（最详尽的一份） |
| [README.md](README.md) | this file, in English |
