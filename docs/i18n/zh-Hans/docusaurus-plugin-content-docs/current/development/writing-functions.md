---
title: 编写函数
sidebar_position: 3
description: 本扩展注册的标量函数逐行拆解、duckfn 对入参与返回值的规则，以及新增函数时该抄哪一段。
---

# 编写函数

一个注册进 DuckDB 的函数 = 「一个普通 Rust 函数 + 一个属性宏」。宏负责生成 FFI 包装层、读参数列、写
结果列、提交注册项；函数体里只有你的逻辑。

从一个普通函数到可调用的 SQL 函数：

```mermaid
flowchart LR
    fn["普通 Rust 函数"] --> macro["duck 属性宏"]
    macro --> wrapper["FFI 包装层与<br/>注册项"]
    wrapper --> cdylib["扩展二进制<br/>cdylib"]
    cdylib --> load["在 DuckDB 里 LOAD"]
```

## 这个扩展注册了什么

七个 SQL 名字、十六个签名，全是标量，外加一个 STRUCT 类型（`open_write_options`），落盘那一族的可选
参数都装在它里面。多数函数体里没有自己的逻辑：校验一下入参，然后交给 `launcher.rs` 或 `writer.rs` ——
起进程的 crate 与写文件的 crate 都藏在这两层后面，不在这里（见[目录结构](./project-structure.md)）。
唯一的例外是 `open_print`，它要做的整件事就是往 stdout 写，所以那段写就在它自己的文件里。下面每一行
对调用方意味着什么（SQL 侧的语义），在用户指南的[函数](../user-guide/functions.md)那一页。

| SQL 名字 | 签名 | Rust 函数 | 文件 | 行为 |
| --- | --- | --- | --- | --- |
| `open_path` | `VARCHAR -> BOOLEAN` | `open_path` | `scalar_open_path.rs` | 默认程序，等启动器返回。 |
| `open_path` | `VARCHAR, VARCHAR -> BOOLEAN` | `open_path_with` | `scalar_open_path.rs` | 同上，程序由第二个参数指定。 |
| `open_path_detached` | `VARCHAR -> BOOLEAN` | `open_path_detached` | `scalar_open_path.rs` | 起进程即返回。 |
| `open_path_detached` | `VARCHAR, VARCHAR -> BOOLEAN` | `open_path_detached_with` | `scalar_open_path.rs` | 同上，程序由第二个参数指定。 |
| `open_commands` | `VARCHAR -> VARCHAR[]` | `open_commands` | `scalar_open_commands.rs` | 列出会依次尝试的启动命令，不起任何进程。 |
| `open_write` | `VARCHAR, VARCHAR -> VARCHAR` | `open_write` | `scalar_open_write.rs` | 把文本落成文件，返回路径，不起进程。 |
| `open_write` | `VARCHAR, VARCHAR, open_write_options -> VARCHAR` | `open_write_with` | `scalar_open_write.rs` | 同上，目录 / 文件名 / 命名段自己挑。 |
| `open_write` | `BLOB, VARCHAR -> VARCHAR` | `open_write_blob` | `scalar_open_write.rs` | 二进制内容，字节原样写出去。 |
| `open_write` | `BLOB, VARCHAR, open_write_options -> VARCHAR` | `open_write_blob_with` | `scalar_open_write.rs` | 同上，带选项。 |
| `open_content` | `VARCHAR, VARCHAR -> VARCHAR` | `open_content` | `scalar_open_content.rs` | 落盘，紧接着打开。 |
| `open_content` | `VARCHAR, VARCHAR, open_write_options -> VARCHAR` | `open_content_with` | `scalar_open_content.rs` | 同上，带选项。 |
| `open_content` | `BLOB, VARCHAR -> VARCHAR` | `open_content_blob` | `scalar_open_content.rs` | 二进制内容。 |
| `open_content` | `BLOB, VARCHAR, open_write_options -> VARCHAR` | `open_content_blob_with` | `scalar_open_content.rs` | 二进制内容，带选项。 |
| `open_html` | `VARCHAR -> VARCHAR` | `open_html` | `scalar_open_html.rs` | 全走缺省、后缀写死成 `html` 的 `open_content`。 |
| `open_print` | `VARCHAR -> VARCHAR` | `open_print` | `scalar_open_print.rs` | 打到 stdout，再补一个换行。返回值恒为 NULL。 |
| `open_print` | `VARCHAR, VARCHAR -> VARCHAR` | `open_print_with_tail` | `scalar_open_print.rs` | 同上，换行换成第二个参数给的东西。 |

