// ============================================================================
// 标量函数：把一段文本直接打到 stdout（「在控制台里打开」）
//
//   op_print(text)              op_print(text, tail)
//
// 它解决的是终端探索里的一个具体尴尬：DuckDB 的 CLI 会把结果**排版** —— 列宽、边框、折行。字符画这类
// 输出（textplot 那种 ASCII/Braille 图）本来靠逐行对齐来读，被塞进一个 VARCHAR 单元格里就全散了：
// 框线被折行、列被拉宽、上下关系还在但已经看不出是图。`SELECT textplot(...)` 拿到的是字符串，格式化是
// CLI 的事，字符串救不回来。
//
// 于是不走结果集：内容直接写到 stdout，CLI 无从排版。这个扩展讲的是「把东西交到一个能看它的地方」，
// 控制台也是那样的地方 —— 这就是名字里的 `open`。
//
// ```sql
// SELECT op_print(textplot(y)) FROM series;              -- 一份图，打出来
// SELECT op_print('rows: ' || count(*)) FROM t;          -- 顺手一行，不占结果集
// ```
//
// 返回值就是递进来的那段文本：`text` 进、`text` 出。打印是这次调用的副作用，值本身不因此丢掉 —— 于是它
// 可以留在投影里，或者被下游的查询接着拼下去。第二个签名里的 tail 只决定「打完追加什么」，不进返回值。
//
// NULL 进、NULL 出：没有输入就没有可打的东西，函数体一次都不执行，答复也还是 NULL。
//
// 两个签名的分工是「打完追加什么」：不写 tail 时补一个换行，免得下一行的输出接在同一行上；写了 tail 就照
// 它给的来（`''` 是「接着打，不要换行」，`chr(10) || chr(10)` 是「空一行再打」）。
//
// 一次调用一个 flush：stdout 在 Rust 里是**按行**缓冲的，tail 不带换行时内容会留在缓冲区里，看起来就像
// 打了一半或者根本没打。探索要的是「现在就看到」。
//
// ============================================================================
// Scalar function: write a piece of text straight to stdout ("opening it in the console").
//
//   op_print(text)              op_print(text, tail)
//
// It answers one concrete awkwardness of exploring in a terminal: the DuckDB CLI *lays out* its results —
// column widths, borders, wrapping. Character-art output (an ASCII/braille chart from something like
// textplot) is read line by line, and stuffing it into one VARCHAR cell destroys it: frame lines get
// wrapped, columns get stretched, and what is left is a string no longer recognisable as a chart. The
// query returns that string; formatting is the CLI's business and the string cannot be saved after it.
//
// So this bypasses the result set and writes to stdout, where nothing can lay it out. The extension is
// about handing a thing to somewhere that can show it, and a console is such a place — hence `open` in
// the name.
//
// The return value is the text it was given: text in, text out. Printing is one side effect, and the
// value is not spent by it — so the call can stay in a projection, or be concatenated further
// downstream. The `tail` of the second signature says what is appended, and is not part of the answer.
//
// NULL in, NULL out: a row with no input has nothing to print, the body never runs, and the answer
// stays NULL.
//
// The two signatures split on "what follows the text": with no tail a newline is added so the next line
// of output does not run into this one; with a tail exactly that is written (`''` means "keep going on
// this line", `chr(10) || chr(10)` means "leave a blank line").
//
// One flush per call: Rust buffers stdout by line, so a tail without a newline would leave the text in
// the buffer, looking like it was printed half-way or not at all. Exploring wants to see it now.
// ============================================================================

use std::io::Write;

use duckfn::{DuckOptionResult, duck_error, duck_scalar_function};

/// 没写 tail 时追加的那个换行。
///
/// The newline appended when no tail is given.
const DEFAULT_TAIL: &str = "\n";

