// ============================================================================
// 标量函数：把路径或 URL 交给系统启动器（open::that / with / that_detached / with_detached）
//
// SQL 侧只有两个名字，各两个签名，靠 overloads_name 并成一个函数集：
//
//   op_open_path(path)              默认程序        op_open_path(path, app)        指定程序
//   op_open_path_detached(path)     默认程序，不等   op_open_path_detached(path, app)  指定程序，不等
//
// 「等不等」是 open 在 Linux 与 macOS 上唯一会显著影响一次查询的事：`open::that` 要等启动器返回，而
// xdg-open 通常到目标程序退出才返回。两个名字因此按这个分，而不是按平台分 —— 调用方要的语义与操作系统
// 无关。
//
// 入参是非空的 `String`，所以 NULL 行由 duckfn 的参数读取层短路成 NULL，函数体根本不会执行：SQL 里因此
// 有三种可分辨的结果 —— **NULL = 这一行没有输入**（不是失败）、**false = 启动器起不来**、**true = 启动器
// 被拉起**。想自己接住 NULL 再决定语义，把参数写成 `Option<String>`。
//
// ============================================================================
// Scalar functions: hand a path or URL to the system launcher (open::that / with / that_detached /
// with_detached).
//
// Two SQL names, two signatures each, merged into a function set with overloads_name:
//
//   op_open_path(path)              default program   op_open_path(path, app)           chosen program
//   op_open_path_detached(path)     same, no waiting  op_open_path_detached(path, app)  same, no waiting
//
// Whether to wait is the one thing that visibly changes a query on Linux and macOS: `open::that` waits
// for the launcher, and xdg-open comes back only when the target program exits. The two names therefore
// split on that, not on the platform — the semantics a caller wants are platform-independent.
//
// The arguments are non-optional `String`, so a NULL row is short-circuited to NULL by duckfn's
// argument reader and the body never runs. SQL can tell three outcomes apart: **NULL = no input on
// this row** (not a failure), **false = no launcher could be started**, **true = a launcher was
// started**. Write the parameter as `Option<String>` to take NULL into the body yourself.
// ============================================================================

use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::launcher;

/// `op_open_path(path)` / `op_open_path(path, app)`：交给系统启动器，并等它返回。
///
/// ```sql
/// SELECT op_open_path('report.html');
/// SELECT op_open_path('https://duckdb.org', 'firefox');
/// ```
///
/// 返回 false 表示这台机器上没有任何一个启动器能把这一步走通（Linux 上没有 `xdg-open`、Windows 上
/// PowerShell 辅助进程与 explorer.exe 都失败）。真实原因在 open 那一侧是 `io::Error`，SQL 里没有地方放它，
/// 所以只有 `op_open_commands` 的预览能回答「这里到底会跑什么」。
///
/// Hands the value to the system launcher and waits for it. false means no launcher on this machine got
/// through (no `xdg-open` on Linux, both the PowerShell helper and explorer.exe failing on Windows).
/// The real cause is an `io::Error` on open's side with nowhere to go in SQL, so only `op_open_commands`
/// can answer "what would actually run here".
#[duck_scalar_function(
    description = "Opens a local path or URL with the program the desktop is configured to use",
    comment = "true once a launcher has been started, false when none could be started; NULL input \
               short-circuits to NULL and starts nothing",
    examples = [
        "SELECT op_open_path('report.html')",
        "SELECT op_open_path('https://duckdb.org', 'firefox')"
    ],
    volatile = true
)]
pub(super) fn op_open_path(path: String) -> DuckOptionResult<bool> {
    launcher::check_not_empty(op_open_path::SQL_NAME, "path or URL", &path)?;
    Ok(Some(launcher::open_default(&path)))
}

/// `op_open_path(path, app)`：同一名字下的第二个签名，指定用哪个程序（`open::with`）。
///
/// ```sql
/// SELECT op_open_path('/data/report.csv', 'LibreOffice Calc');
/// ```
///
/// app 是原样交给平台的：Linux 上要的是能在 `PATH` 里找到的可执行名；Windows 上不带路径分隔符的名字会先查
/// 注册表的 App Paths 换成全路径，查不到就照原样启动。open 不解析别名、也不校验程序存不存在 —— 起不来
/// 就是 false。它也没法替自定义程序保证参数语法（选了哪个程序就接受了它的命令行文法），所以 app 来自
/// 不可信数据时这一步要自己想清楚。
///
/// The second signature under the same name: which program to use (`open::with`). The app reaches the
/// platform as it is: an executable on `PATH` on Linux, on Windows an App Paths lookup first when the
/// name has no separator in it and the name itself after that. open resolves no aliases and checks no
/// existence — a program that cannot be started is a false. It cannot quote arguments for a custom
/// program either (choosing an app means accepting its command-line grammar), so an app that comes from
/// untrusted data deserves a thought before this call.
#[duck_scalar_function(overloads_name = "op_open_path", volatile = true)]
pub(super) fn op_open_path_with(path: String, app: String) -> DuckOptionResult<bool> {
    launcher::check_not_empty(op_open_path_with::SQL_NAME, "path or URL", &path)?;
    launcher::check_not_empty(op_open_path_with::SQL_NAME, "application", &app)?;
    Ok(Some(launcher::open_with_app(&path, &app)))
}

/// `op_open_path_detached(path)` / `op_open_path_detached(path, app)`：起进程即返回（`open::that_detached`、
/// `open::with_detached`）。
///
/// ```sql
/// SELECT op_open_path_detached('https://duckdb.org');
/// SELECT count(*) FROM reports WHERE op_open_path_detached(url);
/// ```
///
/// Windows 上这一步走一次 ShellExecuteW（Cargo.toml 里开的 feature），比不带 `_detached` 的那两个更直接；
/// Linux 与 macOS 上是启动器进程的差别。代价是 true 的含义变松了：它只说明进程起来了，文件是否真被打开
/// 无从得知。要拿启动器的结果就用不带后缀的那一组。
///
/// Starts the launcher and returns immediately (`open::that_detached`, `open::with_detached`). On
/// Windows this is a single ShellExecuteW call (the feature turned on in Cargo.toml), so it is the
/// more direct route than the two waiting signatures; on Linux and macOS the difference is the launcher
/// process. The price is a looser true: it says a process started, not that the file opened. Use the
/// unsuffixed group to get the launcher's outcome.
#[duck_scalar_function(
    description = "Opens a local path or URL and returns as soon as the launcher is started",
    comment = "The non-blocking pair of op_open_path: true means a process was started, not that the \
               target opened",
    example = "SELECT op_open_path_detached('https://duckdb.org')",
    volatile = true
)]
pub(super) fn op_open_path_detached(path: String) -> DuckOptionResult<bool> {
    launcher::check_not_empty(op_open_path_detached::SQL_NAME, "path or URL", &path)?;
    Ok(Some(launcher::open_default_detached(&path)))
}

/// `op_open_path_detached(path, app)`：同一名字下指定程序的签名。
///
/// The signature of the same name that picks the program.
#[duck_scalar_function(overloads_name = "op_open_path_detached", volatile = true)]
pub(super) fn op_open_path_detached_with(path: String, app: String) -> DuckOptionResult<bool> {
    launcher::check_not_empty(op_open_path_detached_with::SQL_NAME, "path or URL", &path)?;
    launcher::check_not_empty(op_open_path_detached_with::SQL_NAME, "application", &app)?;
    Ok(Some(launcher::open_with_app_detached(&path, &app)))
}
