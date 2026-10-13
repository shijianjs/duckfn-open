// 注册到 DuckDB 的函数在这里逐个挂上：一个文件一个（或一组）函数，文件名写清「哪一类 + 做什么」。
// 功能长大之后再像 duckfn 示例那样分成子目录（`functions/<功能>/mod.rs` + 各司其职的文件）。
//
// Registered functions are attached here one by one: one file per function (or per small group), with
// the file name saying "which kind + what it does". When a feature outgrows a single file, split it
// into a subdirectory the way the duckfn example does (`functions/<feature>/mod.rs` plus one file per
// concern).
mod aggregate_sum;
mod scalar_greet;
