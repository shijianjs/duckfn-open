[English](README.md) | [简体中文](README.zh.md)

# duckfn_open

用 [duckfn](https://crates.io/crates/duckfn) 写的 DuckDB [可加载扩展](https://duckdb.org/docs/stable/extensions/extension_development)：
属性宏把普通 Rust 函数变成 DuckDB 的标量函数，全程不碰 C++ 构建（C API 只用头文件，走 DuckDB 的 API 函数表分发）。

它给 SQL 加的东西就是 [open](https://crates.io/crates/open) 这一件事：把一个路径或 URL 交给桌面当前配置好的程序，
于是查询可以直接把报表送到人眼前——

```sql
SELECT open_path('/var/tmp/report.html');
SELECT open_path_detached('https://duckdb.org');
```

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

`open_commands` 是这个扩展里唯一不起进程、也不看桌面配置的入口，所以最适合先试：它回答的是「在这里到底会拿什么去打开」，
而不起任何进程。

`Justfile` 包住同一批命令：`just build`、`just sql "SELECT open_commands('report.html')"`、
`just repl`（扩展已经 LOAD 好的 REPL）。

## 函数

三个 SQL 名字、五个签名，全是标量函数。`open_path` 与 `open_path_detached` 各带两个签名——默认程序，
以及第二个参数指定的程序——用 duckfn 的 `overloads_name` 并成一个函数集。实现在 `src/extension/functions/`。

| 函数 | 输入 → 输出 | 对应的 open 调用 |
| --- | --- | --- |
| `open_path(path)` | `VARCHAR` → `BOOLEAN` | `open::that` |
| `open_path(path, app)` | `VARCHAR, VARCHAR` → `BOOLEAN` | `open::with` |
| `open_path_detached(path)` | `VARCHAR` → `BOOLEAN` | `open::that_detached` |
| `open_path_detached(path, app)` | `VARCHAR, VARCHAR` → `BOOLEAN` | `open::with_detached` |
| `open_commands(path)` | `VARCHAR` → `VARCHAR[]` | `open::commands` |

SQL 里能分辨三种结果，这就是全部约定：

- **`NULL`** —— 这一行没有输入。入参都不是 `Option`，所以 NULL 由 duckfn 的参数读取层短路成 SQL NULL，
  启动器一次都不会被叫起来（`SELECT open_path(NULL); -- NULL`）；
- **`true`** —— 启动器被拉起了。不带后缀的那一组拿得到启动器自己的返回，所以 Linux 上它意味着目标程序跑过并退出；
  Windows 上开了 `open` 的 `shellexecute-on-windows` feature 之后，`open_path_detached` 走一次 ShellExecuteW，
  它只说明进程起来了；
- **`false`** —— 一个启动器都没能起来（这台机器没有 `xdg-open`、指定的程序不存在）。open 那边的真实原因是一个
  `io::Error`，而 SQL 里没有地方放它，所以要知道「这里到底会跑什么」请用 `open_commands`。

空的路径或程序名是一个错误而不是一个 false —— `Invalid Input Error: open_path: the path or URL must not be
empty` —— 因为「写错了」和「打不开」该有不同的答复。

批量调用之前还要知道两件事：

- 等不等是两组名字唯一的可见差别：`open::that` 在启动器返回时才返回，而 Linux 上 `xdg-open` 通常要到目标程序
  退出才返回。Windows 上（依赖里那个 `shellexecute-on-windows` feature）`open_path_detached` 反而是更直接的一条路：
  一次原生 API 调用，而不是「PowerShell 辅助进程 + explorer.exe 后备」；
- 参数全是常量的调用可能被 DuckDB 折叠成一次执行，所以 `SELECT open_path('report.html')` 只开一个窗口，
  而 `SELECT open_path(p) FROM reports` 一行开一个 —— 不去重。
启动器是桌面软件，所以这个扩展只构建桌面平台（见下面「平台」一节）。在没有桌面的机器上，四个启动动作一律
答复 `false`，而 `open_commands` 是那个照样能答的调用 —— 它列出这台机器上会依次尝试什么。见
`src/extension/launcher.rs`。

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
just test          # make configure + make debug + make test
just ci-build      # 只跑官方构建流程，不跑测试
```

`make test` 不会重新构建，改了 Rust 代码要先 `just ci-build`（或 `make debug`）。能不能打开取决于这台机器的桌面配置，
而 CI 的 runner 根本没有桌面，所以测试断言的是不依赖桌面的那部分：`duckdb_functions()` 里注册的签名形状、NULL 短路、
参数校验，以及不起任何进程的 `open_commands`。开发指南的[测试](docs/docs/development/testing.md)一页写了迭代循环
（直接对着 `build/debug/*.duckdb_extension` 跑运行器）以及每个测试文件覆盖到什么。

## 平台

只有 Windows、Linux 与 macOS。启动器是桌面软件，所以这个扩展不构建 WebAssembly 目标：CI 用
`.github/workflows/MainDistributionPipeline.yml` 里的 `exclude_archs` 排掉三条 wasm 架构，而
`just build_wasm` / `build_wasm_eh` / `test_wasm` 在本项目里被覆盖成一句说明，不再构建任何东西。
在没有桌面的机器上，四个启动动作一律答复 `false`；`open_commands` 照样能答。

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
| [README.md](README.md) | this file, in English |
