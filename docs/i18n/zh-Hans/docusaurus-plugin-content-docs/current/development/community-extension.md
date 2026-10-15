---
title: 社区扩展
sidebar_position: 6
description: 把扩展注册进 DuckDB 社区仓：需要哪两份文件、每个字段填什么，以及每次发版怎么让它跟上。
---

# 社区扩展

把扩展注册进 [duckdb/community-extensions](https://github.com/duckdb/community-extensions)，才能把
「从 GitHub Release 下载一个文件」变成：

```sql
INSTALL duckfn_open FROM community;   -- 只需一次，需要网络
LOAD duckfn_open;
```

社区仓构建出的产物是签过名的、并与用户的 DuckDB 版本严格匹配，所以不再需要 `-unsigned`。注册本身是一个
只加**两份文件**的 PR：

| 本仓库 | 社区仓里的位置 |
| --- | --- |
| `community-extension/description.yml` | `extensions/duckfn_open/description.yml` |
| `community-extension/docs/function_descriptions.csv` | `extensions/duckfn_open/docs/function_descriptions.csv` |

目录名必须与 `extension.name` 逐字一致 —— 社区仓的 `scripts/build.py` 会校验这一点。

## 那份 CSV

社区扩展页 `Added Functions` 表里的 description / comment / example 只有一个来源：这份 CSV
（DuckDB 的 C 扩展 API 没有设置它们的接口）。文案来自 `#[duck_*]` 属性上的 `description` / `comment` /
`example`，也就是说它紧挨着被描述的函数：

```rust
#[duck_scalar_function(
    description = "Lists, without running them, the launcher commands op_open_path would try in order",
    comment = "The read-only pair of op_open_path: nothing is started, so the result is stable enough to \
               assert in CI on a headless runner",
    example = "SELECT op_open_commands('report.html')"
)]
```

`op_open_path` 用的不是单个 `example` 而是 `examples = [ … ]`，因为它有两个值得展示的调用形式
（`(path)` 与 `(path, app)`）；导出时它们用 `"; "` 拼在一起。CSV 里也是**一个 SQL 名字一行** ——
这个扩展二十一个签名并成九行 —— 而不是一个签名一行：靠 `overloads_name` 并进来的签名会被折进同一个函数集，
所以 `op_write_file` 有四个签名也还是一行。

这个「合并」正是文案只写在**一个**签名上的原因。`description` 与 `comment` 取的是编译期注册表迭代顺序里
第一个非空值，而那是链接顺序、不是源码顺序 —— 两条 overload 都写文案的话，导出结果取哪一条就取决于链接器
先看到谁。`example` / `examples` 是例外：它们拼接后按内容去重，所以并进来的签名可以自由贡献自己的示例，
不会顶掉别人的。

改了这些属性就重新生成、覆盖过去：

```shell
just docs_csv
cp target/function_descriptions.csv community-extension/docs/function_descriptions.csv
```

想知道哪些函数还没写进这份 CSV，CLI 可以把没写文案的那些导出来：

```shell
cargo run --bin duckfn -- function_descriptions --all    # -> target/function_descriptions_all.csv
```

它不加载扩展、不查目录，读的是编译期记录下来的内容，所以身边不需要有 DuckDB。

从属性到页面：

```mermaid
flowchart LR
    attrs["属性上的 description / comment / example"] --> csv["function_descriptions.csv<br/>just docs_csv"]
    csv --> copy["community-extension/docs/"]
    copy --> page["社区扩展页<br/>Added Functions 表"]
```

多条示例导出时用 `"; "` 拼接、每条去掉结尾分号；换行会压成一个空格（生成页是 Markdown 表格）。所以一句
一条完整 SQL，用英文写即可。

## `description.yml`

这个文件会被原样复制到社区仓，所以里面只有字段、没有注释。需要你决定的字段：

| 字段 | 填什么 |
| --- | --- |
| `extension.name` | 扩展名，与目录名、入口点符号逐字一致。 |
| `extension.description` | 一行说明，显示在扩展列表里。 |
| `extension.version` | 要发布的那一版，不要 `-dev.N` 后缀。 |
| `extension.language` / `build` | 从本模板起步的项目就是 `Rust` 与 `cargo`。 |
| `extension.license` | `MIT`（对应仓库根目录的 `LICENSE`）。注意社区**文档页**把它写成 `licence`，真实 schema 是 `license`。 |
| `extension.requires_toolchains` | `"rust;python3"` —— 与 CI 工作流的 `extra_toolchains` 一致。 |
| `extension.excluded_platforms` | `"wasm_mvp;wasm_eh;wasm_threads"` —— 这个扩展只面向桌面（`open` 在 emscripten 上编不过），而社区仓的构建跑的是它自己那份矩阵、从不读本仓的 `exclude_archs`。 |
| `extension.maintainers` | 你的 GitHub 账号。 |
| `repo.github` | `owner/repository`。 |
| `repo.ref` | 发布那一版的**提交 SHA**（40 位）—— `git rev-list -n 1 v0.1.0` —— 不要写 `main`，也不要写 tag 名。注册项指向的应当是不可变的代码，而从分支构建出的产物会自称开发版本。 |
| `repo.ref_next` | 社区注册表的**预发布腿**要构建的那个提交：那条腿以这个字段为门禁，没有它直接跳过。本扩展对新引擎不需要改任何东西，所以提交时它可以与 `ref` 填同一个 SHA —— 依据见 `community-extension/AGENTS.md`。 |
| `docs.hello_world` | 可直接复制跑的示例，会渲染进代码块。不要在这里写 `INSTALL` / `LOAD`：页面自己会加。`community-extension/description.yml` 里那份草稿两半各给了一条 —— `op_open_commands` 与 `op_write_file`，两者都不起进程 —— 然后是一条 `op_open_path`，最后用 `op_open_path(NULL)` 收尾，那一行根本没有输入。 |
| `docs.extended_description` | 函数表周围的正文。 |

每个字段的来历与提交步骤，`community-extension/AGENTS.md` 里讲得更细。

## 提交

```shell
# 1. 在 duckdb/community-extensions 的 fork 克隆里
git checkout -b add-duckfn-open

# 2. 把两份文件复制到位
mkdir -p extensions/duckfn_open/docs
cp <本仓库>/community-extension/description.yml extensions/duckfn_open/
cp <本仓库>/community-extension/docs/function_descriptions.csv extensions/duckfn_open/docs/

# 3. 提交、推分支、开 PR
git add extensions/duckfn_open
git commit -m "Add duckfn_open: …"
gh pr create --repo duckdb/community-extensions --base main --head <你>:add-duckfn-open
```

维护者会点起来那几条构建工作流（首次贡献者的 PR 会先挂在 `action_required`，等有人批准后才跑，属正常
状态）。合并之后 `INSTALL duckfn_open FROM community` 就真的可用了，README 也可以改成那种加载方式。

## 每次发版都要回来改

`repo.ref` 与 `extension.version` 钉的是某一版。每次发版之后，在本仓库的 `community-extension/` 里改这两
处，复制进社区仓那份克隆，推到同一条 PR 分支（PR 会自动更新）或另开一个。`just release_bump` 已经会顺带
改写 `version` 字段；提交 SHA 那一处仍然要人工填。
