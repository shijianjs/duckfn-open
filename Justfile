# 新扩展项目的 Justfile 模板 —— 与 AGENTS.md、docs 里的开发指南是同一套流程的几种入口。
#
# 只改一处：把 extension_name 改成你的扩展名（或直接跑 `just rename <新名字>` 让别人替你改）。
# 它必须与下面四处一致，否则 LOAD 会失败：
#   - src/extension/mod.rs 里 duckfn_entrypoint!("...") 的名字
#   - Cargo.toml 的 [package] name
#   - Makefile 里的 EXTENSION_NAME
#   - .github/workflows/MainDistributionPipeline.yml 的 extension_name
#
# 日常命令（build / sql / repl / lint / test / docs_* / ci-* / release_* …）都在
# scripts/common.just 里 —— 那是**共享源**（duckfn 仓库里那份的副本，各扩展项目一份、内容相同），
# 由本文件 import 进来。要改共享内容：改 duckfn 仓库那份，然后 `just sync-common`；
# 要在本项目里覆盖某条共享 recipe：加 `set allow-duplicate-recipes := true` 再重写它。
# 细节见 scripts/common.just 的头注释。`just check-common` 可以比对副本与共享源是否一致。
#
# 前置工具：
#   cargo install just          # recipe 驱动器
#   make + python3              # 官方 extension-ci-tools 构建/测试流程（Windows 上 make 用 Git Bash）
#
# 说明：
#   - 构建走官方 makefile：just build = make configure + make debug；just sql / just repl 加载
#     build/debug/<扩展名>.duckdb_extension。
#   - sqllogictest 与 CI 走同一套（just ci-init 一次，之后 just test）。

# Windows 下 recipe 交给 Git Bash 执行；按自己的 Git 安装路径调整。
# 非 Windows 上这一行不生效。
#
# 这是机器相关的路径，所以留在本地文件里，不放进共享文件。
set windows-shell := ["C:\\Program Files\\Git\\bin\\bash.exe", "-c"]

# 本文件在 import 之后覆盖了同名 recipe（default），需要显式允许重名（浅层覆盖深层）。
set allow-duplicate-recipes := true

import "scripts/common.just"

# 不带参数运行 just 时列出所有 recipe。
# 从 common.just import 进来的 default 不会成为本文件的默认（just 取根文件里第一个 recipe），
# 所以在入口文件这里显式定义一份。
default:
    @just --list

# 扩展名：全小写、只含下划线
extension_name := "duckfn_open"

# 克隆模板后第一件事：把扩展名改掉（Cargo.toml / Makefile / Justfile / extension/mod.rs / CI / 文档）
rename new_name:
    bash scripts/rename.sh "{{new_name}}"

# ==== 本项目排除的共享 recipe ====
#
# 启动器是桌面软件，这个扩展只构建桌面平台：Cargo.toml 里没有 wasm 的 [[example]]，src/extension/launcher.rs
# 也没有 wasm 替身，CI 矩阵把 wasm_mvp / wasm_eh / wasm_threads 排除了（见 AGENTS.md「排除 wasm 构建」）。
# 共享副本 scripts/common.just 里的那三条 recipe 因此只剩一条说明 —— 副本不能改（改了下次 `just sync-common`
# 就没了），本项目里的行为差异按约定在这里覆盖。
#
# This project excludes the shared wasm recipes: a launcher is desktop software, so there is no wasm
# `[[example]]` in Cargo.toml, no wasm stand-ins in src/extension/launcher.rs, and the CI matrix excludes
# the wasm architectures (see AGENTS.md). The copies in scripts/common.just must not be edited — a
# project-local behaviour change is overridden here instead, as the convention says.
[private]
no_wasm:
    @echo "duckfn_open 不构建 wasm / this extension has no wasm build — 启动器是桌面软件 (a launcher is desktop software)"

# 已排除：本扩展只构建桌面平台
build_wasm: no_wasm

# 已排除：本扩展只构建桌面平台
build_wasm_eh: no_wasm

# 已排除：文档站没有可运行 SQL 块，也就没有 wasm 产物要校验
test_wasm: no_wasm
