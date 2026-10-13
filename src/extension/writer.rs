use duckfn::{DuckResult, duck_error};

use crate::extension::types::open_write_options::OpenWriteOptions;

// ============================================================================
// 落盘层：把一段**内容**（VARCHAR 或 BLOB 的字节）写成一个文件，返回它的路径
//
// 这个扩展的启动器只认路径、不认内容，而 SQL 里躺着的是内容（渲染好的 HTML、SVG、一段 CSV、BLOB 里的
// PNG 字节）。这一层就是补上这一步：内容 → 文件 → 路径，之后路径可以交给 `open_path` 那一族，也可以由
// `open_content` / `open_html` 直接接着打开。
//
// 只有一个入口 [`write`]，形状是 `(字节, 后缀, 选项)`：后缀必填，`dir` / `name` / `part` 三个都可选
// （见 types/open_write_options.rs）。规则如下：
//
// | 选项 | 缺省 | 给了之后 |
// | --- | --- | --- |
// | `dir` | 系统临时目录 | 用它，不存在时递归创建 |
// | `name` | 自动命名 | 用它，同名文件**被覆盖**，没写后缀时补上 |
// | `part` | 不参与命名 | 拼进自动名字里，作为「这是哪一份」的记号 |
//
// 自动命名交给 `tempfile`：`open-<时间>-<part>-<随机尾缀>.<后缀>`，时间与 `part` 由这里拼，随机尾缀、
// 「这个名字当时是空的」以及临时目录本身都由 `tempfile` 保证（AGENTS.md「尽量用成熟三方库」）。
//
// 三个用户可填的字段（`dir` / `name` / `part`）一律先过 `sanitize-filename` —— 非法字符、控制字符、
// Windows 保留设备名、结尾的点与空格都是它的规则；这里只补三条本项目自己的策略（空格并成 `_`、`part`
// 限长、整段被替换光就报错）。
//
// 返回的路径**一律用 `/` 拼**（见 [`join`]）：`Path::join` 按平台分隔符拼，同一个返回值在 Windows 上会
// 是 `out\x.html`、在别处是 `out/x.html`，而这个字符串要打进终端、要被下游接着拼、要被测试盯着形状。
// `std::fs` 与 `open` 在 Windows 上都接受 `/`。
//
// 这一层不起进程（那是 launcher.rs 的事），也不碰 wasm：本扩展只构建桌面平台（见根目录 AGENTS.md），
// 所以这里没有任何 `#[cfg]`。
//
// ----------------------------------------------------------------------------
// The persistence layer: write a piece of **content** (the bytes of a VARCHAR or a BLOB) to a file and
// return its path.
//
// The launchers in this extension understand paths, not content, while what SQL holds is content (a
// rendered HTML or SVG, a CSV, the PNG bytes of a BLOB). This layer is that missing step: content →
// file → path, after which the path goes to the `open_path` family, or straight to a launcher through
// `open_content` / `open_html`.
//
// There is one entry point, [`write`], shaped `(bytes, extension, options)`: the extension is required
// and `dir` / `name` / `part` are all optional (see types/open_write_options.rs).
//
// | option | default | when given |
// | --- | --- | --- |
// | `dir` | the system temp directory | used as given, created (with parents) when missing |
// | `name` | generated | used as given, an existing file of that name **is overwritten**, the extension appended when absent |
// | `part` | not used | appended to a generated name, as the "which one is this" marker |
//
// Naming is `tempfile`'s job: `open-<time>-<part>-<random>.<extension>`, where the time and `part` are
// assembled here and the random suffix, "this name was free at that moment" and the temp directory
// itself are all guaranteed by it.
//
// All three caller-supplied fields go through `sanitize-filename` first — illegal characters, control
// characters, Windows reserved device names and trailing dots and spaces are its rules. Only three
// policies are this project's own: spaces become `_`, `part` is length-capped, and a field that turns
// into nothing usable is an error.
//
// The returned path is always joined with `/` (see [`join`]): `Path::join` uses the platform separator,
// so one return value would read `out\x.html` on Windows and `out/x.html` elsewhere, while this string
// is printed into a terminal, concatenated further, and asserted on its shape by the tests. Both
// `std::fs` and `open` accept `/` on Windows.
//
// Nothing is launched here (that is launcher.rs) and wasm is not involved: this extension builds for
// desktop targets only (see AGENTS.md), so there is no `#[cfg]` in this file.
// ============================================================================

