// ============================================================================
// 标量函数：把一段**内容**写成一个文件，返回它的路径（不起任何进程）
//
// 这个扩展的其余函数都从「一个路径」出发，而 SQL 探索时手里常是一段内容：渲染好的 HTML、SVG、一份
// CSV、BLOB 里的 PNG 字节。启动器不认内容，所以中间这一步必须有 —— 它就是 `open_write`。
//
//   open_write(content, ext)                 open_write(content, ext, opts)
//   open_write(blob,  ext)                   open_write(blob,  ext, opts)
//
// 第 2 个参数（后缀）**必填**：它决定系统怎么看待这个文件（`.html` 交给浏览器、`.png` 交给图片查看器），
// 猜不得。`dir` / `name` / `part` 三个都可选，装在 STRUCT 里（见 types/open_write_options.rs）：
//
// ```sql
// SELECT open_write(read_text('report.html'), 'html');
// SELECT open_write(svg, 'svg', {'dir':'./out', 'part':'sales'});
// ```
//
// 与 `open_path` 的配合是这个函数存在的另一半理由：它只负责把内容变成路径，剩下那段交给别的函数 ——
//
// ```sql
// SELECT open_path(open_write(html, 'html'));
// SELECT open_path(open_write(html, 'html'), 'firefox');   -- 换程序也照样组合
// ```
//
// 要「写完直接打开」用 `open_content`；要「HTML 直接进浏览器」用 `open_html`。
//
// 四个签名两两一对：VARCHAR 那条按文本写，BLOB 那条按原始字节写（图片这类内容不该被当成 UTF-8 过一遍）。
// 名字相同、选项相同、规则相同，只有内容类型不同。
//
// 入参非 Optional，所以 NULL 行由 duckfn 的参数读取层短路成 NULL：内容没给就一个文件都不该产生。
//
// ============================================================================
// Scalar functions: write a piece of **content** out to a file and return its path (nothing is started).
//
// Every other function in this extension starts from a path, while SQL exploration usually has content
// in hand: a rendered HTML or SVG, a CSV, the PNG bytes of a BLOB. A launcher does not read content, so
// the missing step needs a name of its own — `open_write`.
//
//   open_write(content, ext)                 open_write(content, ext, opts)
//   open_write(blob,  ext)                   open_write(blob,  ext, opts)
//
// The second argument (the extension) is **required**: it decides how the system treats the file
// (`.html` to a browser, `.png` to an image viewer) and is not something to guess. `dir`, `name` and
// `part` are all optional and ride in a STRUCT (see types/open_write_options.rs).
//
// Composing with `open_path` is the other half of why this function exists: it turns content into a
// path and leaves the rest to another function — `open_content` for "open it right away", `open_html`
// for "HTML into the browser".
//
// The four signatures pair up: the VARCHAR ones write text, the BLOB ones write raw bytes (image bytes
// should not be pushed through as UTF-8). Same name, same options, same rules — only the content type
// differs.
//
// The arguments are non-optional, so a NULL row is short-circuited to NULL by duckfn's argument reader:
// with no content there should be no file at all.
// ============================================================================

use duckfn::{DuckBlob, DuckOptionResult, duck_error, duck_scalar_function};

use crate::extension::types::open_write_options::OpenWriteOptions;
use crate::extension::writer;

/// `open_write(content, ext)` / `open_write(content, ext, opts)`：把文本内容落盘，返回路径。
///
/// ```sql
/// SELECT open_write('{"a":1}', 'json');
/// SELECT open_write(svg, 'svg', {'dir':'./out', 'name':'chart'});
/// ```
///
/// 后缀 `html` 与 `.html` 都认（都归成 `.html`），空后缀或带路径分隔符的后缀直接报错。目录缺省是系统
/// 临时目录，给了不存在的目录会连父级一起建出来。没给 `name` 就自动命名
/// `open-<时间>-<part>-<随机尾缀>.<后缀>`（时间与 `part` 在这里拼，随机尾缀与「不覆盖已有文件」由
/// `tempfile` 保证）；给了 `name` 就用它，**同名文件会被覆盖**，没写后缀时补上必填的那个。
///
/// 三个可选项都先过 `sanitize-filename`，所以 `part` 里带空格、斜杠、中文标题都放得进文件名（空格并成
/// `_`，`part` 限 48 字符）。
///
/// 想要缺省就**省略**第 3 个参数：整个 STRUCT 传 NULL 会和内容与后缀一样触发短路（返回 NULL、不落盘），
/// 它不是「其余项按缺省」。
///
/// Writes text content to disk and returns the path. The extension is required (`html` and `.html` are
/// the same thing; an empty one, or one carrying a path separator, is an error), the directory defaults
/// to the system temp directory and is created with its parents when missing. With no `name` the file is
/// generated as `open-<time>-<part>-<random>.<extension>` (the time and `part` assembled here, the
/// random suffix and the nothing-is-overwritten guarantee from `tempfile`); with a `name` that exact path
/// is used and **an existing file is overwritten**, the required extension appended when absent. All
/// three options go through `sanitize-filename` first.
///
/// Omit the third argument for the defaults: a NULL STRUCT short-circuits exactly like a NULL content or
/// extension (NULL back, nothing written) rather than meaning "the rest at their defaults".
#[duck_scalar_function(
    description = "Writes a piece of text or binary content out to a file and returns its path, with the extension required and the directory, file name and name segment optional",
    comment = "Starts nothing: hand the returned path to open_path to open it. `dir` defaults to the \
               system temp directory (created when missing), `name` defaults to a generated \
               open-<time>-<part>-<random> name (an explicit name overwrites), and all three options \
               are sanitized as file names; the BLOB signatures write the bytes exactly as they are",
    examples = [
        "SELECT open_write('{\"a\":1}', 'json')",
        "SELECT open_write(svg, 'svg', {'dir':'./out', 'part':'sales'})"
    ]
)]
pub(super) fn open_write(content: String, extension: String) -> DuckOptionResult<String> {
    write_text(open_write::SQL_NAME, &content, &extension, None)
}