/// `op_print(text)`：把文本原样打到 stdout，后面补一个换行，然后把这段文本还回去。
///
/// ```sql
/// SELECT op_print(textplot(y)) FROM series;
/// SELECT op_print('=== ' || name) FROM reports;
/// ```
///
/// 标量函数逐行求值，所以一段查询里每行都会打一次，顺序就是这个执行序（多行时它是一份流水账，不是一份
/// 排好版的报告 —— 那本来就该留在结果集里）。回来的值和打出去的是同一段文本，所以调用留在投影里，控制台
/// 那份与结果集那份都看得见。NULL 行什么都不打、也回 NULL：没有输入，也就没有可打的东西。
///
/// 一个例外要留意：参数是**常量**时 DuckDB 会把它折成一次求值，于是 `SELECT op_print('x') FROM t`
/// 只打一行，不管 `t` 有多少行。要每行都打，参数里就得带上列引用（`op_print('x: ' || a)`）。
///
/// Writes the text to stdout as it is, then a newline, and hands the text back. A scalar function is
/// evaluated per row, so a query with several rows prints several times in that execution order (a
/// running log, not a laid-out report — which is what the result set is for), and the value that comes
/// back is that same text, so keeping the call in a projection shows it in both places. A NULL row
/// prints nothing and answers NULL: no input, nothing to show.
///
/// One exception to know about: a *constant* argument is folded into a single evaluation, so
/// `SELECT op_print('x') FROM t` prints one line however many rows `t` has. Reference a column to get
/// one line per row (`op_print('x: ' || a)`).
#[duck_scalar_function(
    description = "Prints a piece of text straight to stdout, bypassing the result set's column formatting",
    comment = "For character art and other line-aligned output the CLI would otherwise wrap: the text \
               is written to stdout as it is and handed back unchanged, so the call can stay in a \
               projection; the optional second argument is what follows the text instead of the \
               default newline",
    examples = [
        "SELECT op_print(textplot(y)) FROM series",
        "SELECT op_print('rows: ' || count(*)) FROM t"
    ]
)]
pub(super) fn op_print(text: String) -> DuckOptionResult<String> {
    print(op_print::SQL_NAME, &text, DEFAULT_TAIL)
}

/// `op_print(text, tail)`：同一名字下指定「打完追加什么」的签名。
///
/// ```sql
/// -- 不带换行：下一段接着同一行打
/// SELECT op_print(name || ' ', '') FROM t;
/// -- 空一行再打
/// SELECT op_print(report, chr(10) || chr(10));
/// ```
///
/// The signature of the same name that says what follows the text. An empty tail keeps the next output on
/// the same line; `chr(10) || chr(10)` leaves a blank line after it.
///
/// Only an `example` here: one function set merges into a single docs-page row that keeps the first
/// non-empty description, and `inventory` iterates in link order rather than source order, so two
/// descriptions in one set would pick the text by accident.
#[duck_scalar_function(
    overloads_name = "op_print",
    example = "SELECT op_print(report, chr(10) || chr(10))"
)]
pub(super) fn op_print_with_tail(text: String, tail: String) -> DuckOptionResult<String> {
    print(op_print::SQL_NAME, &text, &tail)
}

/// 写两段（文本、tail）并 flush；打不出去就报错（管道被关掉这类，`stdout` 不是 `/dev/null` 的时候很少）。
///
/// Write both pieces (the text, then the tail) and flush. A failure to write is reported as an error (a
/// closed pipe — rare, since stdout here is the CLI's own).
fn print(sql_name: &str, text: &str, tail: &str) -> DuckOptionResult<String> {
    let mut out = std::io::stdout().lock();

    out.write_all(text.as_bytes())
        .and_then(|_| out.write_all(tail.as_bytes()))
        .and_then(|_| out.flush())
        .map_err(|err| duck_error(format!("{sql_name}: cannot write to stdout: {err}")))?;

    // 打完了，把原样递进来的那段文本还回去：打印是这次调用的副作用，不是这个值的去处。
    Ok(Some(text.to_string()))
}