/// 自动命名的开头一段，用来在目录里认出「这是本扩展写的文件」。
///
/// The leading segment of a generated name — what marks a file in a directory as one this extension
/// wrote.
const AUTO_PREFIX: &str = "open";

/// `part` 在自动名字里最多保留的字符数。
///
/// 名字里已经有时间、随机尾缀与后缀，一段任意长度的标题很容易把整条文件名顶到 255 字符的上限。
///
/// How many characters of `part` may end up in a generated name. The name already carries the time, the
/// random suffix and the extension, and a title of unbounded length easily pushes the whole file name
/// into the 255-character limit.
const MAX_PART_CHARS: usize = 48;

/// 把内容写成文件，返回它的路径；`content` 是原始字节，VARCHAR 与 BLOB 两条路都走到这里来。
///
/// `extension` 是必填的那个后缀（`html` 与 `.html` 都认，见 [`normalize_extension`]）；`options` 是没传
/// 第 3 个参数时的 `None`。
///
/// Write the content to a file and return its path. `content` is raw bytes, which is where both the
/// VARCHAR and the BLOB path land.
///
/// `extension` is the required suffix (both `html` and `.html` are understood, see
/// [`normalize_extension`]); `options` is `None` when no third argument was passed.
pub(crate) fn write(
    content: &[u8],
    extension: &str,
    options: Option<&OpenWriteOptions>,
) -> DuckResult<String> {
    let extension = normalize_extension(extension)?;
    let dir = resolve_dir(options)?;

    let path = match option_field(options, |opts| opts.name.as_ref()) {
        // 用户点名了文件：那就是他要的路径，同名文件按覆盖处理。
        Some(name) => join(&dir, &sanitize_name(name, &extension)?),
        // 没点名：交给 tempfile 新建一个不重名的文件，只把它给出的文件名拼回这个目录。
        None => unique_name(&dir, &extension, option_field(options, |opts| opts.part.as_ref()))?,
    };

    std::fs::write(&path, content)
        .map_err(|err| duck_error(format!("cannot write the file to '{path}': {err}")))?;

    Ok(path)
}

/// 取 `OpenWriteOptions` 里的一个字段，顺带把「整个 struct 是 NULL」和「没传第 3 个参数」两种情况并成
/// 同一条 `None`。
///
/// Read one field of `OpenWriteOptions`, folding "the whole struct is NULL" and "no third argument was
/// passed" into the same `None`.
fn option_field<T>(options: Option<&OpenWriteOptions>, get: impl Fn(&OpenWriteOptions) -> Option<&T>) -> Option<&T> {
    options.and_then(get)
}

/// 输出目录：`dir` 给了就用它，否则用系统临时目录。两种情况都把目录建出来 —— 「我要把文件放这儿」
/// 通常意味着这个目录还没在，而不是已经在了。
///
/// 空串在这里就报掉：它既不是目录也不是相对路径的写法，交给文件系统只会换来一条不说清问题的错误。
///
/// The output directory: `dir` when given, the system temp directory otherwise. Either way the directory
/// is created — "this is where I want the file" usually means it is not there yet, not that it is.
///
/// An empty string fails right here: it is neither a directory nor a way of writing a relative path, and
/// the file system would only answer with an error that says nothing useful.
fn resolve_dir(options: Option<&OpenWriteOptions>) -> DuckResult<String> {
    let dir = match option_field(options, |opts| opts.dir.as_ref()) {
        Some(dir) if dir.trim().is_empty() => {
            return Err(duck_error("open_write_options.dir must not be empty"));
        }
        Some(dir) => dir.to_string(),
        None => std::env::temp_dir().to_string_lossy().into_owned(),
    };

    std::fs::create_dir_all(&dir)
        .map_err(|err| duck_error(format!("cannot create the output directory '{dir}': {err}")))?;

    Ok(dir)
}

