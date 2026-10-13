---
title: 测试
sidebar_position: 4
description: test/sql 下的 SQLLogicTest 用例、怎么跑（make、just 或直接调运行器）、测试把文件写到哪儿，以及新增函数至少该覆盖什么。
---

# 测试

测试用 [SQLLogicTest](https://duckdb.org/dev/sqllogictest/intro) 格式写在 `test/sql/` 下，与 DuckDB
自己的测试套件同一种格式。一个文件就是一串 `query` / `statement` 块，期望结果写在原地，所以用例同时也是
函数行为的可运行示例。一共八份：`src/extension/functions/` 下每个函数文件一份，再加一份冒烟测试。

| 文件 | 覆盖什么 |
| --- | --- |
| `duckfn_open.test` | 冒烟测试。`LOAD` 之前，五个名字（`op_open_path`、`op_open_commands`、`op_write`、`op_print`、`op_filename`）都是 `does not exist`；`require` 之后把目录读两遍 —— 九个 `op_` 名字按顺序并成一串，以及 `count(*)` 出来的二十一个签名。接着对测试目录里真实存在的那个文件跑一次 `op_open_commands`（它不起进程），以及 `SELECT op_open_path(NULL)` 是 `NULL`。它也是「扩展能被加载」的最小证明。 |
| `op_open_path.test` | 两个启动器名字作为函数集的样子：从目录里读出的注册形状（arity 1 与 2，返回 `BOOLEAN`）、常量 **和** 运行期的 `NULL` 都被短路（所以永远叫不起启动器）、四条空参数报错文案、参数个数/类型的 binder 报错，外加一条「Rust 那边的分支名（`op_open_path_with`、`op_open_path_detached_with`）没有多注册成 SQL 函数」的断言。 |
| `op_open_commands.test` | 只读那条预览，也就是在哪台机器上都能真跑的启动答案：`VARCHAR[]` 既从目录里读、也从 `typeof` 验一遍、候选序列非空、至少有一个候选把路径带在参数里、两处 `NULL` 短路、空路径报错、多给一个参数和给数字时的 binder 报错，以及跨多个 DataChunk 的 3000 行。 |
| `op_write.test` | **真的执行**：它不起任何进程，所以每一步在没有桌面的机器上都能断言。一个名字四个签名（arity 2 与 3，其中恰好两个收 `BLOB`，返回类型都是 `VARCHAR`）；显式给的 `dir` + `name` 原样拼成那条路径，内容再用 `read_text` 读回来；覆盖语义 —— 第二次只写一个短 body，文件就是短的；不存在的目录一直建到 `nested/a/b`；自动生成的名字用 `regexp_matches` 对 `open-<8 位>-<6 位>-<part>-<随机>.<ext>` 这个形状，带 `part` 与不带各验一次；PNG 的魔数从一个 `BLOB` 走进去、再用 `read_blob` + `hex` 读回来；四条选项校验报错；被清洗过的 `name` 逃不出给定目录；三处 `NULL` 短路（内容、后缀，以及整个 `NULL` 的选项 STRUCT —— 它**不是**「其余项按缺省走」）；最后是**省略**第 3 个参数的那条：文件真的落在系统临时目录里，也读回来验了一次。 |
| `op_open_content.test` | 只有形状与 launch 之前的校验 —— **这里的调用一次都到不了 launch**。四个签名与那唯一的 `VARCHAR` 返回类型；同样几条 writer 报错，前缀换成 `op_open_content:`；`NULL` 短路（常量与运行期）；多给第四个参数时的 binder 报错。有一条断言证明了它失败得早：名字不可用那条报错之后，对那个目录 `glob()` 数出来是 0 个文件 —— 什么都没写，也就没什么能被打开。 |
| `op_open_html.test` | 那个零配置的快捷方式，同样因为这理由一次都没被真调用：一个名字、一个签名，进 `VARCHAR` 出 `VARCHAR`，而且没有 `BLOB` 签名（数出来是 0）；给两个参数、不给参数、给数字时的 binder 报错；两种 `NULL` 短路。文件末尾记下了一条不给测试跑的说明：空串**是**有内容的（它会落盘并把浏览器叫起来），所以那一条留给人工验证。 |
| `op_print.test` | **真的执行**：不起进程、不碰文件系统，只往 stdout 写，而那些字落在运行器自己的控制台里、从不进结果集 —— 所以断言全都盯着 SQL 的结论，也就是恒为 `NULL`。两个 arity 与它们的返回类型；空串；`tail` 给 `''` 与给两个换行各一次；三行确实打了三次，因为参数引用了列（常量参数会被折叠成一次调用，那种写法什么也证明不了）；两处 `NULL` 短路；三条 binder 报错；以及没有泄漏出 `op_print_with_tail`。 |
| `op_filename.test` | **真的执行**：纯字符串这一族不起进程、不碰文件系统，答复又不依赖这台机器，所以它是唯一一份能把库自己的那六条规则在每一条返回的字符串上逐字对一遍、而不必退回去验形状的用例。六条规则各自一条断言：非法字符与控制字符被删掉、`.` 与 `..` 清空、Windows 的保留设备名连后缀一起把整个名字吞掉（`CON.txt` → `''`）、结尾的点与空格去掉而开头的不动、255 **字节**这一刀开着与关掉各验一次。再加上三个开关（`replacement` 给 `''`、`'_'`、`'-'`，以及两个字符的 `'__'`；`windows` = `false` 时 `CON.txt` 留着）；两个名字的注册形状（arity 1/2/4 → `VARCHAR`，1/3 → `BOOLEAN`，每个 arity 的 `parameter_types` 也读一遍）；**缺的那一档三个参数**被断言成 binder 报错，好让没人把这一处缺口当成漏写；一条证明判断与改写同源的交叉检查 —— 十一个值 × `truncate` × `windows` 的全部组合里，`op_filename_check(n, tr, w)` 与 `op_filename(n, '', tr, w) = n` 不一致的是 0 行；三行里两行真的需要改写（不是常量折叠）；五处 `NULL` 短路，外加一整列里混着一行 `NULL`；五条 binder 报错；以及那三个 Rust 分支名（`op_filename_with_replacement`、`op_filename_with_options`、`op_filename_check_with_options`）没有多注册成 SQL 函数。 |

这些文件围着一条规则写：**只断言没有桌面也答得出的东西。** 会走到桌面的那四个名字 —— `op_open_path`、
`op_open_path_detached`、`op_open_content`、`op_open_html` —— 在整个测试套件里一次都没被真拿去 launch。一次 launch
回不回 `true`，取决于这台机器装了哪些启动器、`.html` 关联到了谁 —— 而 CI 的 runner 恰恰两样都没有，那里
每个动作只能回 `false`。所以这组名字是靠目录、靠 `NULL`、靠 binder 拦下什么来验的；`op_open_content` 与
`op_open_html` 还要靠那些**早于**落盘与 launch 就返回的校验错误 —— 在那里就失败的调用同样一个进程都不会起，
所以在哪台机器上都成立。真正跑起来的那些都不起进程：`op_open_commands` 只列命令不执行，`op_write` 写文件
再把内容读回来，`op_print` 往控制台写，`op_filename` 只凭手里那串字符就答得出来 —— 这台机器的任何差别
都插不进它，所以它的用例是唯一一份把改写规则本身逐条对一遍、而不必退回去验形状的文件。

## 怎么跑

一个文件是怎么走到运行器的：

```mermaid
flowchart LR
    file["test/sql/*.test"] --> req["require duckfn_open<br/>加载产物"]
    req --> blocks["query / statement 块<br/>期望结果写在原地"]
    blocks --> runner["duckdb_sqllogictest<br/>比对并报告"]
```

```shell
just test                 # make configure + make debug，然后 make test，最后清理
just ci-build             # 只做官方构建，不跑测试
```

`just test` 走 DuckDB 官方的 Makefile 流程，CI 也是这条：它先跑依赖 `ci-build`（`make configure` +
`make debug`），再 `make test`，最后 `git clean -fdX -- test/sql`。三件要知道的事：

- **`just test` 会重新构建，单独跑 `make test` 不会。** 改完 Rust 之后，`make test` —— 或者下面那个运行器
  —— 跑的都是上一次构建出来的产物，所以先构建（`just ci-build` 或 `make debug`）。
- **Windows 上 `make` 必须在 Git Bash 里跑**，PowerShell 里跑不起来。
- **最后那句清理是必要的，而且测试失败时它不会跑。** `just` 在报错那一行 recipe 就中止，于是落盘用例写出来
  的文件会一直留在 `test/sql/` 下，直到套件重新变绿。那里的断言之所以要那么写，原因就在这条 —— 见下文
  「测试把文件写到哪儿」。

### 更快的迭代

DuckDB 的测试运行器可以直接驱动产物，完全跳过 `make`。它需要一个装了 `duckdb_sqllogictest` 的 Python
环境 —— `make configure` 会在 `configure/venv` 下建一个，你也可以用任何装了这个包的 Python 3 环境：

```bash
# Linux / macOS
./configure/venv/bin/python -m duckdb_sqllogictest \
    --test-dir test/sql \
    --external-extension build/debug/duckfn_open.duckdb_extension
```

```powershell
# Windows
.\configure\venv\Scripts\python.exe -m duckdb_sqllogictest `
    --test-dir test/sql `
    --external-extension build/debug/duckfn_open.duckdb_extension
```

`--test-dir` 必给：它同时是 `__TEST_DIR__` 的取值，也就是会落盘写文件的用例拿到的目录。只跑一份就再加
`--file-path test/sql/op_write.test`。这条路上没有任何清理 —— 用例写到哪儿，文件就留在哪儿。

## 测试把文件写到哪儿

`__TEST_DIR__` 就是 `--test-dir` 原样代进去的值，而在这个 runner 里那就是 `test/sql` 本身。所以一个会写
文件的用例，写进的是仓库：

```text
test/sql/named/report.md
test/sql/blob/magic.png
test/sql/unique/open-20261013-200706-Sales_Overview-2U9nzK.svg
test/sql/nested/a/b/deep.txt      这个目录连父级一起被建了出来
```

`test/sql/*.*` 早就被忽略了，这次再加一条 `test/sql/*/` 盖住这些子目录，所以它们不会出现在 `git status`
里，而 `just test` 收尾的 `git clean -fdX -- test/sql` 会把它们删掉（`-X` 只删被忽略的文件，tracked 的
`.test` 不受影响）。

落盘用例怎么写断言，由此得出两条。

**要对残留免疫。** 失败那次跑完，清理那句没执行，于是「这个目录里有几个文件」断言的是上一次运行，而不是
这一次。`op_write.test` 把自己拿到的那两个路径留下，只数**这两个**路径对应的文件：

```sql
CREATE TEMP TABLE auto_named AS (
    SELECT op_write('first', 'txt', {'dir':'__TEST_DIR__/unique','name':NULL,'part':NULL}) AS a,
           op_write('second','txt', {'dir':'__TEST_DIR__/unique','name':NULL,'part':NULL}) AS b
);

SELECT count(*) = 2
FROM glob('__TEST_DIR__/unique/*') g, auto_named t
WHERE replace(g.file, chr(92), '/') IN (t.a, t.b);    -- true
```

**`glob()` 用平台分隔符，这个扩展一律用 `/`。** Windows 上 `glob()` 给的是 `test\sql\unique\open-…txt`，
而 `op_write` 回的是 `test/sql/unique/open-…txt`，所以比对之前得把一侧归一化
（`replace(g.file, chr(92), '/')`）。不写这一步，这条断言在 Windows 上就永远配不上 —— 反过来说，「一律用
`/`」那条约定也就在这一处被机器验了一遍。

还有一条关于把内容读回来的：`read_text` / `read_blob` 的文件参数必须是能折叠成常量的表达式。
`read_text(op_write('x', 'txt', {'dir':'…', 'name':'a', 'part':NULL}))` 可以，因为合成两头的参数全是
字面量；`read_text((SELECT a FROM t))` 不行 —— `Binder Error: Table function cannot contain subqueries`。
所以要不在产出路径的那条语句里就把内容读回来，要么用一个字面量路径。

## 约定

- **每个文件都从干净的数据库开始**，所以 `duckfn_open.test` 可以断言加载前这些名字不存在，其余文件开头写
  `require duckfn_open`。
- **错误是子串匹配。** `statement error` 下面写有辨识度的那一段就够了，不必抄 DuckDB 整条错误消息 ——
  抄全了反而会把用例绑死在一个随时可能改的文案上。本仓库自己的文案该点名的就点名（`op_open_path: the path or
  URL must not be empty`、`op_write_options.dir must not be empty`），句首就是那个 SQL 名字。纯字符串那一族
  没有自己的文案 —— `op_filename` 会被拦下来的全是 binder 的报错，所以那些就只匹配 `Binder Error`。这个
  前缀来自宏生成的 `SQL_NAME`，不是 Rust 函数名，所以三个参数的 `op_write_with` 报的仍然是
  `op_write: …` —— 在 SQL 里调用的人看到的就该是这个名字。
- **用例不许走到 launch。** 这里没有任何一条断言依赖真实的 launch：注册形状来自 `duckdb_functions()`，
  `op_open_commands` 被真调用是因为它设计上就不起进程，而那两个真会打开东西的名字（`op_open_content`、
  `op_open_html`）只以「在 launch 之前就失败」或「被 `NULL` 短路」的形式出现。
- **落盘用例要验字节，不能只验路径。** 光断言路径只说明「承诺了什么」，不说明「落下了什么」，所以
  `op_write.test` 每个文件都用 `read_text` / `read_blob` 读回来看一眼。
- **留意值怎么打印。** `DOUBLE` 打印成 `7.0`；用例关心的是数字而不是类型时，转一下
  （`SUM(x)::DECIMAL(10,1)`），这样类型变了期望值也不用改。
- **按目录里的名字检索时小心 `LIKE`。** 它的 `_` 匹配任意单个字符，`function_name LIKE '%_with%'` 会连
  你根本没想包括的名字一起命中；函数集那几份用例因此用 `IN (…)` 逐个点名。（冒烟测试里的 `LIKE 'op_%'`
  用的是同一个通配符，只是目录里眼下没有别的以 `op` 加任意一个字符开头的名字，才没出问题；文档页里那些
  检索 catalog 的查询用的是 `starts_with(function_name, 'op_')`，里面没有通配符。）
- **按关注点拆文件**，不要按函数个数：行为一份、错误路径一份；用到社区扩展（比如解析 HTML）的用例单独
  放，因为它首次运行需要网络。

## 新增函数至少覆盖什么

正常值、`NULL`、边界值、错误路径。另外三条很值：

- 文件是冒烟测试的话，加一条 `LOAD` 之前的 `statement error` … `does not exist`；
- 一条 **不是常量折叠** 的 NULL 输入 —— 常量 `NULL` 根本不会进函数体，而对启动器来说这种行恰恰是「真会
  去叫起一个窗口」的那一种，对 `op_write` 则是「真会写出一个文件」的那一种；
- 行数超过 `STANDARD_VECTOR_SIZE`（2048）的用例 —— 聚合的 `combine`、标量的逐 chunk 路径都靠它才跑到
  （`op_open_commands.test` 用 `range(3000)` 跑的就是后者）。

要跟操作系统打交道的函数，还要覆盖目录里怎么说它：这个 SQL 名字下的每一个 arity 与返回类型，Rust 那一侧
的名字没有多注册出一个函数；如果它还收一个选项 STRUCT，那要数一数有几个签名接受 `BLOB`
（`op_write.test` 与 `op_open_content.test` 都是这么做的，而不是相信那份清单）。

目录是「这台机器上跑不了的那些名字」的**退路**，不是首选写法。参数之外不碰任何东西的函数要按更强的标准
来：逐字断言返回的值，就像 `op_filename.test` 把每条清洗规则在拿回来的字符串上对一遍那样。这样的 API
若在哪一档 arity 上有意留了缺口，就把缺口也测出来 —— `op_filename(x, '_', true)` 被断言成一条 binder
报错，因为一个没被说明的缺失签名，读起来就是漏写。

提交前：`just lint`（`cargo clippy --all-targets -- -D warnings`）。另外，`src/extension/writer.rs` 里的命名
与清洗规则还有一组 Rust 单元测试（`cargo test`，十二条）—— 那是 SQL 用例拿不到的方便：`TempDir` 会在测试
结束时把整个临时目录删掉，`assert_eq!` 比路径也不需要字符串改写。它们不属于 `just test`，那条只跑
`.test` 文件。
