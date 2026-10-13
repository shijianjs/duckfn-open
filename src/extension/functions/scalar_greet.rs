// ============================================================================
// 示例 1：标量函数（逐行进、逐行出）
//
// 一个标量函数就是「一个普通 Rust 函数 + 一个属性宏」：宏负责把参数读出来、把返回值写回 DuckDB，
// 函数体只写业务逻辑。这里放两个，正好是标量返回形状里最常用的两种：
//
//   -> String               朴素值：永不为 NULL
//   -> DuckOptionResult<T>  可空 + 可报错：Ok(None) 落成 SQL NULL，Err 让整条查询失败
//
// 第三种形状 `-> Option<T>`（可空、但报不了错）没写，用到了照葫芦画瓢即可。
//
// 入参写成不可空的 `String` 时，**NULL 输入由 duckfn 的参数读取层短路成 NULL**（函数体不执行）；
// 想让 NULL 进函数体、自己决定语义，就把参数写成 `Option<String>`。这条规则对标量与聚合一样。
//
// Example 1: scalar functions (row in, row out).
//
// A scalar function is "an ordinary Rust function plus one attribute macro": the macro reads the
// arguments and writes the result back into DuckDB, and the body holds nothing but the logic. Two of
// them live here, covering the two most common scalar return shapes:
//
//   -> String               plain value, never NULL
//   -> DuckOptionResult<T>  nullable and fallible: Ok(None) is SQL NULL, Err fails the whole query
//
// The third shape, `-> Option<T>` (nullable but unable to fail), is not written out — copy the
// pattern when you need it.
//
// With a non-optional argument like `String`, **a NULL input is short-circuited to NULL by duckfn's
// argument reader** (the body never runs); write the parameter as `Option<String>` to let NULL reach
// the body and decide its meaning yourself. The rule is the same for aggregates.
// ============================================================================

use duckfn::{DuckOptionResult, duck_error, duck_scalar_function};

/// 最简形态：一个参数、一个返回值，函数体就是一行。
///
/// ```sql
/// SELECT my_greet('world');
/// -- 'Hello, world!'
/// ```
///
/// `description` / `comment` / `example` 不参与注册：属性宏只把它们连同注册名收进 inventory，供
/// `just docs_csv` 导出 `target/function_descriptions.csv`（社区扩展文档页那张函数表的唯一来源，
/// 见开发指南的「社区扩展」一节）。文案是**英文**，因为它会被原样贴进那个页面。
///
/// The simplest shape: one argument, one return value, one line of body.
///
/// `description` / `comment` / `example` take no part in registration: the macro only collects them
/// (together with the registered name) into an inventory entry so that `just docs_csv` can export
/// `target/function_descriptions.csv` — the one source behind the function table of the
/// community-extension doc page (see the "community extensions" page of the development guide). The
/// text is **English** because it is pasted into that page as it is.
#[duck_scalar_function(
    description = "Greets someone by name, the simplest possible scalar function",
    example = "SELECT my_greet('world')"
)]
fn my_greet(name: String) -> String {
    format!("Hello, {name}!")
}

/// 可空 + 可报错：空串落成 SQL NULL，带首尾空格则报错。
///
/// ```sql
/// SELECT my_greet_checked('world');  -- 'Hello, world!'
/// SELECT my_greet_checked('');       -- NULL
/// SELECT my_greet_checked(' x ');    -- 报错
/// ```
///
/// 错误信息前缀手写成函数名（这里只有一处用到）。同一个名字要在好几个地方出现时（错误提示、日志、
/// 提示文案），改读属性宏生成的 `SQL_NAME` 常量，别抄第二份字面量 —— 代价是函数得写成 `pub(super)`，
/// 因为生成的模块沿用函数的可见性（见开发指南的「写函数」一节）。
///
/// Nullable and fallible: an empty string becomes SQL NULL, surrounding whitespace is an error.
///
/// The error prefix repeats the function name by hand because this is the only place that needs it.
/// Once one name shows up in several places (error messages, logs, hints), read the macro-generated
/// `SQL_NAME` constant instead of copying the literal — the price being that the function has to be
/// `pub(super)`, since the generated module inherits the function's visibility (see the "writing
/// functions" page of the development guide).
#[duck_scalar_function(
    description = "Greets someone by name, returning NULL for an empty name and failing on whitespace",
    comment = "The nullable-and-fallible return shape: Ok(None) is SQL NULL, Err fails the query",
    example = "SELECT my_greet_checked('')"
)]
fn my_greet_checked(name: String) -> DuckOptionResult<String> {
    if name.is_empty() {
        return Ok(None);
    }
    if name.trim() != name {
        return Err(duck_error(
            "my_greet_checked: the name must not have surrounding whitespace",
        ));
    }
    Ok(Some(format!("Hello, {name}!")))
}