/// 规范化必填的后缀：`html`、`.html`、` HTM ` 都归一成 `.html` 这样的「点 + 后缀」。
///
/// 三条策略：空后缀报错（后缀是决定系统怎么看待这个文件的，缺了就没有可猜的余地）；含路径分隔符报错
/// （传进来的多半是一整条路径，静默替换成 `_` 会让它写出一个谁也不认识的后缀）；其余交给
/// `sanitize-filename`（和 `name` / `part` 同一套合法性规则）。
///
/// Normalise the required extension: `html`, `.html` and ` HTM ` all come back as a `.html`-shaped
/// "dot plus extension".
///
/// Three policies: an empty extension is an error (the extension is what decides how the system treats
/// the file, so there is nothing to guess from); one carrying a path separator is an error (what was
/// passed is most likely a whole path, and silently replacing the separator would produce an extension
/// nobody recognises); everything else goes through `sanitize-filename` (the same legality rules as
/// `name` and `part`).
fn normalize_extension(raw: &str) -> DuckResult<String> {
    let stem = raw.trim().trim_start_matches('.').trim();

    if stem.is_empty() {
        return Err(duck_error("the extension must not be empty (pass e.g. 'html', not '')"));
    }
    if stem.contains(['/', '\\']) {
        return Err(duck_error(format!(
            "the extension '{raw}' looks like a path: pass only the suffix, e.g. 'html'"
        )));
    }

    let cleaned = sanitize(stem);
    if cleaned.is_empty() {
        return Err(duck_error(format!("'{raw}' is not a usable extension")));
    }

    Ok(format!(".{cleaned}"))
}

/// 把用户给的 `name` 变成这个目录里一个可用的文件名，并在它没写后缀时补上必填的那个。
///
/// 合法性交给 [`sanitize`]；之后是本项目的一条策略：整段都被替换掉（`"/"`、`".."` 这类）说明没有可用
/// 的名字，直接报错。补后缀按**不区分大小写**比较，`REPORT.HTM` 配上 `htm` 不会变成 `REPORT.HTM.htm`；
/// 真要补，补的是调用方写的那个样子（`HTML` 就得到 `.HTML`）。
///
/// Turn a caller's `name` into a usable file name in this directory, appending the required extension
/// when the name carries none.
///
/// Legality is [`sanitize`]'s; one policy is this project's: a name that turns into nothing (`"/"`,
/// `".."`) has no usable form left and is an error. The extension is compared case-insensitively, so
/// `REPORT.HTM` under the `htm` extension does not become `REPORT.HTM.htm` — and when something is
/// appended, it is the extension as the caller wrote it (`HTML` yields `.HTML`).
fn sanitize_name(raw: &str, extension: &str) -> DuckResult<String> {
    let cleaned = sanitize(raw);

    if cleaned.trim_matches(['_', '.', ' ']).is_empty() {
        return Err(duck_error(format!("open_write_options.name is not a usable file name: '{raw}'")));
    }

    let mut name = cleaned;
    if !name.to_ascii_lowercase().ends_with(&extension.to_ascii_lowercase()) {
        name.push_str(extension);
    }

    Ok(name)
}

