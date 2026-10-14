// ============================================================================
// 标量函数：把一段**内容**写成一个文件，并立刻用系统默认程序打开
//
// `op_write_file` 与 `op_open_path` 的组合就是这个函数做的事，但那是探索时最常要做的一件事，值得一个名字：
//
//   op_open_content(content, ext)              op_open_content(content, ext, opts)
//   op_open_content(blob,  ext)                op_open_content(blob,  ext, opts)
//
// ```sql
// SELECT op_open_content(svg, 'svg', {'dir':'./out', 'part':'sales'});
// ```
//
// 与 `op_write_file(...)` 再套一层 `op_open_path(...)` 的区别只有一处 —— 错误放在哪儿：
//
// - `op_open_path` 的返回值就是它的结论，所以启动器起不来是一个 `false`；
// - 这个名字的返回值已经是写好的路径了，启动失败再换成 `false` 就把路径丢了。于是它**报错**，并且
//   报文里带着那个路径：文件确实在磁盘上，只是这台机器（CI 的 runner 就没有桌面）没能把它叫起来。
//
// 「写完就打开」走的是**不等**的那条路（`open::that_detached`）：一次查询不该被「等浏览器关掉」挂住。
// 要等启动器的结果，用 `op_open_path(op_write_file(...))`。
//
// 指定程序不在这个函数的事：`op_open_path(op_write_file(content, ext, opts), 'firefox')` 已经写得出同一件事，
// 所以选项 STRUCT 里刻意没有 `app`（见 types/op_write_file_options.rs）。
//
// `op_open_html` 是这个函数在 HTML 上的零配置特例。
//
// ============================================================================
// Scalar functions: write a piece of content to a file and open it with the default program at once.
//
// Composing `op_write_file` with `op_open_path` does exactly this, but it is the single most common thing to do
// while exploring, so it earns a name of its own.
//
// The only difference from that composition is where the error goes: `op_open_path`'s return value *is* its
// verdict, so a launcher that cannot start is a `false`, while this name spends its return value on the
// path it wrote — turning a launch failure into `false` there would throw the path away. So it reports an
// error, carrying the path: the file really is on disk, this machine just could not start a program for
// it (a CI runner has no desktop).
//
// "Open it right away" takes the non-blocking route (`open::that_detached`): a query should not hang on
// waiting for a browser to be closed. Use `op_open_path(op_write_file(...))` to get the launcher's outcome.
//
// Choosing the program is not this function's business — `op_open_path(op_write_file(content, ext, opts),
// 'firefox')` already says it, which is why the options STRUCT deliberately has no `app` field.
//
// `op_open_html` is this function's zero-configuration special case for HTML.
// ============================================================================

use duckfn::{DuckBlob, DuckOptionResult, duck_error, duck_scalar_function};

use crate::extension::launcher;
use crate::extension::types::op_write_file_options::OpWriteFileOptions;
use crate::extension::writer;

/// `op_open_content(content, ext)` / `op_open_content(content, ext, opts)`：文本落盘后用默认程序打开。
///
/// ```sql
/// SELECT op_open_content(read_text('report.html'), 'html');
/// SELECT op_open_content(svg, 'svg', {'dir':'./out', 'name':'chart'});
/// ```
///
/// 返回写出来的路径，落盘的规则与 `op_write_file` 完全一致（后缀必填、目录可缺省、命名与合法性同一套）。
/// 起不来时报文形如 `op_open_content: the file was written to '…', but no launcher could be started: …`。
///
/// Returns the path it wrote, with exactly the same persistence rules as `op_write_file` (required
/// extension, optional directory, one set of naming and legality rules). When no launcher starts, the
/// message reads `op_open_content: the file was written to '…', but no launcher could be started: …`.
#[duck_scalar_function(
    description = "Writes a piece of text or binary content out to a file and opens it with the program the desktop is configured to use",
    comment = "op_write_file plus op_open_path in one call: it returns the path it wrote, so a launcher that \
               cannot start is reported as an error carrying that path instead of a lost false; it does \
               not wait for the program to exit, and the BLOB signatures write the bytes exactly as they \
               are",
    examples = [
        "SELECT op_open_content(read_text('report.html'), 'html')",
        "SELECT op_open_content(svg, 'svg', {'dir':'./out', 'part':'sales'})"
    ]
)]
pub(super) fn op_open_content(content: String, extension: String) -> DuckOptionResult<String> {
    write_and_open(op_open_content::SQL_NAME, content.as_bytes(), &extension, None)
}

/// `op_open_content(content, ext, opts)`：同一名字下带选项的签名。
///
/// The signature of the same name that carries the options.
#[duck_scalar_function(overloads_name = "op_open_content")]
pub(super) fn op_open_content_with(
    content: String,
    extension: String,
    options: OpWriteFileOptions,
) -> DuckOptionResult<String> {
    write_and_open(
        op_open_content_with::SQL_NAME,
        content.as_bytes(),
        &extension,
        Some(&options),
    )
}

/// `op_open_content(blob, ext)` / `op_open_content(blob, ext, opts)`：二进制内容的同一件事。
///
/// ```sql
/// -- BLOB 里的一张图：落回文件，交给系统的看图程序
/// SELECT op_open_content(logo_blob, 'png');
/// ```
///
/// The binary signatures of the same name: the bytes go out as they are, then the default program for
/// that extension is started.
///
/// Only an `example` here, as in `op_write_file`'s BLOB overload: a function set merges into one docs-page
/// row that keeps the first non-empty description, and the order `inventory` iterates in is the linker's
/// rather than the source's, so two descriptions in one set decide the text by accident.
#[duck_scalar_function(
    overloads_name = "op_open_content",
    example = "SELECT op_open_content(logo_blob, 'png')"
)]
pub(super) fn op_open_content_blob(content: DuckBlob, extension: String) -> DuckOptionResult<String> {
    write_and_open(op_open_content::SQL_NAME, &content.value, &extension, None)
}

/// `op_open_content(blob, ext, opts)`：二进制内容带选项的签名。
///
/// The signature of the same name that carries the options for binary content.
#[duck_scalar_function(overloads_name = "op_open_content")]
pub(super) fn op_open_content_blob_with(
    content: DuckBlob,
    extension: String,
    options: OpWriteFileOptions,
) -> DuckOptionResult<String> {
    write_and_open(
        op_open_content::SQL_NAME,
        &content.value,
        &extension,
        Some(&options),
    )
}

/// 落盘，然后把默认程序叫起来；两步共用一条路径，所以错误前缀（`SQL_NAME`）只写一次。
///
/// Write the file, then start the default program for it. Both steps share one path, so the error prefix
/// (`SQL_NAME`) is written once.
fn write_and_open(
    sql_name: &str,
    content: &[u8],
    extension: &str,
    options: Option<&OpWriteFileOptions>,
) -> DuckOptionResult<String> {
    let path =
        writer::write(content, extension, options).map_err(|err| duck_error(format!("{sql_name}: {err}")))?;

    launcher::open_default_checked(sql_name, &path)?;

    Ok(Some(path))
}
