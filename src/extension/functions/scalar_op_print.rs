// ============================================================================
// 标量函数：把一段文本直接打到 stdout（「在控制台里打开」）
//
//   op_print(text)              op_print(text, tail)
//
// 它解决的是终端探索里的一个具体尴尬：DuckDB 的 CLI 会把结果**排版** —— 列宽、边框、折行。字符画这类
// 输出（textplot 那种 ASCII/Braille 图，或者一个二维码）本来靠逐行对齐来读，被塞进一个 VARCHAR 单元格
// 里就全散了：框线被折行、列被拉宽、上下关系还在但已经看不出是图。`SELECT textplot(...)` 拿到的是字符串，
// 格式化是 CLI 的事，字符串救不回来。
//
// 于是不走结果集：内容直接写到 stdout，CLI 无从排版。这个扩展讲的是「把东西交到一个能看它的地方」，
// 控制台也是那样的地方 —— 这就是名字里的 `open`。
//
// ```sql
// INSTALL textplot FROM community;
// LOAD textplot;
// SELECT op_print(tp_qr('https://query.farm'));           -- 码，一行一组方块，打出来
// SELECT op_print('rows: ' || count(*)) FROM t;           -- 顺手一行，这一列也就是那句话
// ```
//
// 返回值就是递进来的那段文本：`text` 进、`text` 出。打印是这次调用的副作用，值本身不因此丢掉 —— 于是它
// 可以留在投影里，或者被下游的查询接着拼下去。重点仍在控制台那一份；一个本身就是那幅图的值（二维码）跟着
// 进结果集、被 CLI 排版之后扫不出来，那是这一份的问题，折一下就行（`SELECT length(op_print(tp_qr(url)))`
// 打出码、只回一个数字），或者把调用放进 `WHERE`。第二个签名里的 tail 只决定「打完追加什么」，不进返回值。
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
// column widths, borders, wrapping. Character-art output (an ASCII/braille chart, or a QR code, from an
// extension like textplot) is read line by line, and stuffing it into one VARCHAR cell destroys it: frame
// lines get wrapped, columns get stretched, and what is left is a string no longer recognisable as a
// chart. The query returns that string; formatting is the CLI's business and the string cannot be saved
// after it.
//
// So this writes to stdout, where nothing lays it out:
//
// ```sql
// INSTALL textplot FROM community;
// LOAD textplot;
// SELECT op_print(tp_qr('https://query.farm'));   -- the code, one row of blocks per line
// SELECT op_print('rows: ' || count(*)) FROM t;   -- a running note, which is also that column's value
// ```
//
// The return value is the text it was given: text in, text out. Printing is one side effect, and the value
// is not spent by it — the call can stay in a projection, or be concatenated further downstream. The console
// copy is still the point: a value that *is* the picture (a QR code) lands in the result set too, laid out,
// where it no longer scans — and that copy is the one thing to handle, by collapsing it
// (`SELECT length(op_print(tp_qr(url)))` prints the code and answers with a number) or by keeping the call
// out of the SELECT list. The `tail` of the second signature says what is appended, and is not part of the
// answer.
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
/// SELECT op_print(tp_qr('https://query.farm'));
/// SELECT op_print('=== ' || name) FROM reports;
/// ```
///
/// 标量函数逐行求值，所以一段查询里每行都会打一次，顺序就是这个执行序（多行时它是一份流水账，不是一份
/// 排好版的报告 —— 那本来就该留在结果集里）。回来的值和打出去的是同一段文本，所以调用留在投影里，控制台
/// 那份与结果集那份都看得见。NULL 行什么都不打、也回 NULL：没有输入，也就没有可打的东西。
///
/// 它是 volatile（见属性上的 `volatile = true`），所以一次调用就是一行：参数是常量也一样，
/// `SELECT op_print('x') FROM t` 有几行就打几行。同一个开关保证它折进别的表达式时只求值一次，也保证
/// `EXPLAIN` 底下一次都不打 —— 只做规划的查询不该有副作用。
///
/// Writes the text to stdout as it is, then a newline, and hands the text back. A scalar function is
/// evaluated per row, so a query with several rows prints several times in that execution order (a
/// running log, not a laid-out report — which is what the result set is for), and the value that comes
/// back is that same text, so keeping the call in a projection shows it in both places. A NULL row
/// prints nothing and answers NULL: no input, nothing to show.
///
/// It is volatile (the `volatile = true` on the attribute below), so one call is one row: a constant
/// argument behaves like a column one, and `SELECT op_print('x') FROM t` prints once per row. The same flag
/// is what makes a call inside a larger expression evaluate exactly once, and what makes `EXPLAIN` print
/// nothing — a query that is only being planned should not have side effects.
#[duck_scalar_function(
    description = "Prints a piece of text straight to stdout, bypassing the result set's column formatting",
    comment = "For character art and other line-aligned output the CLI would otherwise wrap: the text \
               is written to stdout as it is and handed back unchanged, so the call can stay in a \
               projection; the optional second argument is what follows the text instead of the \
               default newline",
    examples = [
        "SELECT op_print(tp_qr('https://query.farm'))",
        "SELECT op_print('rows: ' || count(*)) FROM t"
    ],
    volatile = true
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
    example = "SELECT op_print(report, chr(10) || chr(10))",
    volatile = true
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