/// 在 `dir` 下新建一个不重名的文件，返回它那个「目录 + `/` + 文件名」形状的路径。
///
/// 名字的前缀是 `open-<时间>[-<part>]`（时间在最前，所以一个目录按名字排序就是按时间排序），随机尾缀、
/// 「当时这个名字是空的」以及撞名重试都交给 `tempfile`。它建的是**已经存在的空文件**，内容随后由
/// [`write`] 写进去 —— 名字归 `tempfile`、内容归 `std::fs`，写只有一条路径。
///
/// 返回时只取 `tempfile` 给的文件名再拼回 `dir`：`Path::join` 的分隔符按平台走，而这个字符串的形状是
/// 我们承诺过的（见 [`join`]）。
///
/// Create a uniquely named file under `dir` and return its "dir + `/` + file name" path.
///
/// The prefix is `open-<time>[-<part>]` (the time first, so sorting a directory by name sorts it by
/// time), while the random suffix, "this name was free at that moment" and the retry on collisions are
/// `tempfile`'s. What it creates is an **existing empty file**, which [`write`] then fills — the name is
/// `tempfile`'s, the content is `std::fs`', and there is still one write path.
///
/// Only the file name `tempfile` produced is taken back and joined onto `dir`: `Path::join` follows the
/// platform separator, while the shape of this string is one we promised (see [`join`]).
fn unique_name(dir: &str, extension: &str, part: Option<&String>) -> DuckResult<String> {
    // 结尾那个 `-` 是必要的：`tempfile` 把随机尾缀**直接接在** prefix 后面，不再加分隔符。
    let prefix = match part.map(|part| sanitize_part(part)) {
        Some(part) if !part.is_empty() => format!("{AUTO_PREFIX}-{}-{part}-", timestamp()),
        _ => format!("{AUTO_PREFIX}-{}-", timestamp()),
    };

    let created = tempfile::Builder::new()
        .prefix(&prefix)
        .suffix(extension)
        .tempfile_in(dir)
        .map_err(|err| {
            duck_error(format!("cannot create a file named '{prefix}' in '{dir}': {err}"))
        })?
        .into_temp_path()
        .keep()
        .map_err(|err| duck_error(format!("cannot keep the created file in '{dir}': {}", err.error)))?;

    let file_name = created
        .file_name()
        .ok_or_else(|| duck_error(format!("'{prefix}' is not usable as a file name in '{dir}'")))?
        .to_string_lossy();

    Ok(join(dir, &file_name))
}

/// 自动名字里的时间：`%Y%m%d-%H%M%S`，交给 chrono（日历算术不是这里该重写的东西）。
///
/// The timestamp of a generated name: `%Y%m%d-%H%M%S`, produced by chrono (calendar arithmetic is not
/// something to rewrite here).
fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
}

/// 把自动名字里 `part` 那一段变短、变干净：空格并成 `_`，最多 [`MAX_PART_CHARS`] 个字符，去掉截断
/// 之后多出来的 `_`。合法性同 [`sanitize`]。
///
/// Shorten and clean the `part` segment of a generated name: spaces become `_`, at most
/// [`MAX_PART_CHARS`] characters, with the `_` left over by the cut trimmed off. Legality is
/// [`sanitize`]'s.
fn sanitize_part(part: &str) -> String {
    sanitize(part)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .chars()
        .take(MAX_PART_CHARS)
        .collect::<String>()
        .trim_end_matches('_')
        .to_string()
}

/// 文件名合法性：非法字符（`/ \ ? < > : * | "`）、控制字符、Windows 保留设备名（`CON`、`NUL`、`COM1`…）
/// 与结尾的点与空格都按 `sanitize-filename` 的规则处理，我们只说「换成 `_`」。
///
/// 两个选项是刻意写死的：`windows: true`，因为 Windows 的规则最严，一套规则在三个平台都成立，不必按平台
/// 下菜；`truncate: false`，因为它按 255 **字节**截断，对一段名字太宽松，长度由调用方按字符数自己管。
///
/// File-name legality: illegal characters (`/ \ ? < > : * | "`), control characters, Windows reserved
/// device names (`CON`, `NUL`, `COM1`…) and trailing dots and spaces all follow `sanitize-filename`'s
/// rules, and all we say is "replace with `_`".
///
/// Two options are deliberately pinned: `windows: true`, because the Windows rules are the strictest and
/// one set of rules then holds on all three platforms; `truncate: false`, because it truncates at 255
/// *bytes*, far too generous for one segment, so the length stays the caller's business in characters.
fn sanitize(raw: &str) -> String {
    sanitize_filename::sanitize_with_options(
        raw,
        sanitize_filename::Options {
            windows: true,
            truncate: false,
            replacement: "_",
        },
    )
}

