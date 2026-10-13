// ============================================================================
// 标量函数：预览 open 会依次尝试哪些启动命令（open::commands）
//
// 「启动」这一族里唯一一个**不起进程**的动作，也是唯一一个启动答复稳定到能写进断言的：无桌面的 CI runner 上
// `open_path` 只会给出 false（那里没有 xdg-open，也没有 explorer.exe），而它把「换成有桌面的机器会跑什么」
// 原样列出来。两条用途都由此而来 —— 排查「为什么什么都没打开」，以及把候选序列交给 SQL 自己检查。
// （这个扩展里另有两条不起进程的路：`open_write` 只落盘、`open_print` 只打 stdout —— 它们不答复「会跑什么
// 命令」这个问题。）
//
// Example: the only action in the launch family that starts no process, and the only launch answer stable
// enough to assert. On a headless CI runner `open_path` can only answer false (no xdg-open, no explorer.exe
// there), while this lists what would run on a machine that has a desktop. Both uses come from that:
// debugging "nothing opened", and handing the candidate sequence to SQL to inspect. (Two other entry points
// start nothing either — `open_write` only writes, `open_print` only writes to stdout — but neither answers
// "what command would run here".)
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::launcher;

/// `open_commands(path)`：列出默认程序这条路会依次尝试的命令，一条都不执行。
///
/// ```sql
/// SELECT open_commands('report.html');
/// -- Linux: ['"xdg-open" "report.html"', '"/usr/bin/xdg-open" "report.html"', …]
/// -- Windows: ['"powershell.exe" "-NoProfile" … ', '"explorer.exe" "report.html"']
/// ```
///
/// 返回的是 LIST(VARCHAR)，顺序即 open 的尝试顺序：前一个起不来才轮到后一个。它回答的是「这台机器上会跑
/// 什么命令」，不是「目标程序会不会成功」—— 命令跑出来的退出码只有真去 launch 才看得到，那是
/// `open_path` 的事。
///
/// A LIST(VARCHAR) in open's own trying order: the next candidate only runs when the previous one could
/// not be started. It answers "what command would run on this machine", not "will the target program
/// succeed" — the exit status is only visible to something that actually launches, which is
/// `open_path`'s job.
#[duck_scalar_function(
    description = "Lists, without running them, the launcher commands open_path would try in order",
    comment = "The read-only pair of open_path: nothing is started, so the result is stable enough to \
               assert in CI on a headless runner",
    example = "SELECT open_commands('report.html')"
)]
pub(super) fn open_commands(path: String) -> DuckOptionResult<Vec<String>> {
    launcher::check_not_empty(open_commands::SQL_NAME, "path or URL", &path)?;
    launcher::preview_commands(&path)
}
