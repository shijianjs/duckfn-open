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

三个 SQL 名字、五个签名，全是标量。函数体里没有自己的逻辑：校验一下入参，然后交给
`launcher.rs` —— 真正调用 `open` 的地方在那一层（见[目录结构](./project-structure.md)）。

| SQL 名字 | 签名 | Rust 函数 | 文件 | 行为 |
| --- | --- | --- | --- | --- |
| `open_path` | `VARCHAR -> BOOLEAN` | `open_path` | `scalar_open_path.rs` | 默认程序，等启动器返回。 |
| `open_path` | `VARCHAR, VARCHAR -> BOOLEAN` | `open_path_with` | `scalar_open_path.rs` | 同上，程序由第二个参数指定。 |
| `open_path_detached` | `VARCHAR -> BOOLEAN` | `open_path_detached` | `scalar_open_path.rs` | 起进程即返回。 |
| `open_path_detached` | `VARCHAR, VARCHAR -> BOOLEAN` | `open_path_detached_with` | `scalar_open_path.rs` | 同上，程序由第二个参数指定。 |
| `open_commands` | `VARCHAR -> VARCHAR[]` | `open_commands` | `scalar_open_commands.rs` | 列出会依次尝试的启动命令，不起任何进程。 |

传 `NULL` 就什么都不跑：`NULL` 的路径直接被短路成 `NULL`，SQL 因此能把「这一行没有输入」和
「这台机器起不动启动器」（`false`）分开。

## 标量：三种返回形状

宏按返回类型生成不同的代码：

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

这五个函数用的都是这个形状。`open_path` 其实不会真去返回 `Ok(None)` —— SQL 里的 `NULL` 来自下面说的参数
短路，而不是函数体 —— 但那个空串检查确实需要 `Err`，而 `bool` 正是平台层还给它的答复。
`open_commands` 是同一个形状，只是值类型换成 `Vec<String>`，也就是 SQL 的 `VARCHAR[]`：返回类型是从
Rust 类型读出来的，两边都不需要额外声明。

### 同一个名字下的多个签名

两个启动器名字各有 `(path)` 与 `(path, app)`。第二个形式是一个单独命名的 Rust 函数，靠 `overloads_name`
并到第一个名字底下：

```rust
#[duck_scalar_function(overloads_name = "open_path")]
pub(super) fn open_path_with(path: String, app: String) -> DuckOptionResult<bool> {
    launcher::check_not_empty(open_path_with::SQL_NAME, "path or URL", &path)?;
    launcher::check_not_empty(open_path_with::SQL_NAME, "application", &app)?;
    Ok(Some(launcher::open_with_app(&path, &app)))
}
```

目录里因此只有一个 SQL 名字、两个 arity，而 Rust 那边的分支名（`open_path_with`）不会多注册成一个 SQL
函数 —— `test/sql/open_path.test` 断言的正是这一点。

### 传入 NULL 时会发生什么

参数是否可空由 **参数类型** 决定，这条规则对标量与聚合一样：

- **`path: String`** —— 这里五个签名都是这种写法。NULL 行被 duckfn 的参数读取层短路成 SQL `NULL`，函数体
  根本不会执行到那一行，也就一次启动器都不叫。多数情况下你要的就是这个。
- **`path: Option<String>`** —— NULL 以 `None` 进函数体，语义由你决定（返回 NULL、换成默认值、统计
  NULL 行数……）。

### 错误与 panic

遇到处理不了的值就 `Err(duck_error("…"))`，整条查询会带着你的消息失败。函数体里的 `panic!` 会被捕获、
作为 DuckDB 错误上报，而不是展开着穿过 FFI 边界。错误信息是给用户看的：用英文写，并带上函数名做前缀，
这样单独一行报错也读得懂 —— 前缀读的是宏生成的 `SQL_NAME` 常量（`open_path: the path or URL must not be
empty`），不是手抄的字面量；这也是这类函数要写成 `pub(super)` 的原因，生成的模块沿用函数的可见性。

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
   `scalar_open_path.rs` 在它之上加了第二个签名和 `overloads_name` 那次合并。
3. **平台相关的活交给平台层。** 函数体这里只做校验 + 一次调用。凡是按目标平台分叉的东西 —— `#[cfg]`、
   某个只有一两个平台编得过的 crate —— 都写到 `src/extension/launcher.rs` 里，函数文件因此始终只看到
   一个形状。本项目只构建桌面平台，所以这棵树里没有 `#[cfg]`，`open` 也就是 `[dependencies]` 里普通的
   一项，而不是 `[target.'cfg(…)'.dependencies]` 里的一项；除非真要再加一套目标，否则别把它补回来。
   见[目录结构](./project-structure.md)。
4. **挂进模块树**：`src/extension/functions/mod.rs` 里一个文件一行 `mod …;`。crate root 不用动。
5. **写文档元数据**：属性上的 `description`、`comment`、`example` / `examples`：

   ```rust
   #[duck_scalar_function(
       description = "Lists, without running them, the launcher commands open_path would try in order",
       comment = "The read-only pair of open_path: nothing is started, so the result is stable enough to \
                  assert in CI on a headless runner",
       example = "SELECT open_commands('report.html')"
   )]
   ```

   DuckDB 的 C 扩展 API 没有设置描述与示例的接口，所以这段文本是社区扩展页 `Added Functions` 表的唯一
   来源。`just docs_csv` 会把它导出到 `target/function_descriptions.csv`（见[社区扩展](./community-extension.md)）。
   文案用英文写 —— 它会被原样贴到那个页面上。
6. **补测试**（见[测试](./testing.md)），然后跑 `just lint`。

### 命名

每个 SQL 名字共用一个短前缀（这里是 `open_`），前缀之后的部分要能读出这个函数做什么。这个名字是用户要
敲的，所以别写成 `open_open_thing` 这类堆砌。

属性宏默认拿 **Rust 函数名**当注册名，所以 `open_commands` 与 `open_path_detached` 连一个额外参数都不必
写。同一个名字底下还要第二个签名时，用 `overloads_name = "open_path"` 并进去（写法见上面「同一个名字下的
多个签名」那节），此时 Rust 函数名与 SQL 名字就是有意不同的。

宏还会为每个签名生成 `SQL_NAME` 常量。同一个名字要在多处出现时（错误信息前缀、日志、提示），读那个
常量，别再抄一份字面量；代价是这类函数得写成 `pub(super)`，因为生成的模块沿用函数的可见性。

### 配置参数

配置类参数（`DuckLazy<T>`）只在函数内部解析一次，逐行解析不会出现在 profile 里。这个扩展没有用到它；那套
写法（加载时创建命名 STRUCT 类型）在 duckfn 指南里有，[duckfn_quantstats](https://github.com/shijianjs/duckfn-quantstats)
里也到处都是。
