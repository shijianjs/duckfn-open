/// 原生 crate root：只挂 `mod extension;`，子模块都在 `extension/mod.rs` 下面挂。
///
/// 与 wasm_lib.rs 保持同一组 mod，解决官方模板 `mod lib;` 方式在嵌套模块时路径不一致、
/// 无法嵌套的问题：
/// error[E0583]: file not found for module functions --> src\lib.rs:3:1
///
/// The native crate root: it only declares `mod extension;`, and `extension/mod.rs` attaches the
/// submodules from there. Keeping the same set of `mod`s as `wasm_lib` is what fixes the official
/// template's `mod lib;` approach, which breaks as soon as modules are nested
/// (`error[E0583]: file not found for module ...`).
mod extension;
