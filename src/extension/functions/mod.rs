// 注册到 DuckDB 的函数在这里逐个挂上：一个文件一个（或一组）函数，文件名写清「哪一类 + 做什么」。
// 功能长大之后再像 duckfn 示例那样分成子目录（`functions/<功能>/mod.rs` + 各司其职的文件）。
//
// 这一层只放「带 `#[duck_*]` 的属性宏、会被 SQL 调到」的东西；把平台差异收起来的那一层在
// `extension/launcher.rs`，函数文件通过它调 open，不直接 `use open::`。
//
// Registered functions are attached here one by one: one file per function (or per small group), with
// the file name saying "which kind + what it does". When a feature outgrows a single file, split it
// into a subdirectory the way the duckfn example does (`functions/<feature>/mod.rs` plus one file per
// concern).
//
// This layer holds only what carries a `#[duck_*]` attribute and is reachable from SQL; the layer that
// absorbs the platform differences is `extension/launcher.rs`, which the files below call instead of
// using `open::` directly.
mod scalar_open_commands;
mod scalar_open_content;
mod scalar_open_html;
mod scalar_open_path;
mod scalar_open_print;
mod scalar_open_write;