传 `NULL` 就什么都不跑：`NULL` 的路径不叫启动器，`NULL` 的内容不落盘，`NULL` 的文本不打，答复一律是
`NULL` —— SQL 因此能把「这一行没有输入」和「这台机器起不动启动器」（`false`）、或者和一个真的写出来的
路径分开。

## 标量：三种返回形状

宏按返回类型生成不同的代码，而它认的外层形状恰好只有三种：

| 签名 | 语义 |
| --- | --- |
| `-> T` | 朴素值，永不为 NULL。 |
| `-> DuckOptionResult<T>` | 可空 + 可报错：`Ok(None)` 落成 SQL `NULL`，`Err` 让整条查询失败。 |
| `-> Option<T>` | 可空，但报不了错。 |

```rust
use duckfn::{DuckOptionResult, duck_scalar_function};

use crate::extension::launcher;

#[duck_scalar_function(
    description = "Opens a local path or URL with the program the desktop is configured to use",
    comment = "true once a launcher has been started, false when none could be started; NULL input \
               short-circuits to NULL and starts nothing",
    examples = [
        "SELECT open_path('report.html')",
        "SELECT open_path('https://duckdb.org', 'firefox')"
    ]
)]
pub(super) fn open_path(path: String) -> DuckOptionResult<bool> {
    launcher::check_not_empty(open_path::SQL_NAME, "path or URL", &path)?;
    Ok(Some(launcher::open_default(&path)))
}
```

这七个名字用的都是这个形状，剩下的信息由值类型说：

- 启动那一族是 `DuckOptionResult<bool>` —— 结论就是返回值；
- 落盘那一族与 `open_html` 是 `DuckOptionResult<String>`（一个路径），`open_print` 也是它，只不过永远回
  `Ok(None)`：文本已经上控制台了，SQL 这边没有值可给；
- `open_commands` 是 `DuckOptionResult<Vec<String>>`：`Vec<T>` 本身就是 SQL 的 list，所以返回类型是从
  Rust 类型读出来的，两边都不需要额外声明。

`open_path` 其实不会真去返回 `Ok(None)` —— SQL 里的 `NULL` 来自下面说的参数短路，而不是函数体 ——
但那个空串检查确实需要 `Err`，而 `bool` 正是平台层还给它的答复。

宏**不接受**的一个形状是 `DuckResult<T>`（它是 `Result<T, ExtensionError>` 的别名，而 `ExtensionError`
就是 `duck_error` 造出来的那个 `Err` 类型）。它属于下面两层 —— `writer::write` 与
`launcher::open_default_checked` 返回的都是 `DuckResult<…>` —— 函数体在自己的
`DuckOptionResult` 里用一个 `?` 把它变成 SQL 错误。

### 同一个名字下的多个签名

七个名字里有五个带着不止一个签名：`open_write` 与 `open_content` 各四个（文本 / 二进制 × 带选项 /
不带选项），`open_path` 与 `open_path_detached` 各两个（`(path)`、`(path, app)`），`open_print` 两个
（`(text)`、`(text, tail)`）。每个名字底下只有第一个签名按自己的 Rust 函数名注册，其余都是另一条定义，
靠 `overloads_name` 并进去：

```rust
#[duck_scalar_function(overloads_name = "open_write")]
pub(super) fn open_write_blob_with(
    content: DuckBlob,
    extension: String,
    options: OpenWriteOptions,
) -> DuckOptionResult<String> {
    write_bytes(open_write::SQL_NAME, &content.value, &extension, Some(&options))
}
```