/// 目录 + 文件名，一律用 `/`（理由见模块头）。只去掉用户写在末尾的分隔符。
///
/// Directory plus file name, always with `/` (see the module header). Only a separator the caller wrote
/// at the end of the directory is trimmed.
fn join(dir: &str, file_name: &str) -> String {
    format!("{}/{}", dir.trim_end_matches(['/', '\\']), file_name)
}

#[cfg(test)]
mod tests {
    use super::{MAX_PART_CHARS, join, normalize_extension, sanitize_name, sanitize_part, write};
    use crate::extension::types::open_write_options::OpenWriteOptions;

    /// 一次测试自己的临时目录：`TempDir` 在测试结束时 drop，整棵目录跟着删掉，所以每个测试都要把它
    /// 握住（`into_path` 会放弃这个保证）。
    ///
    /// A temporary directory of one test's own: `TempDir` removes the whole tree when the test ends, so
    /// every test holds it (`into_path` would give that guarantee up).
    fn scratch_dir() -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().expect("create the scratch directory");
        let path = dir.path().to_string_lossy().into_owned();
        (dir, path)
    }

    fn options(dir: Option<&str>, name: Option<&str>, part: Option<&str>) -> OpenWriteOptions {
        OpenWriteOptions {
            dir: dir.map(str::to_owned),
            name: name.map(str::to_owned),
            part: part.map(str::to_owned),
        }
    }

    fn read_back(path: &str) -> String {
        std::fs::read_to_string(path).expect("read the written file back")
    }

    #[test]
    fn writes_the_named_file_into_the_given_directory() {
        let (_dir, dir_str) = scratch_dir();
        let path =
            write(b"hello", "html", Some(&options(Some(&dir_str), Some("report"), None))).expect("write");

        // 没写后缀的名字补上了后缀，路径按 `/` 拼。
        assert_eq!(path, format!("{dir_str}/report.html"));
        assert_eq!(read_back(&path), "hello");
    }

    #[test]
    fn an_existing_named_file_is_overwritten() {
        let (_dir, dir_str) = scratch_dir();
        let opts = options(Some(&dir_str), Some("report.html"), None);

        write(b"the first", "html", Some(&opts)).expect("first write");
        write(b"the second", "html", Some(&opts)).expect("second write");

        assert_eq!(read_back(&join(&dir_str, "report.html")), "the second");
    }

    #[test]
    fn the_extension_is_never_duplicated_behind_a_name_that_has_one() {
        let (_dir, dir_str) = scratch_dir();

        // 比较不区分大小写，两个方向都要成立：后缀那一项也可能是大写。
        let lower = write(b"x", "htm", Some(&options(Some(&dir_str), Some("REPORT.HTM"), None)))
            .expect("write");
        assert_eq!(lower, format!("{dir_str}/REPORT.HTM"));

        let upper = write(b"x", "HTML", Some(&options(Some(&dir_str), Some("page.HTML"), None)))
            .expect("write");
        assert_eq!(upper, format!("{dir_str}/page.HTML"));
    }

    #[test]
    fn a_generated_name_carries_the_prefix_the_time_and_the_part() {
        let (_dir, dir_str) = scratch_dir();
        let path = write(
            b"x",
            "svg",
            Some(&options(Some(&dir_str), None, Some("Sales Overview"))),
        )
        .expect("write");

        let file_name = path.rsplit('/').next().expect("a file name");
        // `open-<时间>-<part>-<随机尾缀>.svg`：时间紧跟前缀，所以按名字排序就是按时间排序。
        let rest = file_name.strip_prefix("open-").expect("the leading segment");
        let (time, tail) = rest.split_at(15);
        assert_eq!(time.chars().nth(8), Some('-'), "unexpected timestamp: {time}");
        assert!(
            time.chars().all(|c| c.is_ascii_digit() || c == '-'),
            "unexpected timestamp: {time}"
        );
        assert!(tail.starts_with("-Sales_Overview-"), "unexpected name: {file_name}");
        assert!(tail.ends_with(".svg"), "unexpected name: {file_name}");
        assert_eq!(read_back(&path), "x");
    }

    #[test]
    fn two_generated_names_do_not_collide() {
        let (_dir, dir_str) = scratch_dir();

        let first = write(b"a", "txt", Some(&options(Some(&dir_str), None, None))).expect("first");
        let second = write(b"b", "txt", Some(&options(Some(&dir_str), None, None))).expect("second");

        assert_ne!(first, second, "the random suffix keeps two writes apart");
        assert_eq!(read_back(&first), "a");
        assert_eq!(read_back(&second), "b");
    }

    #[test]
    fn a_missing_directory_is_created_including_parents() {
        let (_dir, dir_str) = scratch_dir();
        let nested = format!("{dir_str}/a/b");

        let path =
            write(b"x", "csv", Some(&options(Some(&nested), Some("table.csv"), None))).expect("write");

        assert_eq!(path, format!("{nested}/table.csv"));
        assert_eq!(read_back(&path), "x");
    }

    #[test]
    fn no_directory_lands_in_the_system_temp_directory() {
        let path = write(b"x", "txt", None).expect("write");

        assert!(path.ends_with(".txt"), "unexpected path: {path}");
        assert!(path.contains("open-"), "unexpected path: {path}");
        assert_eq!(read_back(&path), "x");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn illegal_characters_are_sanitized_out_of_the_name() {
        let (_dir, dir_str) = scratch_dir();
        let path =
            write(b"x", "txt", Some(&options(Some(&dir_str), Some("../evil:name"), None))).expect("write");

        let file_name = path.rsplit('/').next().expect("a file name");
        assert!(!file_name.contains(':') && !file_name.contains('/'), "unexpected: {file_name}");
        assert!(path.starts_with(&format!("{dir_str}/")), "unexpected: {path}");
    }

    #[test]
    fn an_extension_that_is_not_usable_is_reported() {
        for raw in ["", "   ", ".", "out/report.html"] {
            let err = normalize_extension(raw).unwrap_err();
            assert!(err.to_string().contains("extension"), "unexpected message for '{raw}': {err}");
        }

        assert_eq!(normalize_extension("html").unwrap(), ".html");
        assert_eq!(normalize_extension(".HTML").unwrap(), ".HTML");
        assert_eq!(normalize_extension(" tar.gz ").unwrap(), ".tar.gz");
    }

    #[test]
    fn a_name_that_is_not_usable_is_reported() {
        for raw in ["/", "..", "  "] {
            let err = sanitize_name(raw, ".html").unwrap_err();
            assert!(
                err.to_string().contains("open_write_options.name"),
                "unexpected message for '{raw}': {err}"
            );
        }
    }

    #[test]
    fn an_empty_directory_is_reported() {
        let err = write(b"x", "txt", Some(&options(Some("   "), None, None))).unwrap_err();
        assert!(err.to_string().contains("dir"), "unexpected message: {err}");
    }

    #[test]
    fn the_part_is_capped_and_spaced_out() {
        let long = "x".repeat(MAX_PART_CHARS + 20);
        assert_eq!(sanitize_part(&long).chars().count(), MAX_PART_CHARS);
        assert_eq!(sanitize_part("Sales  Overview"), "Sales_Overview");
        // 截断处多出来的 `_` 去掉。
        assert!(!sanitize_part(&"a_".repeat(MAX_PART_CHARS)).ends_with('_'));
    }
}
