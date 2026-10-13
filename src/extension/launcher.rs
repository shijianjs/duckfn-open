// ============================================================================
// 平台层：把 open crate 的启动能力收成一个供 SQL 函数调用的形状
//
// 这一层只有两个职责，每个函数都薄到一眼看得完：
//
//   1. 把 `io::Result<()>` 压成 `bool` —— SQL 侧就返回它（true 表示启动器被成功拉起，见 scalar_open_path.rs）；
//   2. 把 `&str` 交给 open（它要的是 `OsStr` 与 app 名），并把 `Vec<Command>` 渲染成字符串列表。
//
// 五个签名共用这一个模块，函数文件因此只面对「返回 bool 的四个动作 + 一个会报错的预览」这一种形状。
// 这里不需要 `#[cfg]`：启动器是桌面软件，本扩展只构建桌面平台，没有 wasm 目标（见根目录 AGENTS.md）。
//
// Example：open 自己承担的事——各平台入口、参数引用、WSL 与 Wine 的分支、启动器起不来的错误——这里
// 一件都不重做（AGENTS.md「尽量用成熟三方库」）。
//
// ============================================================================
// The platform layer: open's launching ability, reshaped into what the SQL functions call.
//
// Two jobs, each function one line thin:
//
//   1. squash `io::Result<()>` into `bool` — the SQL return value (true means the launcher was started,
//      see scalar_open_path.rs);
//   2. hand `&str` to open (which wants `OsStr` and an app name) and render `Vec<Command>` as strings.
//
// The five signatures share this one module, so the function files face a single shape — four actions
// returning a bool plus one fallible preview. No `#[cfg]` is needed here: a launcher is desktop
// software, and this extension builds for desktop targets only (no wasm target; see AGENTS.md).
//
// Everything open already owns — the per-platform entry points, argument quoting, the WSL and Wine
// branches, the "no launcher started" error — is deliberately not reimplemented here.
// ============================================================================

use duckfn::{DuckOptionResult, DuckResult, duck_error};

/// 用系统默认程序打开路径或 URL，并等启动器返回（`open::that`）。
///
/// 会等这件事是 open 的语义，不是这里的取舍：Linux 上 `xdg-open` 通常要到目标程序退出才返回。不想等用
/// [`open_default_detached`]。
///
/// Open with the default program and wait for the launcher to come back (`open::that`). Waiting is
/// open's semantics, not a choice made here: `xdg-open` on Linux returns only once the target exits.
/// Use [`open_default_detached`] to not wait.
pub(crate) fn open_default(path: &str) -> bool {
    open::that(path).is_ok()
}

/// 用指定程序打开，并等启动器返回（`open::with`）。
///
/// Open with a chosen program and wait for the launcher (`open::with`).
pub(crate) fn open_with_app(path: &str, app: &str) -> bool {
    open::with(path, app).is_ok()
}

/// 用系统默认程序打开，起进程即返回（`open::that_detached`）。
///
/// Windows 上这一步走 ShellExecuteW（Cargo.toml 里那个 `shellexecute-on-windows` feature），是一次原生
/// API 调用；其余平台是 spawn 之后脱离。true 只说明**启动器起来了**，文件是否真被打开无从得知 —— 这
/// 正是它与 [`open_default`] 的分工：后者会拿到启动器的退出码。
///
/// Open with the default program and return as soon as the process is started (`open::that_detached`).
/// On Windows this is one ShellExecuteW call (the `shellexecute-on-windows` feature in Cargo.toml);
/// elsewhere it spawns and detaches. true only says **the launcher started** — whether the file opened
/// is unknowable there, which is the division of labour with [`open_default`]: that one gets the
/// launcher's exit status.
pub(crate) fn open_default_detached(path: &str) -> bool {
    open::that_detached(path).is_ok()
}

/// 用指定程序打开，起进程即返回（`open::with_detached`）。
///
/// Open with a chosen program and return as soon as the process is started (`open::with_detached`).
pub(crate) fn open_with_app_detached(path: &str, app: &str) -> bool {
    open::with_detached(path, app).is_ok()
}

/// 列出「默认程序这条路会依次尝试哪些启动命令」，一条都不执行（`open::commands`）。
///
/// 它是 open 在各平台上排好的候选序列（Linux 是 xdg-open 及其后备，Windows 是 PowerShell 辅助进程再
/// explorer.exe），所以这台机器上到底会跑什么，看它就够了 —— 而这是唯一一个**不起进程**的动作，也是
/// 唯一一个结果稳定到可以在 CI 里断言的动作（无桌面的 runner 上真去 launch 只会得到一个 false）。
///
/// 渲染成字符串用的是 `Command` 的 Debug 形状，即「程序名 + 逐个引用参数」，与 shell 无关；参数里的空格
/// 因此看得见面，不会被拼成一条歧义命令行。
///
/// List the launchers the default-program path would try in order, running none of them
/// (`open::commands`). This is open's per-platform candidate sequence (xdg-open and its fallbacks on
/// Linux, the PowerShell helper then explorer.exe on Windows), so it answers "what would actually run
/// here" — and it is the only action that starts no process, hence the only one whose result is stable
/// enough to assert in CI (a headless runner turns a real launch into a plain false).
///
/// Each command renders through `Command`'s Debug shape, i.e. "program plus one quoted argument each",
/// so spaces inside an argument stay visible instead of being spliced into an ambiguous command line.
pub(crate) fn preview_commands(path: &str) -> DuckOptionResult<Vec<String>> {
    Ok(Some(
        open::commands(path)
            .iter()
            .map(|cmd| format!("{cmd:?}"))
            .collect(),
    ))
}

/// 空串既不是路径也不是 URL，交给启动器只会换来一个答不上来的 false，所以在 SQL 边界上就报错。
///
/// `kind` 只为了把错误文案写得能指回调用点（"path or URL" / "application"）。
///
/// An empty string is neither a path nor a URL; handing it to a launcher only yields a false that
/// explains nothing, so it fails at the SQL boundary instead. `kind` exists only to make the message
/// point back at the call site ("path or URL" / "application").
pub(crate) fn check_not_empty(sql_name: &str, kind: &str, value: &str) -> DuckResult<()> {
    if value.is_empty() {
        return Err(duck_error(format!(
            "{sql_name}: the {kind} must not be empty"
        )));
    }
    Ok(())
}