目录里因此是一个 SQL 名字、四个签名 —— 两个长度为 2、两个长度为 3，各长度里再有一条收 `BLOB` ——
而 Rust 那边的分支名（`open_write_with`、`open_write_blob`、`open_write_blob_with`）不会多注册成额外的
函数。`test/sql/open_write.test` 断言的正是这个形状，两个签名的那一种由 `test/sql/open_path.test` 断言。

`overloads_name` 给的那个名字，也是宏生成的 `SQL_NAME` 常量装着的值，所以上面四条签名报错时都自称
`open_write: …`。这恰是共用那条助手函数（这里的 `write_bytes`）能成立的原因：前缀是个参数，每条签名把
自己的 `SQL_NAME` 传进去。

### 用到的参数类型

参数的 Rust 类型决定这一列怎么读；这个扩展用到三种。

| Rust 参数 | SQL 类型 | 为什么是它 |
| --- | --- | --- |
| `path: String` | `VARCHAR` | 文本进来，NULL 行短路（见下面一节）。 |
| `content: DuckBlob` | `BLOB` | 字节进、字节出，一个都不动 —— `content.value` 是 `Vec<u8>`。图片的字节不该被当成 UTF-8 过一遍，所以 `open_write` / `open_content` 各有文本与二进制两对。 |
| `options: OpenWriteOptions` | `open_write_options` | 这个扩展自己注册的那个 STRUCT，装三个可选项。 |

### STRUCT 参数：三个会咬人的地方

`open_write_options` 只定义在一处，`src/extension/types/open_write_options.rs`：

```rust
#[derive(Clone, Debug, Default, DuckStruct)]
#[duck(sql_name = "open_write_options", create_type = "replace")]
pub(crate) struct OpenWriteOptions {
    pub dir: Option<String>,
    pub name: Option<String>,
    pub part: Option<String>,
}
```

1. **每个字段都得是 `Option<T>`，这是硬要求，不是风格。** `#[derive(DuckStruct)]` 给每个字段生成的读取
   形如 `<字段类型>::read_slot(…) ?`，而它待在那个函数里的返回类型是 `Option<Self>` —— 所以只要有一个字段
   读到 NULL，**整个 struct** 就是 NULL。DuckDB 会把 struct 字面量里没写的键补成 NULL，于是若把 `name`
   写成 `String`，最常见的 `{'dir':'./out','part':'sales'}` 就会整体退化成「一个选项都没给」，用户给的
   `part` 被静默丢掉。写成 `Option<T>`，缺键才等于调用方的本意。
2. **省略字段的字面量不会被隐式加宽，所以要么写全、要么 cast。** 这是 DuckDB 的 binder，不是 duckfn：
   参数类型是完整的 `STRUCT(dir VARCHAR, "name" VARCHAR, part VARCHAR)`。

   ```sql
   SELECT open_write(svg, 'svg', {'dir':'./out'});
   -- Binder Error: No function matches the given name and argument types
   --   'open_write(STRING_LITERAL, STRING_LITERAL, STRUCT(dir VARCHAR))'

   SELECT open_write(svg, 'svg', {'dir':'./out'}::open_write_options);   -- 可以
   SELECT open_write(svg, 'svg', {'dir':'./out', 'name':NULL, 'part':'sales'});    -- 也可以
   ```

   这个字段显示成 `"name"`，因为 NAME 在 DuckDB 的语法里是关键字；字面量里写 `name:` 与 `'name':` 都成立。
3. **整个 STRUCT 传 NULL 就是普通 NULL 参数，照样短路。** `open_write('a', 'txt', NULL)` 回 `NULL`、一个
   文件都不写 —— 它**不是**「没给的那些按缺省」。想要「不传选项就走缺省」，函数就得写成两个签名（落盘
   那一族正是这么写的），而不是靠一个能收 NULL 的选项参数。

### 传入 NULL 时会发生什么

参数是否可空由 **参数类型** 决定，这条规则对标量与聚合一样：

- **`path: String`** —— 这里每一个签名都是这种写法。NULL 行被 duckfn 的参数读取层短路成 SQL `NULL`，
  函数体根本不会执行到那一行，也就一次启动器都不叫、一个文件都不写。多数情况下你要的就是这个。
