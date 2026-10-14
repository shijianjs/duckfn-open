// ============================================================================
// 标量函数：把 sanitize-filename 这个库的能力原样接到 SQL 上
//
//   op_sanitize_filename(value[, replacement[, truncate, windows]])  →  VARCHAR
//   op_sanitize_filename_check(value[, truncate, windows])           →  BOOLEAN
//
// 这个库维护的是「一个字符串能不能当文件名」：非法字符 `/ \ ? < > : * | "`、控制字符、Windows 的保留设备名
// （`CON` / `NUL` / `COM1`…）、结尾的点与空格，以及 255 字节这一刀。落盘那一族内部也用它，但**这两个函数不是
// `op_write_file` 的助手**：后面的三个参数就是库里 `Options` 的字段，缺省也跟着库走（`replacement` 是空串，也就是
// **删掉**；`truncate` 是开），唯一例外是 `windows` —— 库里它的缺省是编译平台，那会让同一条 SQL 在不同机器上
// 给出不同答案，所以这里钉成 `true`（最严的一套，三个平台同形）；要 Unix 那套规则就显式传 `windows`。
//
// 为什么值得单独给 SQL：查询里的文件名常常来自数据本身 —— 客户名、报表标题、抓回来的那一列 path。先把它变成
// 一个名字，才谈得上拼目录、拼 URL、交给别的扩展；`op_sanitize_filename_check` 是不改写的那一半，回答「这个名字本来就
// 合法吗」，用于过滤和体检（`WHERE NOT op_sanitize_filename_check(name)`），而不是悄悄把值换掉。
//
// Example: what sanitize-filename maintains is whether a string can be a file name: the illegal characters
// `/ \ ? < > : * | "`, the control characters, the Windows reserved device names (`CON` / `NUL` / `COM1`…),
// trailing dots and spaces, and the 255-byte cut. The write family uses the same crate internally, but
// **these two functions are not helpers of `op_write_file`**: the trailing arguments are the crate's own
// `Options` fields and the defaults follow the crate (`replacement` is the empty string, i.e. characters are
// *deleted*; `truncate` is on). The one exception is `windows`, whose default in the crate is the compiling
// platform — that would make one SQL statement answer differently on different machines, so it is pinned to
// `true` here (the strictest set, the same answer on all three platforms); pass `windows` explicitly for the
// Unix rules.
//
// Why SQL deserves them: in a query a file name usually comes from the data itself — a customer name, a
// report title, the `path` column that was crawled in. Turning it into a name is what makes the directory,
// the URL and the next extension composable; `op_sanitize_filename_check` is the half that rewrites nothing, it
// answers "was this already a usable name" so a query can filter and audit instead of silently substituting.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

/// 这条函数族的缺省里唯一不跟库走的那一项：`windows` 检查永远是开的。
///
/// The one default that does not follow the crate: the Windows checks are always on.
const WINDOWS_BY_DEFAULT: bool = true;

/// 库里那两个入口在这里的唯一调用点：把三个参数摊平成一次改写。
///
/// The only place the crate's two entry points are called: three arguments flattened into one rewrite.
fn sanitize(raw: &str, replacement: &str, truncate: bool, windows: bool) -> String {
    sanitize_filename::sanitize_with_options(
        raw,
        sanitize_filename::Options {
            windows,
            truncate,
            replacement,
        },
    )
}

/// 不改写的那一半：这个名字经过 `sanitize` 后是否还是它自己。
///
/// The half that rewrites nothing: whether this name would still be itself after `sanitize`.
fn is_usable(raw: &str, truncate: bool, windows: bool) -> bool {
    sanitize_filename::is_sanitized_with_options(
        raw,
        sanitize_filename::OptionsForCheck {
            windows,
            truncate,
        },
    )
}

/// `op_sanitize_filename(value)`：按库的缺省改写成一个可用的文件名（非法字符被**删掉**）。
///
/// ```sql
/// SELECT op_sanitize_filename('quarterly/report:v2.csv');   -- 'quarterlyreportv2.csv'
/// SELECT op_sanitize_filename('  the report.  ');           -- 结尾的点与空格被去掉
/// ```
///
/// 想要「看得见被改过」的结果，把替换符给上：`op_sanitize_filename(title, '_')`。这一族与 `op_write_file` 的 `name`
/// 那条策略（替换成 `_`、不截断）是两套缺省，各自写在文档里，别把其中一个当成另一个。
///
/// For a result that shows where it was edited, pass the substitution: `op_sanitize_filename(title, '_')`. These
/// defaults are not `op_write_file`'s `name` policy (which substitutes `_` and never truncates); the two are
/// documented separately, so neither is a stand-in for the other.
#[duck_scalar_function(
    description = "Rewrites a string into something usable as a file name, following sanitize-filename",
    comment = "The crate's own defaults: the illegal characters are deleted (pass a replacement to see where \
               they were), the 255-byte cut is applied, and the Windows rules are on because a query should \
               answer the same way on every machine",
    examples = [
        "SELECT op_sanitize_filename('quarterly/report:v2.csv')",
        "SELECT op_sanitize_filename(report_title, '_')"
    ]
)]
pub(super) fn op_sanitize_filename(value: String) -> DuckOptionResult<String> {
    Ok(Some(sanitize(
        &value,
        "",
        true,
        WINDOWS_BY_DEFAULT,
    )))
}

