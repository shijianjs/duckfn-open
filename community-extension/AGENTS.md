# community-extension/ 的约定

这个目录不参与扩展运行，它是**向 [duckdb/community-extensions](https://github.com/duckdb/community-extensions)
提 PR 的暂存处**：那个 PR 只加两个文件，这里按目标路径摆好，提交时复制过去即可。

| 本目录 | 社区仓里的位置 |
| --- | --- |
| `description.yml` | `extensions/<扩展名>/description.yml` |
| `docs/function_descriptions.csv` | `extensions/<扩展名>/docs/function_descriptions.csv` |

目录名必须与 `extension.name` **逐字一致** —— 社区仓的 `scripts/build.py` 会校验，不一致直接报错。

## 提交进度与还要改什么

`description.yml` 的 `extension.description` / `maintainers` / `repo.github` 与 `docs.hello_world` /
`docs.extended_description` 都已按本扩展的真实 API（九个 `op_*` 名字）写好，改 API 时跟着改 —— 改了
`hello_world` / `extended_description`，或改了函数属性上的 `description` / `example`，都要重生成
`docs/function_descriptions.csv`（见下面「那份 CSV 是必须的」），并**真跑一遍** `hello_world`：它渲染进社区
文档页的代码块，写歪了不会有人报错（跑的时候别真去 launch —— 会起进程那句用 `EXPLAIN` 代替）。

**首次提交**：2026-10-15，[#3078](https://github.com/duckdb/community-extensions/pull/3078)。
`extension.version` 与 `repo.ref` 都指向 **v0.1.0**（`6995c11bb78279aab4caf5b11f3d0df3011fb3a7`）；
`repo.ref_next` 与 `ref` 同一个提交 —— 本扩展对 DuckDB 2.0 不需要改任何代码。之后每次发版回来改这两处
（见下面「每次发版都要回来改」）。本文件下面提到的路径与用户名也照本机实际情况核对一遍。

`extension.name` / `version` / `license` / `language` / `build` / `requires_toolchains` 在模板里已经是可直接
沿用的值（下面一节解释为什么），只有 `name` 要跟着你改名后的扩展名走（`scripts/rename.sh` 会一起改）。

## `description.yml` 里**不要写注释**

这份文件会被原样复制到上游，所以刻意保持「只有字段」。需要解释的东西写在本文件里，不要写回 YAML。

`repo.ref` 写**发布那一版的提交 SHA（40 位）**（`git rev-list -n 1 v0.1.0`），**不要写 `main`、也不要写
tag 名**。三者都是合法 git ref、社区仓都能照着 clone，但上游已收录的扩展清一色用提交 SHA
（`extensions/h3`、`extensions/orc` 都是），跟着走既不用解释，也天生不可变 —— 注册项指向的东西不会随时
间漂移。写 `main` 的代价是实打实的：构建出来的二进制会自称 main 上的开发版本（`X.Y.Z-dev.N`），与这里
声明的 `version` 对不上，而且同一个注册项在不同时间构建出的是不同代码。

## `repo.ref_next`：下一个大版本那一腿

社区注册表除了正式版那条腿，还跑一条**预发布**腿（他们的 `build_next.yml`：DuckDB 用
`v2.0-cyanoptera` 分支、ci-tools 用 `main`、并带 `require_ref_next: true`）。这条腿只认描述文件里的
`repo.ref_next`：没有这个字段就直接跳过，日志里打一句
`Skipping prerelease validation: repo.ref_next is not defined` —— 也就是那条 PR 什么都没验证。

本扩展对 2.0 **扩展代码与测试都不用改**（只用 C API 稳定区、也不构建 wasm；本机拿 2.0 引擎把整套
sqllogictest 跑过是 8/8 全绿，仓库自己的 2.0 腿也是绿的），属上游 `UPDATING.md` 说的 state 1 ——
所以正式提交时 `ref` 与 `ref_next` 填**同一个发布提交**就行（上游的 `duck_geoarrow`、`duck_hunt`
就是这么写的）。

本仓还没提交注册，所以现在只是先在这里摆上：`ref_next` 填当前 main 的主提交（`7a666e6`，它的两条腿都
跑绿了），`ref` 仍是发布时才填的占位符。注意它**不跟着 main 走** —— 它钉的是「跑绿那一次运行所用的提交」，
后面再有提交也不会自动跟上；要改就按下面的时机来。

改动它的时机：先在本仓 `.github/workflows/MainDistributionPipeline.yml` 的 `duckdb-next-build` 腿下改到
绿，再把 `ref_next` 换成那次运行的提交 SHA（跑一次 `gh workflow run "Main Extension Distribution
Pipeline" --ref main`，绿了就把 `git rev-parse HEAD` 那个值填上）。

## 字段为什么这么写（都不是猜的）

- `language: Rust` / `build: cargo`：本仓 Makefile 就是官方 Rust 模板那一套（include
  `extension-ci-tools/makefiles/c_api_extensions/rust.Makefile`）；社区仓里的 `rusty_quack`
  （仓库就是 `duckdb/extension-template-rs`，即该模板）用的正是 `build: cargo`。
- `requires_toolchains: "rust;python3"`：与 `.github/workflows/MainDistributionPipeline.yml` 的
  `extra_toolchains` 一致；`python3` 是 extension-ci-tools 的 configure / 测试流程要的。
- `excluded_platforms: "wasm_mvp;wasm_eh;wasm_threads"`：本仓自己的 CI 已经把 wasm 三条架构排除在外
  （`MainDistributionPipeline.yml` 的 `exclude_archs`，见根 `AGENTS.md`「排除 wasm 构建」），但**社区仓的
  构建跑的是它自己那份矩阵**：`duckdb/community-extensions/.github/workflows/build.yml` 只把
  `override_repository` + `override_ref`（你的仓库与提交 SHA）转给指定 ref 上的
  `_extension_distribution.yml`，本仓工作流的输入（包括 `exclude_archs`）根本到不了那边（依据见 duckfn
  文档 `docs/docs/duckdb-versions/wasm-toolchain.md` 的 "The community registry build uses its own pins"，
  里面给出的正是这一行）。
  而且这条**删不掉**：duckfn 文档里把它写成「注册仓的 Rust 过期时的临时措施，等上游 #394 修好就可以撤」，
  我们不是那个原因 —— `open` 在 emscripten 上直接 `compile_error!`，与对方的 Rust 版本无关，那三条 wasm job
  只会红。上游已收录的扩展写这一行的很多（`extensions/ai`、`extensions/dicom`、`extensions/dbn` 都是三条
  wasm）。两个 musl 平台（`linux_amd64_musl` / `linux_arm64_musl`）在社区矩阵里是 `opt_in`，不主动点名就
  不会构建，暂时不写；真红了再回来加。其它平台同理 —— 不要改依赖去找 wasm 的出路。
- `version`：写**要发布的那一版**，不要 `-dev.N`（开发版本是 `0.1.1-dev.0`，发版流程把它抬成 `0.1.1`）。
  它与 `repo.ref` 配套 —— `ref: v0.1.1` ↔ `version: 0.1.1`，所以社区仓构建出的二进制自称的版本、
  文档页上的版本号、这里的字段三者一致，不存在漂移。`just release_bump` 会顺手改掉这里的 `version`，
  `repo.ref` 仍然要人工改。
- `license: MIT`：对应仓库根目录的 `LICENSE`。注意 duckdb.org 的社区扩展文档页把字段名写成 `licence`，
  那是**文档的错**，真实 schema 是 `license`（以已收录扩展的 `description.yml` 为准）。
- `docs.hello_world`：社区文档页会把它渲染进代码块，所以必须是**可直接复制跑**的真实例子 —— 现在这三条是
  「预览会依次尝试的启动命令 / 把已有路径交给默认程序 / 把一段内容落盘并回路径」。页面上
  「Installing and Loading」那段由社区仓的 `layout/default.md` 自动加，`hello_world` 里不要再写
  `INSTALL` / `LOAD`。

## 那份 CSV 是必须的，而且会过期

社区文档页「Added Functions」表里，函数的 description / comment / example **只有一个来源**：这个 CSV
（DuckDB 的 C 扩展 API 没有设置它们的接口）。它由 `just docs_csv` 从属性宏上的 `description` /
`comment` / `example` 生成。**改了那些属性就要重新生成、覆盖这里这份**，否则文档页停在旧文案上。

```shell
just docs_csv
cp target/function_descriptions.csv community-extension/docs/function_descriptions.csv
```

这一份已经是按本扩展的九个名字生成的（与 `target/function_descriptions.csv` 一致）；改了属性就重跑上面
两条命令覆盖它。

## 提交：在 fork 的克隆里做，本仓只出那两张文件

上游注册仓（`duckdb/community-extensions`）的 fork 与本地克隆：

- fork：`shijianjs/duckdb-community-extensions`
- 克隆：`S:\workspace\my\rust\duckdb\duckdb-community-extensions`（`origin` 就是这个 fork，没有配
  `upstream`）—— 按本机实际路径替换。

流程是「加分支 → 复制两个文件 → 提交 → 推分支 → 开 PR」，本仓这边一行都不用改：

```powershell
$p = 'S:\workspace\my\rust\duckdb\duckdb-community-extensions'
$ext = 'duckfn_open'                                # 与 extension.name 逐字一致
git -C $p checkout main; git -C $p pull              # 先和 fork 的 main 同步
git -C $p checkout -b add-$ext                       # 已有同名分支就跳过这步

New-Item -ItemType Directory -Force "$p\extensions\$ext\docs" | Out-Null
Copy-Item community-extension\description.yml "$p\extensions\$ext\description.yml" -Force
Copy-Item community-extension\docs\function_descriptions.csv "$p\extensions\$ext\docs\function_descriptions.csv" -Force

git -C $p add extensions/$ext
git -C $p commit -m "Add $ext: …"                    # 上游惯例是 `Add <扩展名>` 再跟一句说明
git -C $p push -u origin add-$ext
gh pr create --repo duckdb/community-extensions --base main --head shijianjs:add-$ext `
  --title '…' --body-file …
```

- **每次发版都要回来改一处**：`repo.ref` 换成新发布提交的 SHA（`git rev-list -n 1 v0.1.1`），
  `version` 跟着改成 `0.1.1`，改完在本仓 `community-extension/` 里改，再复制进那份克隆，推到同一条 PR
  分支（PR 会自动更新）或另开一个。注册项钉的是具体提交，不会自己跟。
- PR 一开，上游会起三条构建（`Community Extension Build`、`Community Extension with latest DuckDB`、
  `Community Extension with DuckDB on Andium`）。外部贡献者的 PR 会被 GitHub 挂成 `action_required`
  —— 等维护者点 Run，属正常状态，别因此重推。
- 想把 fork 与上游同步时，那份克隆里得先补个上游远程（它只配了 `origin`）：
  `git -C $p remote add upstream https://github.com/duckdb/community-extensions.git`，之后
  `git -C $p fetch upstream && git -C $p merge upstream/main`。

## 提交前自查

```powershell
# 1. 键集要和社区仓 generate_extensions_json.py 读的一致（三组顶层键 + 各组的子键）
.\configure\venv\Scripts\python.exe -c "import yaml;d=yaml.safe_load(open('community-extension/description.yml',encoding='utf-8'));print(list(d));print(list(d['extension']));print(list(d['repo']));print(list(d['docs']))"

# 2. hello_world 原样能跑：把 docs.hello_world 抽成 .sql，前面补一行 LOAD 本地产物，再交给本机 duckdb
duckdb -unsigned -c ".read target/hello_from_yml.sql"
```

- 仓库根目录必须有 `LICENSE`（社区扩展要求开源，`license` 字段只是声明）。
- `INSTALL <扩展名> FROM community` 要等 PR 合并后才真的可用 —— 合并前 README 里那套加载方式
  只对之后的版本成立。