- **`path: Option<String>`** —— NULL 以 `None` 进函数体，语义由你决定（返回 NULL、换成默认值、统计
  NULL 行数……）。

### 错误与 panic

遇到处理不了的值就 `Err(duck_error("…"))`，整条查询会带着你的消息失败。函数体里的 `panic!` 会被捕获、
作为 DuckDB 错误上报，而不是展开着穿过 FFI 边界。错误信息是给用户看的：用英文写，并带上函数名做前缀，
这样单独一行报错也读得懂 —— 前缀读的是宏生成的 `SQL_NAME` 常量（`open_path: the path or URL must not be
empty`），不是手抄的字面量；这也是这类函数要写成 `pub(super)` 的原因，生成的模块沿用函数的可见性。

落盘那一族报错也是同一个样子，一个用不了的设置一条消息：

```text
open_write: the extension must not be empty (pass e.g. 'html', not '')
open_write: open_write_options.dir must not be empty
open_write: open_write_options.name is not a usable file name: '/'
```

这三条出自不同的地方，形状却一致：前缀是 `SQL_NAME`（在函数文件里加），正文是那条领域规则（在
`writer.rs` 里加，并且按 SQL 里的键名把选项说清楚，好让调用点认得出来）。

该在 SQL 边界上报错就报错，别换一个说明不了任何事的值：这里空串要是交出去，只会得到一个 `false`，和
「这台机器没有可用的启动器」长得一模一样。

## 聚合：逐行输入 + 一个状态

:::note[这个扩展没有注册聚合函数]

下面这一节讲的是 duckfn 的写法，留在这里是因为它是这套宏能做的另一半；能跑的实现在 duckfn 自己的示例
扩展里（解包目录下的 `src/extension/functions/aggregate_function.rs`，路径见[目录结构](./project-structure.md)），
不在本仓库。

:::

聚合函数的签名 = 「输入列 + 一个 `&mut` 状态参数」（位置随意）。状态类型需要
`Default + Clone + Debug` —— 宏生成的包装结构体已经 derive 了它们 —— 并实现 `DuckAggregateState`：

- `combine` / `simple_combine` 合并两个状态。多线程与 group 归并都会走它，所以要满足结合律。
- `result` / `simple_result` 把状态变成这一组的结果。`simple_result` 只能给出永不为 NULL 的值；某一组
  需要回 SQL NULL 时，覆盖 `result` 返回 `Ok(None)`。
- `Output` 决定 SQL 返回类型：`i64`、`f64`、`String`、`Vec<…>`（即 `list<…>`）等等。

```rust
#[derive(Default, Debug, Clone)]
struct SumState {
    total: f64,
    rows: i64,
}

impl DuckAggregateState for SumState {
    type Output = f64;

    fn simple_combine(&mut self, other: &Self) {
        self.total += other.total;
        self.rows += other.rows;
    }

    fn result(&self) -> DuckOptionResult<f64> {
        if self.rows == 0 {
            Ok(None)          // 一行都没读到 -> SQL NULL，而不是 0
        } else {
            Ok(Some(self.total))
        }
    }
}
```

`SumState` 多记一个「读到过几行」，用来区分「空输入」与「读到过、和为 0」。状态里同样可以放 `String`、
`HashMap`、`Vec<…>`，或者按分组键各存一个槽 —— duckfn 指南的聚合那一章列了它支持的各种形状。

## 新增一个函数