/// `open_write(content, ext, opts)`：同一名字下带选项的签名。
///
/// The signature of the same name that carries the options.
#[duck_scalar_function(overloads_name = "open_write")]
pub(super) fn open_write_with(
    content: String,
    extension: String,
    options: OpenWriteOptions,
) -> DuckOptionResult<String> {
    write_text(open_write_with::SQL_NAME, &content, &extension, Some(&options))
}

/// `open_write(blob, ext)` / `open_write(blob, ext, opts)`：二进制内容（图片这类）的同一件事。
///
/// ```sql
/// -- read_blob 进来的 PNG 字节，落回文件再交给系统看图程序
/// SELECT open_write(read_blob('logo.png'), 'png');
/// ```
///
/// 与文本那条的差别只有一处：字节原样写出去，不经 UTF-8。后缀在这里更加要紧 —— 图片没有后缀，
/// 系统就不知道拿什么程序打开它。规则（目录、命名、合法性、覆盖）与文本签名完全一致。
///
/// The same thing for binary content (an image, say), with one difference: the bytes go out as they are,
/// without passing through UTF-8. The extension matters even more here — without it the system has no
/// idea which program should open the image. Every other rule (directory, naming, legality, overwriting)
/// is identical to the text signatures.
///
/// This overload carries only an `example`, and that is deliberate: `just docs_csv` merges a function set
/// into one row and keeps the **first non-empty** description / comment, but the order it iterates in is
/// the linker's, not the source's. Two overloads both carrying a description is therefore a coin flip on
/// which one reaches the community-extension docs page, so exactly one carries it and the rest add
/// examples (those concatenate and de-duplicate).
#[duck_scalar_function(
    overloads_name = "open_write",
    example = "SELECT open_write(read_blob('logo.png'), 'png')"
)]
pub(super) fn open_write_blob(content: DuckBlob, extension: String) -> DuckOptionResult<String> {
    write_bytes(
        open_write::SQL_NAME,
        &content.value,
        &extension,
        None,
    )
}

/// `open_write(blob, ext, opts)`：二进制内容带选项的签名。
///
/// The signature of the same name that carries the options for binary content.
#[duck_scalar_function(overloads_name = "open_write")]
pub(super) fn open_write_blob_with(
    content: DuckBlob,
    extension: String,
    options: OpenWriteOptions,
) -> DuckOptionResult<String> {
    write_bytes(
        open_write::SQL_NAME,
        &content.value,
        &extension,
        Some(&options),
    )
}

/// 文本 → 文件路径。
///
/// Text to a file path.
fn write_text(
    sql_name: &str,
    content: &str,
    extension: &str,
    options: Option<&OpenWriteOptions>,
) -> DuckOptionResult<String> {
    write_bytes(sql_name, content.as_bytes(), extension, options)
}

/// 四个签名共用的一条写路径：错误加上函数名前缀，好让 SQL 里的报文指回调用点（与 `open_path` 那一族
/// 同一个形状）。
///
/// The one write path behind all four signatures: the error carries the function name so a message from
/// SQL points back at the call site (the same shape as the `open_path` family).
fn write_bytes(
    sql_name: &str,
    content: &[u8],
    extension: &str,
    options: Option<&OpenWriteOptions>,
) -> DuckOptionResult<String> {
    let path = writer::write(content, extension, options)
        .map_err(|err| duck_error(format!("{sql_name}: {err}")))?;

    Ok(Some(path))
}
