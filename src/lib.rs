/// Crate root：只挂 `mod extension;`，其余模块都在 `extension/mod.rs` 下面挂。
///
/// 官方模板在这里写的是 `mod lib;`，再由第二个 crate root 转发一遍；模块一嵌套就会因为两条路径对不上而
/// 编译失败（`error[E0583]: file not found for module ...`），所以本仓库不保留那份转发。
///
/// The crate root declares one module, `mod extension;`, and `extension/mod.rs` attaches everything
/// below it. The official template writes `mod lib;` here and forwards it a second time from another
/// crate root, which breaks as soon as modules nest (`error[E0583]: file not found for module ...`), so
/// that second copy is not kept in this repository.
mod extension;