1. **挑属性。** 标量、聚合、表函数、`COPY`、cast、SQL 宏、替换扫描各有一个，而且各自只接受自己的参数。
   参考 [duckfn 用户指南](https://shijianjs.github.io/duckfn/zh-Hans/) —— 对应那一章。
2. **从 `src/extension/functions/` 里抄最接近的那个函数**，只改业务逻辑，不要凭印象自创签名。
   `scalar_open_commands.rs` 是最小的完整样例（一个 `VARCHAR -> VARCHAR[]` 的标量，函数体两行）；
   `scalar_open_print.rs` 在它之上加了第二个签名和 `overloads_name` 那次合并；一个名字要同时带多个签名
   加一个选项参数时，抄 `scalar_open_write.rs`；想要一个把选项全部按缺省写死的快捷名字，看
   `scalar_open_html.rs`。
3. **把这活交给管这件事的那一层。** 函数体这里只做校验 + 一次调用。起进程，以及凡是按目标平台分叉的东西
   —— `#[cfg]`、某个只有一两个平台编得过的 crate —— 都写到 `src/extension/launcher.rs` 里；写文件，
   以及「写到哪儿」的一切（后缀、目录、自动命名、文件名合法性）都写到 `src/extension/writer.rs` 里。
   这两层存在的意义就是让函数文件始终只看到一个形状、让两个 SQL 名字共用一条规则。本项目只构建桌面
   平台，所以这棵树里没有 `#[cfg]`，每个依赖也就是 `[dependencies]` 里普通的一项，而不是
   `[target.'cfg(…)'.dependencies]` 里的一项；除非真要再加一套目标，否则别把它补回来。
   见[目录结构](./project-structure.md)。
4. **挂进模块树**：`src/extension/functions/mod.rs` 里一个文件一行 `mod …;`，新增一个面向 SQL 的类型就在
   `src/extension/types/mod.rs` 里同样加一行。crate root 不用动。
5. **写文档元数据**：属性上的 `description`、`comment`、`example` / `examples`：

   ```rust
   #[duck_scalar_function(
       description = "Lists, without running them, the launcher commands open_path would try in order",
       comment = "The read-only pair of open_path: nothing is started, so the result is stable enough to \
                  assert in CI on a headless runner",
       example = "SELECT open_commands('report.html')"
   )]
   ```

   只有那个拿到 SQL 名字的定义必须写这段文本：靠 `overloads_name` 并进去的签名可以什么都不带
   （`open_write_with` 就是这么写的），而某条签名要补充读者不能漏的信息时也可以自带一份
   （`open_write_blob` 就写了）。DuckDB 的 C 扩展 API 没有设置描述与示例的接口，所以这段文本是社区扩展页
   `Added Functions` 表的唯一来源。`just docs_csv` 会把它导出到 `target/function_descriptions.csv`，
   一个 SQL 名字一行（见[社区扩展](./community-extension.md)）。文案用英文写 —— 它会被原样贴到那个页面上。
6. **补测试**（见[测试](./testing.md)），然后跑 `just lint`。

### 命名

每个 SQL 名字共用一个短前缀（这里是 `open_`），前缀之后的部分要能读出这个函数做什么。这个名字是用户要
敲的，所以别写成 `open_open_thing` 这类堆砌。

属性宏默认拿 **Rust 函数名**当注册名，所以 `open_commands`、`open_html` 和 `open_print` 的第一条签名连
一个额外参数都不必写。同一个名字底下还要第二个签名时，用 `overloads_name = "open_write"` 并进去（写法见
上面「同一个名字下的多个签名」那节），此时 Rust 函数名与 SQL 名字就是有意不同的 —— SQL 那边从头到尾只
出现 `overloads_name` 给的那个名字。

宏还会为每个签名生成 `SQL_NAME` 常量，装的就是这条签名的 SQL 名字。同一个名字要在多处出现时（错误信息
前缀、日志、提示），读那个常量，别再抄一份字面量；代价是这类函数得写成 `pub(super)`，因为生成的模块
沿用函数的可见性。

### 配置参数

落盘那一族的选项是一个普通的 STRUCT 参数（三个会咬人的地方见上面「STRUCT 参数：三个会咬人的地方」那节），
它是这套写法里简单的那一半：形状不多花一分钱，校验规则还是 DuckDB 自己的。

另一半是 `DuckLazy<T>`，duckfn 的「配置只解析一次」参数：值按惰性读取，一条查询只解析一次而不是每行一次，
所以逐行解析不会出现在 profile 里。这个扩展没有用它 —— 落盘这一族本来就是逐行求值的，「每行一个文件」
正是要的效果。那套写法（命名 STRUCT 类型 + 惰性参数）在 duckfn 指南里有，
[duckfn_quantstats](https://github.com/shijianjs/duckfn-quantstats) 里也到处都是。
