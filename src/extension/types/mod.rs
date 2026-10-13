// 面向 SQL 的类型放这里：`#[derive(DuckStruct)]` / `#[derive(DuckEnum)]` 定义的自定义类型、用
// `list<struct<…>>` 当返回值的行类型、给函数当可选参数的配置 STRUCT，都归这一层。
//
// 本扩展目前有一个：`open_write_options`（`open_write` / `open_content` 的第 3 个参数）。凡是「三个以下
// 的可选设置」都值得用一个具名 STRUCT 接住 —— 它比一串同类型的可选位置参数好读，也比一段 JSON 好校验
// （字段拼错在 binder 那一层就报错，不必等运行期）。
//
// 挂法（和 functions 一样，往这里加 `mod`）：被 functions 子树读到的类型放宽一档（`pub(crate)`）。
//
// The SQL-facing types live here: custom types defined with `#[derive(DuckStruct)]` /
// `#[derive(DuckEnum)]`, the row type of a function returning `list<struct<…>>`, and the options STRUCT
// a function takes as an optional argument.
//
// There is one today: `open_write_options` (the third argument of `open_write` / `open_content`). Any
// handful of optional settings (three or fewer) is worth catching with a named STRUCT — it reads better
// than a run of optional positional arguments of the same type, and validates better than a JSON string
// (a misspelled field fails at the binder instead of at run time).
//
// Attach submodules here the same way as in `functions`: a type read by the `functions` subtree goes one
// notch more visible (`pub(crate)`).
pub(crate) mod open_write_options;
