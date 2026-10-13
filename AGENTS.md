<!--
AGENTS.md（duckfn-extension-template 自带的那一份）：克隆模板后**只改下面「项目事实」里的
{{PROJECT_GOAL}}**，其余部分（约定与流程）不用动，它们对任何 duckfn 扩展都成立。
-->

# AGENTS.md

这是一个用 [duckfn](https://crates.io/crates/duckfn) 写的 DuckDB 扩展（loadable extension）。

## 项目事实（唯一需要人维护的一段）

- 这个扩展做什么：{{PROJECT_GOAL}}
- 本仓库来自 [duckfn-extension-template](https://github.com/shijianjs/duckfn-extension-template)：
  克隆后第一件事是 `just rename <新扩展名>`（见下面「扩展名与改名」）。

> 扩展名、crate 名、duckfn 版本**不要抄到这里**：
> 扩展名读 `src/extension/mod.rs` 里的 `duckfn_entrypoint!("...")`（也是 `Makefile` 的 `EXTENSION_NAME`），
> crate 名与 duckfn 版本读 `Cargo.toml`。它们本来就在代码里，复制一份只会变成第二份会过期的真相。

## 动手前先读

**duckfn 的文档与示例随 crate 一起发布**（0.0.11 起）：跑过一次 `cargo build` 之后它们就在本机 cargo 的
解包目录里，与 `Cargo.toml` 钉的版本严格对应 —— 不需要 clone duckfn 仓库，也不需要联网。

```powershell
# Windows：版本号从 Cargo.toml 读（例如 0.0.11）
Get-ChildItem "$env:CARGO_HOME\registry\src\*\duckfn-<版本>" -Directory | Select-Object -ExpandProperty FullName
```

```bash
# Linux / macOS
ls -d ~/.cargo/registry/src/*/duckfn-*/
```

| 资源 | 路径（`<crate>` = 上面那个目录） |
| --- | --- |
| 示例扩展（各类注册方式都有可运行实现） | `<crate>/src/extension/**`：`functions/` 每类一个文件、`types/` 自定义类型、`demo/` 组合示例、`entry.rs` 入口 |
| sqllogictest 范例（41 份 `.test`） | `<crate>/test/sql/**` |
| 用户文档正文（英文） | `<crate>/docs/docs/**` |
| 用户文档正文（简体中文） | `<crate>/docs/i18n/zh-Hans/docusaurus-plugin-content-docs/current/**` |
| 一组可直接跑的 `just sql` 示例 | `<crate>/demo.sh` |

按主题查表（路径都相对 `<crate>`）：

| 主题 | 文档 | 参考实现 |
| --- | --- | --- |
| 示例扩展逐个功能讲解（先读它更快） | `docs/docs/examples/duckfn.md` | `src/extension/**`（`demo/` 是组合示例） |
| 全部属性与参数 | `docs/docs/guide/attributes.md` | — |
| 标量函数 | `docs/docs/guide/scalar-functions.md` | `src/extension/functions/scalar_function.rs` |
| 聚合函数 | `docs/docs/guide/aggregate-functions.md` | `src/extension/functions/aggregate_function.rs` |
| 表函数 | `docs/docs/guide/table-functions.md` | `src/extension/functions/table_function.rs`、`dynamic_table_function.rs` |
| `COPY ... TO` / `FROM` | `docs/docs/guide/copy-functions.md` | `src/extension/functions/copy_function.rs`、`copy_from_function.rs` |
| 类型转换 cast | `docs/docs/guide/casts.md` | `src/extension/functions/cast_function.rs` |
| 替换扫描 | `docs/docs/guide/replacement-scans.md` | `src/extension/functions/replacement_scan.rs` |
| SQL 宏 | `docs/docs/guide/sql-macros.md` | `src/extension/functions/sql_macro.rs`（脚本见 `src/extension/functions/sql/*.sql`） |
| STRUCT / ENUM 等自定义类型 | `docs/docs/guide/custom-types.md` | `src/extension/types/duck_struct_scalar_echo.rs`、`duck_enum_echo.rs` |
| Rust ↔ DuckDB 类型映射 | `docs/docs/guide/types.md` | `src/extension/types/**` |
| 宿主文件系统（`duck_vfs`） | `docs/docs/guide/file-system.md` | `src/extension/functions/file_system.rs` |
| 错误与 panic | `docs/docs/guide/errors-and-panics.md` | — |
| 构建与发布 | `docs/docs/development/build-and-release.md` | — |
| 排错（已知问题） | `docs/docs/known-issues.md` | — |
| 社区扩展文档页（`function_descriptions.csv`） | `docs/docs/community-extension-docs.md` | `src/extension/functions/*.rs`（带 `description` / `example` 的那几个） |

属性宏接受哪些参数、允许哪些返回形状，**真相在 `duckfn-macro` 的源码里** —— 它是独立发布的 crate，
解包在同一个 registry 目录下的 `duckfn-macro-<版本>/src/**`；文档与示例只覆盖常用面。

在线版本（文档站 <https://shijianjs.github.io/duckfn/zh-Hans/>、API <https://docs.rs/duckfn>）随时可能是
更新的一版，**与本机依赖冲突时以本地那份为准** —— 它就是实际编译的代码。

**铁律**：任何来源都拿不到时，停下来告诉用户「我查不到 duckfn 的这部分 API」，
不要凭记忆编属性名、参数或返回类型。写错的宏会以编译错误的形式暴露，
但更常见的是一路编到底、最后没法编译。

## 升级 duckfn 时

1. 改 `Cargo.toml` 里的 duckfn 版本，`cargo update -p duckfn -p duckfn-macro`。
2. `cargo build --all-targets` 跑一次：新版本的 crate 会被解包到 registry，文档、示例与 sqllogictest
   范例随包而来，自动与依赖对齐 —— 不需要任何 git 操作。
3. 本文件不用改：它只写占位符，不钉具体版本号。

## 仓库约定

### 临时文件放到 target/

生成的临时文件（脚本、数据、日志、一次性验证代码等）一律放到 `target/` 下，
不要放在仓库根目录或其它已跟踪的目录里。`target/` 已被 git 忽略，不会污染工作区，
用完顺手删掉。

### 文本文件一律用 LF

所有新增或修改的文本文件使用 LF（`\n`）换行，不要 CRLF（`\r\n`）。

任务结束时，对本次新增的文本文件**机械地跑一遍替换命令即可，不需要先检测**
里面是否真的有 CRLF：

```powershell
# PowerShell：逐个文件把 CRLF 换成 LF（保持 UTF-8 无 BOM）
foreach ($f in @('path/to/new-file.md', 'path/to/new-script.sh')) {
    $p = Join-Path (Get-Location) $f
    $c = [IO.File]::ReadAllText($p)
    [IO.File]::WriteAllText($p, ($c -replace "`r`n", "`n"), [System.Text.UTF8Encoding]::new($false))
}
```

```bash
# Git Bash / Linux / macOS
sed -i 's/\r$//' path/to/new-file.md path/to/new-script.sh
```

> 仓库开启了 `core.autocrlf`，所以 `git diff` 偶尔会提示 "LF will be replaced by CRLF"，
> 那是检出到工作区时的行为，提交进仓库的内容始终是 LF。

### 尽量用成熟三方库实现，不要自己造轮子

写任何「通用」逻辑之前先问一句：这件事是不是已经有 crate（或 std API）在做？

- **平台差异、临时文件与随机名、文件名的合法性规则、编码、哈希、日期时间算术、序列化** ——
  这类通用问题一律先找库。已经这么做的先例见 duckfn_quantstats：`open_in_browser` 用 `open`
  （各平台的启动命令与参数引用）、临时文件用 `tempfile`（随机尾缀与撞名重试）、
  文件名的合法性用 `sanitize-filename`（非法字符、保留设备名、结尾的点与空格）。
- **能用 std 就用 std**，别自己拼底层积木：路径绝对化用 `std::path::absolute`，而不是
  `env::current_dir()?.join(path)`；日期换算不要手算 epoch，交给 duckfn 的 chrono 桥
  （`DuckDate::to_naive_date` 这类方法）。
- 手写只允许出现在**领域逻辑**上（这个扩展真正要解决的问题），或者已知的库都不合适 ——
  后者必须在这段代码的注释里写明「为什么不用库」，例如判断一个路径能不能交给浏览器，
  看的是字面上的 `://` 而不是引一个 URL 解析库（Windows 的 `C:\…` 在 URL 语法里同样是 scheme）。
- 依赖不是免费的：引入 crate 时在 `Cargo.toml` 里写一句它负责什么、为什么选它，让取舍一眼看得出来；
  只服务某个平台的依赖挂到 target 专属依赖表下
  （见 duckfn_quantstats 的 `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`），别让别的目标
  替它付编译成本 —— 有时这甚至是硬要求（`open` 没有 emscripten 的实现，编到 wasm 直接失败）。
- 自查标准：一个「通用」函数如果在 crates.io 上能查到现成实现，它就需要一个留下来的理由。

### 扩展名与改名

扩展名必须**全小写、只含下划线与数字**，并且处处一致 —— 它是入口点符号名，也是产物文件名，
DuckDB 按文件名找符号，对不上 `LOAD` 就直接失败。五个地方：`Cargo.toml`（`[package] name` 与
`[[example]] name`）、`Makefile` 的 `EXTENSION_NAME`、`src/extension/mod.rs` 的
`duckfn_entrypoint!`、`Justfile` 的 `extension_name`、CI 工作流的 `extension_name`。

**不要手改**：跑 `just rename <新扩展名>`（`scripts/rename.sh`），它一次改齐上面五处以及文档与
README 里出现的路径示例，并把 `Cargo.lock` 按新包名重写；脚本末尾会打印还需要人工过一遍的清单。

### 注册到 DuckDB 的函数名统一加短前缀

凡是出现在 SQL 里的名字都加同一个前缀：标量函数、聚合函数、表函数、COPY 格式、cast、SQL 宏、
replacement scan。模板里这个前缀是 `my_`。

社区扩展几乎都不把包名/扩展名写进函数名（见
<https://duckdb.org/community_extensions/list_of_extensions>）：`my_extension_greet` 这样的全名在每个
调用点上都是纯噪声，而 `my_` 短到可以忽略，又足以在 `duckdb_functions()` 里按前缀检索。
**前缀只是命名空间，不再是扩展名的缩写** —— 不要因为扩展名变了就跟着改。

前缀之后的部分要能读懂，不要拿缩写堆砌。示例里的名字各只有一个签名：

```text
my_greet(name)
my_sum(value)
```

`rename` 脚本不动函数名（那是你的域代码）：换完扩展名顺手把示例函数与 `test/sql/*.test` 一起改成
你的 API，前缀也在这里一并定下来。

duckfn 的属性宏默认拿 **Rust 函数名**当注册名，所以直接把函数定义成 `fn my_xxx(...)` 即可。
宏还会为每个签名生成 `SQL_NAME` 常量：同一个名字要在多处出现（错误信息前缀、日志）时读它，
不要再抄一份字面量。代价是这类函数得写成 `pub(super)`，因为生成的模块沿用函数的可见性。
`overloads_name` 能把「同一名字下按参数个数/类型分派」的多个签名并成一个函数集，需要时再用
（见 duckfn 的文档）。

### 文档站（docs/）

`docs/` 是一份 Docusaurus 站点（英文 + 简体中文），**不是必须的**：不用就整个目录删掉，仓库里只有两处
引用它 —— `.github/workflows/DeployDocs.yml` 与 Justfile 的 `docs_*` recipe —— 一起删掉即可。

维护约定（目录、命令、翻译流程、部署、版本占位符）见 [`docs/README.md`](docs/README.md)；改可运行
SQL 块前先读 [`docs/AGENTS.md`](docs/AGENTS.md)（块里哪些路径不能写死、怎么读 `static/` 下的数据）。
可复用的部件
来自 npm 上的 [`duckfn-docs-kit`](https://www.npmjs.com/package/duckfn-docs-kit)（首页 `<dfk-*>` 组件、
目录折叠控件、版本占位符 remark 插件、可运行 SQL 块），站点里不再留副本，注册处见 `docusaurus.config.ts`。
三条容易踩的：

- 正文里**不要手写版本号**：写 `{{EXTENSION_VERSION}}`（放在围栏代码块或行内代码里），构建期由 kit 的
  `remarkVersionPlaceholder` 从 `docs/extension-version.ts` 替换。那个文件由 `just release_bump` 更新，
  是文档站版本号的唯一来源。
- 译文要在 `docs/i18n/zh-Hans/docusaurus-plugin-content-docs/current/` 下按**同样的相对路径**放一份全文，
  `id` / `slug` / `sidebar_position` 与英文页保持一致，页内链接用相对文件路径（写 `/docs/…` 会把中文页
  送到英文页）。可运行块的 SQL 是代码，照抄，只翻注释。
- **可运行 SQL 块**（info string 为 `{"type":"duckfn",…}` 的 `sql` 围栏）在浏览器里用 DuckDB-Wasm 真跑，
  并调用本扩展。本地预加载的是 `static/duckdb-extensions/` 下**本机构建的那份**：`just test_wasm` 会先
  `just build_wasm_eh` 构建、放进该目录，再跑一遍所有块；只有 GitHub Pages 部署才去拉 `REPO_URL` 指向
  仓库的最新 Release（由 `DOCS_EXTENSION_FROM_RELEASE` 区分，见 `docs/docusaurus.config.ts`）。因此克隆后
  要把 `REPO_URL` 改成自己的仓库（`just rename` 只改扩展名，不动这个 URL）。不需要这套能力时，把
  `docusaurus.config.ts` 里的 `remarkRunnableSql` 与 `dfkExtensions` 两行去掉即可。

站点分成两个侧边栏、对应顶栏两项（见 `docs/sidebars.ts`），**目录名就是受众**：

- **用户指南**（`docs/docs/user-guide/`）—— 面向用 SQL 调用这个扩展的人：它给 DuckDB 加了什么、怎么
  装与加载、注册了哪些函数。**不出现 Rust 与构建步骤**。
- **开发指南**（`docs/docs/development/`）—— 面向改这个仓库的人：快速开始、目录结构、写函数与测试、
  构建与发版、社区扩展注册。

新增页面按这个归属放进对应目录，两侧的文档树要各自翻译一份。首页（`docs/src/pages/index.tsx`）同样
面向用户：不展示 Rust 源码，讲的是安装与函数。

### 社区扩展注册

`community-extension/` 是向 [duckdb/community-extensions](https://github.com/duckdb/community-extensions)
提 PR 的暂存处，不参与扩展运行。字段依据与提交流程见
[`community-extension/AGENTS.md`](community-extension/AGENTS.md)。

## 共享 justfile：`scripts/common.just`

日常命令（`build` / `sql` / `repl` / `lint` / `test` / `docs_*` / `ci-*` / `build_wasm*` / `test_wasm` /
`release_*` …）
都在 `scripts/common.just` 里；根 `Justfile` 只 `import "scripts/common.just"`，再留下机器相关的
`set windows-shell`、项目相关的 `extension_name`，以及模板特有的 `rename`（由模板生成的项目还会
加上自己的 recipe 与覆盖）。

这份副本的**源在 duckfn 仓库**（`shijianjs/duckfn/scripts/common.just`）：各项目一份、逐字节相同。

| 命令 | 作用 |
| --- | --- |
| `just sync-common` | 拉回最新副本；默认跟 `main`，`DUCKFN_JUST_REF=vX.Y.Z just sync-common` 可钉到某个已发布版本 |
| `just check-common` | 只比对不写回，副本与源不一致时非零退出（可挂进自己的 CI） |

两条规则：

- **不要在副本里改共享 recipe** —— 改了下次同步就没了。要改就改 duckfn 仓库里那份，然后各项目
  `just sync-common`；只想在本项目里改行为，就在根 `Justfile` 里覆盖它，那需要先加
  `set allow-duplicate-recipes := true`：不开这个开关，重名 recipe 会让 just 在解析期直接报错
  （连 `just --list` 都跑不了）。
- 共享文件里**不写具体版本号**（用 `X.Y.Z` 占位），所以 `scripts/release.sh` 的版本替换与它无关，
  每次同步也不会多出一行噪音 diff。

## 发版流程

发版命令都在 `scripts/common.just` 里（`just --list` 可查，见上面「共享 justfile」），实际逻辑在
`scripts/release.sh`。
放进脚本而不是直接写进 Justfile，是因为 just 的 shebang recipe 在 Windows 上需要 `cygpath`
翻译解释器路径，而 Git Bash 并不提供它。

版本号形如 `X.Y.Z`（例如 `0.1.0`）。一次完整的发版 =
提升版本号 → 提交并打 tag → 等 CI 产出 GitHub Release → 切回下一开发版本。

**本项目不发 crates.io。** 它是 DuckDB 的 loadable extension，分发靠 GitHub Release 上的
`.duckdb_extension` 文件（`LOAD` 一个文件即用），所以 duckfn 流程里的发布 crate 那一步
在这里不存在。

只有**正式版本**才打 tag；`0.1.1-dev.0` 这类预发布版本留在分支上，不打 tag、不发布。

### 命令速查

| 步骤 | 命令 |
| --- | --- |
| 0. 前置检查 | `just release_check`（需要时再 `just test`） |
| 1. 提升版本号 | `just release_bump 0.1.0` |
| 2. 提交并打 tag | `git commit …` 后 `just release_tag 0.1.0` |
| 3. 查看 CI | `just release_ci` |
| 4. 切开发版本 | `just release_dev 0.1.1-dev.0` |

### 0. 前置检查

```bash
just release_check   # lint（clippy --all-targets -- -D warnings）+ cargo build --all-targets
just test            # 需要时（等价 make configure debug test，make 部分要在 Git Bash 里跑）
```

有 warning 先修好再提交。确认 `git status` 干净、`main` 已与远程同步。

### 1. 提升版本号

```bash
just release_bump 0.1.0
```

脚本做三件事，并打印每个被改动的文件：

- **Cargo.toml**：`[package]` 段的 `version` 从项目当前版本改成新版本。**只改这一行**，不做整份文件的
  全局替换 —— 本文件里还写着依赖需求（`duckfn = "0.0.10"` 这类），全局替换会把它一起改掉。
- **文档 / README / CI 注释 / Justfile 示例**：取**最近一次 tag** 的版本改成新版本，文件由 `git grep`
  自动找出，不需要维护清单；排除 `Cargo.toml`、`Cargo.lock`、`AGENTS.md`、`scripts/`、`test/`。
  其中 `test/` 是必须排掉的：那里的版本号是断言的期望值，与本项目的版本号无关。
  `community-extension/description.yml` 里的 `version` **会**被一起改（它就该跟发布版本走），
  但 `repo.ref` 那个提交 SHA 仍然要人工更新（见 community-extension/AGENTS.md）。
- `cargo update -p <扩展名>` 同步 `Cargo.lock`；然后回读 `Cargo.toml` 的 `[package] version` 确认改写
  生效，并在**刚改过的那些文件**里核对旧版本号残留（应当为空）。

还没有任何版本 tag 时（首次发版），文档那一步整体跳过：没有「上一个版本」可以替换。

### 2. 提交并打 tag

```bash
git add -A
git commit -m "chore(release): 发布 v0.1.0" -m "- 版本号 0.1.1-dev.0 -> 0.1.0"
just release_tag 0.1.0     # 打 tag v0.1.0，推送 main 与 tag
```

`release_tag` 先检查工作区是否干净，再核对 `Cargo.toml` 的 `[package] version` 与 tag 一致 ——
扩展二进制里的版本号（`make debug` 追加进扩展元数据的 extension version）是构建时写进去的，对不上就会发出一个自称别的版本的 Release。

推送 tag 触发 `.github/workflows/MainDistributionPipeline.yml`：为各平台构建扩展并跑测试，然后为该 tag
创建（或更新）GitHub Release，把构建出的 `.duckdb_extension` 全部挂上去。推 main 本身不构建。

同一个 tag 还会触发 `.github/workflows/DeployDocs.yml`，把 `docs/` 里的文档站构建后发布到 GitHub Pages
（需要先在仓库 Settings → Pages → Build and deployment → Source 里选 **GitHub Actions**，一次性设置）。
不需要文档站就把那个工作流与 `docs/` 一起删掉。

### 3. 等 CI 全绿

```bash
just release_ci            # gh run list --limit 5
gh run watch <run-id>
```

失败就修到成功为止。若已推送的 tag 需要重发（修复后重新指向新的提交）：

```bash
git push origin --delete v0.1.0   # 删除远程 tag
git tag -f v0.1.0                 # 本地 tag 指向修复后的提交
git push origin v0.1.0            # 重新推送
```

> 删除 / 移动已发布的 tag 会影响已有的 GitHub Release，谨慎操作。

网络报错（`schannel: failed to receive handshake`、`SSL connect error` 之类）是**间歇性**的，原样重试
一两次即可，**不要擅自更改网络 / 代理设置**：本机 git 是全局配了代理的，`github.com` 不走代理基本用
不了，动了它反而让 `release_tag` 的推送直接失败。

### 4. 切到下一开发版本

```bash
just release_dev 0.1.1-dev.0
```

这一步只动 `Cargo.toml` 与 `Cargo.lock`：文档与 README 里的示例始终指向最新**已发布**版本，
不打 tag、不发布。

## 相关文档

- [`docs/README.md`](docs/README.md)：文档站的布局、命令、翻译流程与部署。
- [`docs/AGENTS.md`](docs/AGENTS.md)：改文档站（尤其可运行 SQL 块）时的约定与已知坑。
- 开发指南（`docs/docs/development/`，中英各一份）：目录结构、骨架取舍、写函数与测试、构建与发版 ——
  面向改这个仓库的人，替代了模板早期的 `DEVELOPMENT.md`。
- [`README.zh.md`](README.zh.md)（[英文](README.md)）：SQL 接口与使用说明。
- [`scripts/release.sh`](scripts/release.sh) 与 [`scripts/rename.sh`](scripts/rename.sh)：
  `release_bump` / `release_dev` / `release_tag` 与改名的实际实现。
- [`.github/workflows/MainDistributionPipeline.yml`](.github/workflows/MainDistributionPipeline.yml)：
  构建矩阵、触发面与 Release 发布。
- [`community-extension/AGENTS.md`](community-extension/AGENTS.md)：社区扩展注册（上游 `description.yml` 的
  草稿、字段依据、提交 PR 的步骤）。
