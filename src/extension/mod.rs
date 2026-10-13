// 子模块只在 `extension/` 树下挂：src/lib.rs 与 src/wasm_lib.rs 都只写 `mod extension;`，
// 新增模块时不要动那两个 crate root，改这里即可。
//
// Submodules hang off the `extension/` subtree only: both src/lib.rs and src/wasm_lib.rs just declare
// `mod extension;`, so new modules are attached here instead of in the crate roots.
mod functions;
mod types;

use duckfn::duckfn_entrypoint;

// 扩展名：全部小写，仅包含下划线。如果符号缺失或名称错误，DuckDB 将无法加载扩展。
//
// 它必须同时出现在五个地方，且处处一致，否则 LOAD 会失败（产物文件名尤其要紧：DuckDB 是按文件名
// 去找入口点符号的）：
//   - 这里（入口点符号名）
//   - Cargo.toml 的 [package] name 与 [[example]] name
//   - Makefile 的 EXTENSION_NAME
//   - Justfile 的 extension_name
//   - .github/workflows/MainDistributionPipeline.yml 的 extension_name
// scripts/rename.sh 一次改齐这五处（含文档里的出现处），不要手改。
//
// The extension name: all lowercase, underscores only. If the symbol is missing or misnamed, DuckDB
// cannot load the extension. It has to appear in five places at once and match everywhere, or `LOAD`
// fails (the artifact file name matters most: DuckDB looks up the entry-point symbol by it):
//   - here (the entry-point symbol)
//   - Cargo.toml's `[package] name` and `[[example]] name`
//   - `EXTENSION_NAME` in the Makefile
//   - `extension_name` in the Justfile
//   - `extension_name` in .github/workflows/MainDistributionPipeline.yml
// `scripts/rename.sh` rewrites all five (and the occurrences in the docs) in one go; do not edit them
// by hand.
duckfn_entrypoint!("my_extension");