/// `op_sanitize_filename(value, replacement)`：同样改写，但非法字符换成给定的那一段。
///
/// ```sql
/// SELECT op_sanitize_filename('a/b:c.txt', '_');    -- 'a_b_c.txt'
/// SELECT op_sanitize_filename('sales|east', '-');   -- 'sales-east'
/// ```
///
/// 替换符可以是空串（与一条参数的写法相同），也可以是多个字符。NULL 行整行短路成 NULL，与这个扩展里其它
/// 函数一个规矩。
///
/// The substitution may be empty (the same as the one-argument form) or several characters long. A NULL row
/// short-circuits to NULL, as everywhere else in this extension.
#[duck_scalar_function(overloads_name = "op_sanitize_filename")]
pub(super) fn op_sanitize_filename_with_replacement(
    value: String,
    replacement: String,
) -> DuckOptionResult<String> {
    Ok(Some(sanitize(
        &value,
        &replacement,
        true,
        WINDOWS_BY_DEFAULT,
    )))
}

/// `op_sanitize_filename(value, replacement, truncate, windows)`：三个开关全给出来。
///
/// ```sql
/// SELECT op_sanitize_filename(x, '_', false, false);  -- 不截断，也不套 Windows 那套保留名（`CON.txt` 留着）
/// SELECT op_sanitize_filename(x, '_', true, true);    -- 库最严的一套：255 字节一刀，Windows 规则全开
/// ```
///
/// 四个参数是一条调用，不是两次 —— `truncate` 与 `windows` 只能跟在一起给，这是这份 API 里唯一的一处压缩：
/// 两个 BOOLEAN 摆在可选位置上有五种组合，而其中只有「按库的两种缺省」值得写在两行的形式里。缺省的
/// `windows` 是 `true`（三个平台同形），显式传 `false` 才是 Unix 的那一套。
///
/// The four arguments are one call, not two — `truncate` and `windows` can only be passed together, the one
/// compression in this API: two BOOLEANs in optional positions would be five combinations of which only the
/// crate's two defaults deserve a two-argument form. `windows` defaults to `true` (one answer on every
/// platform); pass `false` for the Unix rules.
#[duck_scalar_function(overloads_name = "op_sanitize_filename")]
pub(super) fn op_sanitize_filename_with_options(
    value: String,
    replacement: String,
    truncate: bool,
    windows: bool,
) -> DuckOptionResult<String> {
    Ok(Some(sanitize(&value, &replacement, truncate, windows)))
}

/// `op_sanitize_filename_check(value)`：这个名字本来就合法吗（什么都不改写）。
///
/// ```sql
/// SELECT op_sanitize_filename_check('report.csv');    -- true
/// SELECT op_sanitize_filename_check('CON.txt');       -- false
/// SELECT count(*) FROM files WHERE NOT op_sanitize_filename_check(name);   -- 有几条需要改
/// ```
///
/// `true` 的意思是在同样的开关下 `op_sanitize_filename` 会原样返回这个值。它是「体检」而不是「改写」的那一半：
/// 想知道一批数据里有多少名字会被动到，又不想在这一层就改掉它们。
///
/// `true` means the same call would return the value unchanged. This is the audit half rather than the
/// rewriting half: how many of these names would be touched, without touching them here.
#[duck_scalar_function(
    description = "Tells whether a string is already usable as a file name, rewriting nothing",
    comment = "The predicate half of sanitize-filename: true means op_sanitize_filename with the same options would \
               answer with the value it was given",
    example = "SELECT count(*) FROM files WHERE NOT op_sanitize_filename_check(name)"
)]
pub(super) fn op_sanitize_filename_check(value: String) -> DuckOptionResult<bool> {
    Ok(Some(is_usable(&value, true, WINDOWS_BY_DEFAULT)))
}

/// `op_sanitize_filename_check(value, truncate, windows)`：把这两个开关交出来。
///
/// ```sql
/// SELECT op_sanitize_filename_check('CON.txt', true, false);     -- true：Unix 的规则里没有保留设备名
/// SELECT op_sanitize_filename_check(x, false, true);             -- 超过 255 字节也算合法
/// ```
///
/// 这里没有 `replacement`：一个都不改写的问题，没有「换成什么」这一项。
///
/// There is no `replacement` here: a question that rewrites nothing has nothing to substitute.
#[duck_scalar_function(overloads_name = "op_sanitize_filename_check")]
pub(super) fn op_sanitize_filename_check_with_options(
    value: String,
    truncate: bool,
    windows: bool,
) -> DuckOptionResult<bool> {
    Ok(Some(is_usable(&value, truncate, windows)))
}
