[English](README.md) | [简体中文](README.zh.md)

# duckfn_open

用 [duckfn](https://crates.io/crates/duckfn) 写的 DuckDB [可加载扩展](https://duckdb.org/docs/stable/extensions/extension_development)：
属性宏把普通 Rust 函数变成 DuckDB 的标量函数，全程不碰 C++ 构建（C API 只用头文件，走 DuckDB 的 API 函数表分发）。

它给 SQL 加的是两步，两步都为了「把东西送到人眼前」：[open](https://crates.io/crates/open) 这一件事 ——
把一个路径或 URL 交给桌面当前配置好的程序；以及它前面的那一步，因为查询手里躺着的大多是**内容**
（渲染好的页面、一张图、一份 CSV、BLOB 里的字节），而启动器只认路径：

```sql
SELECT op_open_path('/var/tmp/report.html');           -- 已经存在的路径
SELECT op_open_html('<h1>hello</h1>');                 -- 内容：临时 .html，进浏览器
SELECT op_write_file(csv, 'csv');                      -- 内容：落成文件，回路径，不起进程
SELECT op_print(chart);                           -- 内容：直接进 stdout，绕开结果集的排版
```

它的方向是**交互式探索**：人坐在那台机器前，跑着 DuckDB，想让结果出现在看得见的地方。它调的启动器就是
那台机器上配好的那个，这也正是它不适合被嵌进一个应用里的原因 —— DuckDB 无头跑的地方（服务器、定时任务、
CI runner）没有桌面可以把文件交出去，而一次库调用弹出别人机器上的窗口，不是调用方想要的东西。两半里，
不起进程的那一半（`op_write_file`、`op_print` 与两个 `op_sanitize_filename` 名字）才是那边还讲得通的。

内容那一半是把那段落盘的活儿只写一遍，做成通用的「内容 → 一个能看它的地方」，而不是每个扩展各抄一份。

还有第三族站在这两步之外，它什么都不往人眼前送：`op_sanitize_filename` 与 `op_sanitize_filename_check` 把一个字符串
改写成能当文件名用的样子，或者回答它本来是不是。不起进程、不碰文件、连路径都不要 —— 这两个名字把
[sanitize-filename](https://crates.io/crates/sanitize-filename) 这个库自己的规则与缺省原样接到 SQL 上。

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
SELECT op_open_commands('report.html');
-- 这台机器上会依次尝试的启动命令，一条都不执行
"
```

有五个名字不起任何进程，所以在任何机器上都能答，也因此最适合先试：`op_open_commands` 回答「在这里到底会
拿什么去打开」，`op_write_file` 只写一个文件、把路径回给你，`op_print` 只往 stdout 写，而两个
`op_sanitize_filename` 名字连桌面与文件都不要 —— 进一个字符串，出一个可用的名字（或者一个是/否）。

`Justfile` 包住同一批命令：`just build`、`just sql "SELECT op_open_commands('report.html')"`、
`just repl`（扩展已经 LOAD 好的 REPL）。

## 函数

九个 SQL 名字、二十一个签名，全是标量函数 —— 没有聚合函数、没有表函数 —— 另外注册一个 STRUCT 类型
`op_write_file_options(dir VARCHAR, "name" VARCHAR, part VARCHAR)`，落盘这一族的三个可选项就装在它里面。
九个名字里有七个在同一个 SQL 名字下带多个签名（靠 duckfn 的 `overloads_name` 并成一个函数集），所以
调用点写的是 `op_write_file(html, 'html', opts)` 与 `op_open_path('a', 'firefox')`，不会变成
`op_write_file_with(…)` 或 `op_open_path_with(…)`。实现在 `src/extension/functions/`，落盘与命名规则集中在
`src/extension/writer.rs`，平台差异集中在 `src/extension/launcher.rs`。

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

`op_open_content` 就是 `op_write_file` 与 `op_open_path` 的合成，之所以给它一个名字，是因为「写完就打开」是探索时
最常做的事；`op_open_html` 则是 `op_open_content(content, 'html')` 把三个可选项全按缺省走 —— 因为 SQL 渲染出来的
东西十有八九是 HTML。

第三族是**纯字符串**，与上面两组都不相干：它里面没有任何一样东西会起进程、会碰文件。

| 函数 | 签名 | 输出 |
| --- | --- | --- |
| `op_sanitize_filename` | `(value)`、`(value, replacement)`、`(value, replacement, truncate, windows)` | `VARCHAR` —— 把同一个字符串改写成能当文件名用的样子 |
| `op_sanitize_filename_check` | `(value)`、`(value, truncate, windows)` | `BOOLEAN` —— 那一次改写会不会什么都不动 |

查询里的文件名常常来自数据本身 —— 客户名、报表标题、抓回来的那一列 `path` —— 而它得先**成为一个名字**，
才谈得上往后拼目录、拼 URL、交给下一个扩展。`op_write_file` 内部也确实把 `dir`、`name`、`part` 各洗一遍，
但这两个名字**不是落盘那一族的助手**：它们是那个库的完整能力原样搬到 SQL 上，用库自己的规则与缺省，
回答的时候连一个要写的文件都没有。

### 一次调用答的是什么

这几组的答法不一样，而每一组都有自己的约定。

**启动这一族** —— SQL 里能分辨三种结果，这就是全部约定：

- **`NULL`** —— 这一行没有输入。入参都不是 `Option`，所以 NULL 由 duckfn 的参数读取层短路成 SQL NULL，
  启动器一次都不会被叫起来（`SELECT op_open_path(NULL); -- NULL`）；
- **`true`** —— 启动器被拉起了。不带后缀的那一组拿得到启动器自己的返回，所以 Linux 上它意味着目标程序跑过
  并退出；Windows 上 `_detached` 那一组走一次 ShellExecuteW，它只说明进程起来了；
- **`false`** —— 一个启动器都没能起来（这台机器没有 `xdg-open`、指定的程序不存在）。open 那边的真实原因是
  一个 `io::Error`，而 SQL 里没有地方放它，所以要知道「这里到底会跑什么」请用 `op_open_commands`。

**落盘这一族**回的是一个路径，这改变了失败能写在哪儿：

- `op_write_file` 不起进程，所以它的三种答复都关于文件：一个路径、没有输入的那一行给 NULL、
  一个成不了文件名的参数给错误；
- `op_open_content` 与 `op_open_html` 在前面那一步之后再加一次启动。它们的返回值已经是那个路径了，所以启动器
  起不来时给的**不是** `op_open_path` 会答的那个 `false`，而是**带着路径的报错** —— 文件确实在磁盘上，
  只是这台机器没有东西能把它显示出来。这一步走的是**不等**的那条路（`open::that_detached`）：
  写完一张图再打开它，不该被「等查看器关掉」挂住；
- `op_print` 回的是递进去的那段文本：它原样上了控制台，而值本身也留在了 SQL 这一侧。

**文件名这一族**回的是一个字符串或一个 BOOLEAN，再没有别的：任何一格参数是 NULL 就整行 NULL ——
这一族没有缺省值可猜 —— 也没有「值不对」这种报错，因为这里没有哪种值是错的。什么都留不下的名字照样
有答复，而那个答复可能就是空串（`SELECT op_sanitize_filename('CON.txt'); -- ''`）。会被拦下来的只有 binder
那一层：个数不对、类型不对。

### 落盘这一族守的规则

`op_write_file` 的四个签名与 `op_open_content` 的四个签名共用同一层落盘代码，所以一份清单讲完两个名字：

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
`op_open_path(op_write_file(html, 'html'), 'firefox')` 已经写得出同一件事，而且它还留着路径，动手打开之前
先看得一眼。

`op_print` 存在的理由是结果集做不到的一件事。DuckDB 的 CLI 会把结果**排版** —— 列宽、边框、折行 ——
而字符画（`textplot` 那类扩展返回成一个 VARCHAR 的 ASCII/盲文图，或者一个二维码）靠逐行对齐来读，塞进一个
单元格就全散了。内容原样写到 stdout，每次调用都 flush，因为 Rust 的 stdout 是按行缓冲的；`tail` 取代默认
追加的那个换行 —— `''` 是「接着同一行打」，`chr(10) || chr(10)` 是「空一行再打」。回来的就是同一段文本，
所以这个调用可以留在投影里；值本身就是那幅图时（二维码），结果集里那一份折一下就行 ——
`SELECT length(op_print(tp_qr(url)))` 打出那个码，回一个数字。

### 文件名这一族守的规则

那个库管六件事，`op_sanitize_filename` 六件全上：非法字符 `/ \ ? < > : * | "`、控制字符、整段只是点的名字
（`.` 与 `..`）、Windows 的保留设备名（`CON`、`PRN`、`AUX`、`NUL`、`COM0`–`COM9`、`LPT0`–`LPT9`，
带着后缀也算）、结尾的点与空格，以及 **255 字节**这一刀。

```sql
SELECT op_sanitize_filename('quarterly/report:v2.csv');                    -- 'quarterlyreportv2.csv'
SELECT op_sanitize_filename('  the report.  ');                            -- '  the report'：结尾的点与空格去掉，开头的留着
SELECT op_sanitize_filename('CON.txt');                                    -- ''
SELECT op_sanitize_filename('..');                                         -- ''
SELECT op_sanitize_filename('a/b:c?.txt', '_');                            -- 'a_b_c_.txt'
SELECT op_sanitize_filename('sales|east', '-');                            -- 'sales-east'
SELECT op_sanitize_filename('CON.txt', '_', true, false);                  -- 'CON.txt'：Unix 的规则里没有保留设备名
SELECT length(op_sanitize_filename(repeat('a', 300)));                     -- 255
SELECT length(op_sanitize_filename(repeat('a', 300), '_', false, true));   -- 300
```

缺省跟着库走，**只有一处是刻意不跟的**：

- `replacement` 缺省是 `''` —— 非法字符被**删掉**，不是被替换。想看它在哪里动过手就把替换符给上：`'_'`、
  `'-'`、一整段也可以（`op_sanitize_filename('a/b', '__')` → `'a__b'`），或者再回到 `''`；
- `truncate` 缺省是 `true`，255 字节那一刀是开着的；
- `windows` **钉成 `true`**，跟宿主平台无关。库里它的缺省是编译平台，那会让同一条 SQL 在三台机器上给出
  三个答案；这里跑的是最严的那一套，三个平台同形，要 Unix 那套规则就明写
  `op_sanitize_filename(x, replacement, truncate, false)`。

两个 BOOLEAN 只能一起给，也就是四个参数的那一条 —— **没有三个参数的写法**。这是这份 API 里唯一的一处
压缩，不是漏配：两个开关各自摆在可选位置上就是五种组合，而其中只有库的那两种缺省值得一个更短的形式。
判断那一条也是同一个形状，那两个开关在那里是第 2 与第 3 个参数。

`op_sanitize_filename_check` 是同一个库的判断那一半，它什么都不改写：`true` 的意思是在同样的开关下
`op_sanitize_filename` 会把拿到的那个值原样回给你。体检和过滤要的就是这一句 —— 先看看会有多少条被动到，
再决定要不要动它们：

```sql
SELECT op_sanitize_filename_check('report.csv');    -- true
SELECT op_sanitize_filename_check('a/b.txt');       -- false
SELECT op_sanitize_filename_check('CON.txt');       -- false
SELECT op_sanitize_filename_check('trail. ');       -- false
SELECT count(*) FROM files WHERE NOT op_sanitize_filename_check(name);
```

这里没有 `replacement`：一个都不改写的问题，没有「换成什么」这一项。

至于**跑在 `op_write_file` 里面**的那条命名规则（`part` 里的空格并成 `_`、最多 48 个字符）是另一套策略，
它刻意没有自己的 SQL 名字。别把一族的缺省读成另一族的。

### 两个 DuckDB 的坑

- 省略了字段的 struct 字面量**不会**被隐式加宽成三字段的 STRUCT，所以要么把三个字段写全，要么加转换：
  `SELECT op_write_file(svg, 'svg', {'dir':'./out'}::op_write_file_options);`
- 整个 STRUCT 传 NULL 就是一个 NULL 参数，它和内容传 NULL 一样**短路成 NULL、什么都不写**。它不是
  「其余项按缺省」；要缺省就**省略第 3 个参数**。

### 放进循环之前

有两件事适用于每一个名字：

- 等不等是启动那一组里两个名字唯一的可见差别：`open::that` 在启动器返回时才返回，而 Linux 上 `xdg-open`
  通常要到目标程序退出才返回。Windows 上（依赖里那个 `shellexecute-on-windows` feature）
  `op_open_path_detached` 反而是更直接的一条路：一次原生 API 调用，而不是「PowerShell 辅助进程 +
  explorer.exe 后备」；
- 凡是会碰外面世界的名字都注册成 **volatile**：一次调用就是一行，既不折叠也不缓存 ——
  `SELECT op_open_path('report.html')` 开一个窗口，`SELECT op_open_path(p) FROM reports` 一行开一个，
  常量参数和列参数的表现完全一样。规划一条查询时也不会执行到它们，所以
  `EXPLAIN SELECT op_write_file('a', 'txt')` 一个文件都不写。对 `op_write_file` 来说，「一行一个文件」正是
  自动命名要解决的问题。

空的路径、空的程序名、空的后缀、用不了的选项值都是错误，而不是一个 `false` 或者一次静默的缺省 ——
因为「写错了」和「打不开」该有不同的答复。DuckDB 报的是 invalid input error，报文最前面是函数自己的名字：

```sql
SELECT op_open_path('');    -- op_open_path: the path or URL must not be empty
SELECT op_write_file('a', '');
-- op_write_file: the extension must not be empty (pass e.g. 'html', not '')
SELECT op_write_file('a', 'txt', {'dir':'   ', 'name':NULL, 'part':NULL});
-- op_write_file: op_write_file_options.dir must not be empty
```

启动器是桌面软件，所以这个扩展只构建桌面平台（见下面「平台」一节）。在没有桌面的机器上，四个
`op_open_path` 动作一律答复 `false`，`op_open_content` 与 `op_open_html` 报一个带着它们确实写出来的路径的错误，
而 `op_open_commands` 照样能答 —— 它列出这台机器上会依次尝试什么。`op_write_file`、`op_print` 与两个
`op_sanitize_filename` 名字根本不需要桌面。见 `src/extension/launcher.rs`。

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
错误的检查）、不起任何进程的 `op_open_commands` 预览，以及整套落盘本身 —— `op_write_file` 写出去的文件用
`read_text` / `read_blob` 读回来逐个验（命名、建目录、BLOB 字节原样往返、非法字符替换、同名覆盖），
`op_print` 是真的被调用、断言它回的是它打出去的那段文本。`op_open_content` 与 `op_open_html` 走的是同一个 `writer::write`，
所以它们的测试留在 catalog 一侧，只跑那些到不了 launch 那一步的调用。`op_sanitize_filename` 那一族既不碰桌面
也不碰磁盘，所以它的测试逐字断言每一次改写：库的六条规则一条条对、这个 API 刻意不给的那一档参数个数
（`op_sanitize_filename(x, '_', true)` 是个 binder error）、以及判断与改写在同样的开关下始终对得上。

这些测试会往 `test/sql/`（运行器的 `__TEST_DIR__`）里写产物，所以 `.gitignore` 在那儿只留 `*.test`，
而 `just test` 末尾会再跑一次 `git clean -fdX -- test/sql`。开发指南的
[测试](docs/docs/development/testing.md)一页写了迭代循环（直接对着 `build/debug/*.duckdb_extension`
跑运行器）以及每个测试文件覆盖到什么。

## 平台

只有 Windows、Linux 与 macOS。启动器是桌面软件，所以这个扩展不构建 WebAssembly 目标：CI 用
`.github/workflows/MainDistributionPipeline.yml` 里的 `exclude_archs` 排掉三条 wasm 架构，而
`just build_wasm` / `build_wasm_eh` / `test_wasm` 在本项目里被覆盖成一句说明，不再构建任何东西。
在没有桌面的机器上，四个 `op_open_path` 动作一律答复 `false`，`op_open_content` / `op_open_html` 报一个带着路径的
错误，而 `op_open_commands`、`op_write_file`、`op_print` 与 `op_sanitize_filename` 那一对照常工作。

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
| [docs/docs/user-guide/functions/](docs/docs/user-guide/functions/index.md) | 函数参考：逐个名字的签名与示例，按组分成几页 |
| [README.md](README.md) | this file, in English |
