use duckfn::DuckStruct;

// ============================================================================
// `open_write_options`：把内容落盘时的三个可选项
//
// `open_write` / `open_content` 的第 3 个参数。内容（VARCHAR 或 BLOB）与后缀（必填的第 2 个参数）
// 之外，剩下三件事都放这里，于是这个函数永远只有两种 arity：
//
// ```sql
// SELECT open_write(html, 'html');                                    -- 全默认
// SELECT open_write(html, 'html', {'dir':'./out','part':'sales'});    -- 只挑两个选项
// ```
//
// 三个字段**全部**是 `Option<T>`，这是硬要求：DuckDB 的 struct 字面量缺字段时补 NULL，而 duckfn 读到
// 「非 Option 字段为 NULL」会让**整个 struct** 变成 NULL —— 那样 `{'part':'sales'}` 会整体退化成
// 「一个选项都没给」，用户给的那个 part 被静默丢掉。
//
// 字段本身不在这里校验（空串、非法字符都在 `extension/writer.rs` 处理），这一层只声明形状。
// 落盘的形状归 writer.rs，起进程的形状归 launcher.rs。
//
// ----------------------------------------------------------------------------
// `open_write_options`: the three optional settings for writing content out.
//
// The third argument of `open_write` / `open_content`. Everything except the content (VARCHAR or BLOB)
// and the extension (the required second argument) lives here, so those functions have exactly two
// arities.
//
// Every field is an `Option<T>` on purpose: DuckDB fills the missing keys of a struct literal with
// NULL, and duckfn turns the **whole struct** into NULL when a non-Option field reads NULL — so
// `{'part':'sales'}` would silently degrade to "no options at all" and the caller's `part` would be
// dropped.
//
// No validation happens here (empty strings and illegal characters are `extension/writer.rs`'s
// business); this layer only states the shape. Writing goes through writer.rs, launching through
// launcher.rs.
// ============================================================================

/// 内容落盘的可选项，字段名即 SQL 里的键名。
///
/// 刻意**不含**「用哪个程序打开」：那是 `open_path(path, app)` 的事，把路径交给它是组合的用法，
/// 而不是每个写函数再抄一遍启动参数。
///
/// The optional settings for writing content out; the Rust field names are the keys used in SQL.
///
/// It deliberately holds no "which program opens it": that is `open_path(path, app)`'s job, and handing
/// it the path is the composable way to say so, instead of every writer copying the launcher arguments.
#[derive(Clone, Debug, Default, DuckStruct)]
#[duck(sql_name = "open_write_options", create_type = "replace")]
pub(crate) struct OpenWriteOptions {
    /// 输出目录。缺省用系统临时目录；目录不存在时递归创建。
    ///
    /// The output directory. The system temp directory when unset; created (with any missing parents)
    /// when it does not exist yet.
    pub dir: Option<String>,

    /// 文件名。给了就用它（同名文件被覆盖），缺省自动命名 `open-<时间>-<部分>-<随机尾缀>.<后缀>`。
    ///
    /// The file name. Used as given when set (an existing file of that name is overwritten); otherwise
    /// the name is generated as `open-<time>-<part>-<random>.<extension>`.
    pub name: Option<String>,

    /// 自动名字里「有意义的那一段」（例如图表标题），用来认文件；只在没给 `name` 时参与命名。
    ///
    /// The meaningful segment of a generated name (a chart title, say) — the part that makes the file
    /// recognisable. It only takes part in naming when `name` is unset.
    pub part: Option<String>,
}
