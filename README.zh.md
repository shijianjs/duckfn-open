[English](README.md) | [简体中文](README.zh.md)

# my_extension

一个用 [duckfn](https://crates.io/crates/duckfn) 写的 DuckDB [loadable extension](https://duckdb.org/docs/stable/extensions/extension_development)：
属性宏把普通的 Rust 函数变成 DuckDB 的标量 / 聚合 / 表函数，全程不碰 C++ 构建（只经 DuckDB 的 API
函数表使用 C API，不链接库）。

本仓库是一份**模板**：[duckfn-extension-template](https://github.com/shijianjs/duckfn-extension-template)。
它把完整的开发闭环 —— 构建、sqllogictest、函数描述导出、发版 —— 都装在两个示例函数周围，新扩展从
「一次就能编过、就能跑测试」的状态起步，而不是从空目录起步。同一种写法写出来的真实扩展见
[duckfn_quantstats](https://github.com/shijianjs/duckfn-quantstats)。

## 用这份模板开一个新扩展

```shell
git clone https://github.com/shijianjs/duckfn-extension-template my_new_extension
cd my_new_extension
rm -rf .git && git init    # 可选：丢掉模板的历史，从头开始自己的仓库
just rename my_new_extension
```

`just rename`（即 `scripts/rename.sh`）会把扩展名必须一致的地方一次改齐 —— crate 名与
`[[example]] name`、Makefile 的 `EXTENSION_NAME`、入口点符号、Justfile、CI 工作流，以及文档里的路径
示例 —— 并按新包名重写 `Cargo.lock` 里那一条。脚本末尾会打印剩下需要人工过一遍的事情，清单在开发
指南的[快速开始](docs/i18n/zh-Hans/docusaurus-plugin-content-docs/current/development/quick-start.md)
（下一步）与 [AGENTS.md](AGENTS.md)（约定，含 `{{PROJECT_GOAL}}`
占位符）里。把两个示例函数换成你自己的 API 就是其中一条。

## 快速上手

```shell
make configure   # 只做一次：建 configure/venv（Python 与 sqllogictest 运行器）
make debug       # -> build/debug/my_extension.duckdb_extension
```

自己构建的产物没有签名，加载时必须给 DuckDB 加 `-unsigned`：

```shell
duckdb -unsigned -c "
LOAD './build/debug/my_extension.duckdb_extension';
SELECT my_greet('world');
-- Hello, world!
SELECT my_sum(x) FROM (VALUES (1.5::DOUBLE), (2.5::DOUBLE), (3.0::DOUBLE)) t(x);
-- 7.0
"
```

`Justfile` 把同样的命令包了一层：`just build`、`just sql "SELECT my_greet('world')"`、
`just repl`（已 LOAD 扩展的 REPL）。

## 函数

两个示例函数，各演示一条注册路径。它们是给你替换的，代码在 `src/extension/functions/`。

| 函数 | 类别 | 入参 → 出参 |
| --- | --- | --- |
| `my_greet(name)` | 标量 | `VARCHAR` → `VARCHAR`，永不为 NULL |
| `my_greet_checked(name)` | 标量 | `VARCHAR` → `VARCHAR`，空串回 `NULL`，首尾空格报错 |
| `my_sum(value)` | 聚合 | `DOUBLE` → `DOUBLE`，跳过 NULL，空组回 `NULL` |

下面这些行为值得先知道，它们是 duckfn 的规则而不是本模板的：

- 入参写成不可空的 `T` 时，NULL 行被短路成 SQL NULL —— 函数体根本不会执行到那一行；想让 NULL 进函数体
  自己决定语义，把参数写成 `Option<T>`；
- 标量函数要返回 NULL（`Ok(None)`）或让查询失败（`Err`），用 `-> DuckOptionResult<T>`；
- 聚合函数就是「多一个 `&mut` 状态参数的函数」：状态的 `Output` 决定 SQL 返回类型，`result` 决定这一组
  是出值还是出 NULL。

## 从源码构建

构建走官方 DuckDB `extension-ci-tools` makefile：

```shell
make configure           # 只做一次：建 configure/venv（Python 与 sqllogictest 运行器）
make debug               # -> build/debug/my_extension.duckdb_extension
```

`make release` 是带优化的同一套流程。Windows 上 `make` 需要在 Git Bash 里跑。
`Justfile` 把它包了一层（`just build` = `make configure && make debug`、`just ci-build`、`just test`、
`just ci-release`）。

## 测试

测试用 SQLLogicTest 格式写在 `test/sql/` 下：

```shell
just test          # make configure + make debug + make test
just ci-build      # 只做官方构建，不跑测试
```

`make test` 不会自动重新构建，改完 Rust 先跑 `just ci-build`（或 `make debug`）。
更快的迭代方式（拿运行器直接跑 `build/debug/*.duckdb_extension`）与各测试文件覆盖什么，
见开发指南的[测试](docs/i18n/zh-Hans/docusaurus-plugin-content-docs/current/development/testing.md)。

## WebAssembly

```shell
just config_env   # 只做一次：固定工具链版本并装上 wasm target
just build_wasm
```

wasm 构建走 `src/wasm_lib.rs`（`src/lib.rs` 的 `staticlib` 镜像），两个 crate root 必须始终声明同一组
`mod`。

## 文档站

仓库里带着一个 [Docusaurus](https://docusaurus.io/) 站点（`docs/`），中英双语，并且有工作流在每次版本
tag 时把它发布到 GitHub Pages：

```shell
just docs_install    # 只做一次
just docs_start      # 本地预览 http://localhost:3000
just docs_build      # 真正该跑的那一条：onBrokenLinks 设为 throw，链接断了就构建失败
```

模板里的页面写的是示例函数；你的 API 长出自己的样子之后，把这些页面（以及 `docs/i18n/zh-Hans/` 下的译文）
改掉，或者直接删掉 `docs/` —— 仓库里没有别的东西依赖它。页面里带着可运行的 SQL 块（由
[`duckfn-docs-kit`](https://www.npmjs.com/package/duckfn-docs-kit) 提供），在浏览器里直接调用本扩展；
`cd docs && npm test` 会把它们重跑一遍。约定（目录、命令、翻译流程、部署、`{{EXTENSION_VERSION}}`
版本占位符）写在 [`docs/README.md`](docs/README.md) 里。

## 安装已发布的扩展

发布物是 GitHub Release 上各平台构建出的 `.duckdb_extension` 文件：

```shell
duckdb -unsigned -c "
LOAD 'https://github.com/<owner>/<repo>/releases/latest/download/my_extension-windows_amd64.duckdb_extension';
"
```

注册进 DuckDB 的[社区扩展](https://duckdb.org/community_extensions/list_of_extensions)之后，就变成
一句 `INSTALL my_extension FROM community`；那需要提交的两份文件已经备在
[`community-extension/`](community-extension/AGENTS.md)。

## 文档

| 文件 | 里面有什么 |
| --- | --- |
| [AGENTS.md](AGENTS.md) | 约定、duckfn 知识地图、发版流程 |
| [docs/i18n/zh-Hans/…/development/](docs/i18n/zh-Hans/docusaurus-plugin-content-docs/current/development/quick-start.md) | 开发指南：目录结构、骨架取舍、写函数与测试、构建与发版 |
| [docs/README.md](docs/README.md) | 文档站：目录、命令、翻译流程、部署 |
| [README.md](README.md) | 本文件，英文 |
