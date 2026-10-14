// ============================================================================
// 标量函数：把一段 HTML 落成临时文件，并用系统默认浏览器打开（零配置的快捷方式）
//
//   op_open_html(content)      →  临时目录里的 open-<时间>-<随机尾缀>.html，返回它的路径
//
// 它就是 `op_open_content(content, 'html')`，只是把三个可选项全部按缺省走：后缀写死成 `html`、目录交给系统
// 临时目录、名字自动生成。存在的理由是「SQL 里渲染出来的东西十有八九是 HTML」，每次都要再敲一遍后缀与
// 一个空 STRUCT 太啰嗦。
//
// ```sql
// SELECT op_open_html(kuva_render('{"series":[…]}'));
// ```
//
// 什么时候不用它：要挑目录、要给文件名、要在名字里留一段认得出「这是哪一份」的记号（`part`），或者要换
// 一个非默认的浏览器 —— 那些都是 `op_open_content(content, 'html', opts)` 与
// `op_open_path(op_write_file(html, 'html'), 'firefox')` 的事。
//
// BLOB 没有对应的签名：这里的输入是 HTML **文本**，二进制那一路请先
// `CAST(blob AS VARCHAR)`（或者干脆走 `op_open_content(blob, 'html')`）。
//
// ============================================================================
// Scalar function: write an HTML string to a temporary file and open it in the system default browser —
// the zero-configuration shortcut.
//
// It is `op_open_content(content, 'html')` with every option left at its default: the extension is fixed to
// `html`, the directory is the system temp directory, the name is generated. It exists because what SQL
// renders is HTML nine times out of ten, and typing the extension plus an empty STRUCT each time is
// noise.
//
// When not to use it: to pick a directory, name the file, leave a recognisable `part` in the name, or use
// a non-default browser — those are `op_open_content(content, 'html', opts)` and
// `op_open_path(op_write_file(html, 'html'), 'firefox')`.
//
// There is no BLOB signature: the input here is HTML *text*. For the binary case, cast first
// (`CAST(blob AS VARCHAR)`) or go through `op_open_content(blob, 'html')`.
// ============================================================================

use duckfn::{DuckOptionResult, duck_error, duck_scalar_function};

use crate::extension::launcher;
use crate::extension::writer;

/// 这个快捷方式写死的后缀。
///
/// The extension this shortcut pins.
const HTML_EXTENSION: &str = "html";

/// `op_open_html(content)`：落成临时 `.html` 并在默认浏览器里打开，返回那个路径。
///
/// ```sql
/// SELECT op_open_html('<h1>hello</h1>');
/// SELECT op_open_html(string_agg(row, chr(10))) FROM rendered_rows t(row);
/// ```
///
/// 与 `op_open_content` 同一套：不等浏览器退出，起不来就报错并且报文里带着路径（文件已经写出来了，只是这台
/// 机器没有桌面 —— CI 的 runner 正是这种）。NULL 行整行短路，一个文件都不会产生。
///
/// Same contract as `op_open_content`: it does not wait for the browser to exit, and a launcher that cannot
/// start is an error carrying the path (the file is written; this machine has no desktop, which is exactly
/// what a CI runner is). A NULL row short-circuits and produces no file.
#[duck_scalar_function(
    description = "Writes an HTML string to a temporary file and opens it in the system default browser, returning the file path",
    comment = "The zero-configuration shortcut: op_open_content(content, 'html') with every option left at \
               its default (system temp directory, generated name); needs no configuration because SQL \
               renders HTML",
    examples = [
        "SELECT op_open_html('<h1>hello</h1>')",
        "SELECT op_open_html(read_text('report.html'))"
    ],
    volatile = true
)]
pub(super) fn op_open_html(content: String) -> DuckOptionResult<String> {
    let path = writer::write(content.as_bytes(), HTML_EXTENSION, None)
        .map_err(|err| duck_error(format!("{}: {err}", op_open_html::SQL_NAME)))?;

    launcher::open_default_checked(op_open_html::SQL_NAME, &path)?;

    Ok(Some(path))
}
