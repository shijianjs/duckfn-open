---
title: 编写函数
sidebar_position: 3
description: 示例标量函数与聚合函数逐行拆解、duckfn 对入参与返回值的规则，以及新增函数时该抄哪一段。
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

## 示例

| 函数 | 类别 | 签名 | 行为 |
| --- | --- | --- | --- |
| `my_greet` | 标量 | `VARCHAR -> VARCHAR` | 永不为 NULL，最简形态。 |
| `my_greet_checked` | 标量 | `VARCHAR -> VARCHAR` | 空串回 `NULL`，首尾空格报错。 |
| `my_sum` | 聚合 | `DOUBLE -> DOUBLE` | 跳过 NULL 输入，整组没有有效行时返回 `NULL`。 |

三个都在 `src/extension/functions/` 下，一个文件放一个函数（或一小组合相关的函数）。

## 标量：三种返回形状

宏按返回类型生成不同的代码：

| 签名 | 语义 |
| --- | --- |
| `-> T` | 朴素值，永不为 NULL。 |
| `-> DuckOptionResult<T>` | 可空 + 可报错：`Ok(None)` 落成 SQL `NULL`，`Err` 让整条查询失败。 |
| `-> Option<T>` | 可空，但报不了错。 |

```rust
use duckfn::{DuckOptionResult, duck_error, duck_scalar_function};

#[duck_scalar_function(
    description = "Greets someone by name, returning NULL for an empty name and failing on whitespace",
    comment = "The nullable-and-fallible return shape: Ok(None) is SQL NULL, Err fails the query",
    example = "SELECT my_greet_checked('')"
)]
fn my_greet_checked(name: String) -> DuckOptionResult<String> {
    if name.is_empty() {
        return Ok(None);
    }
    if name.trim() != name {
        return Err(duck_error(
            "my_greet_checked: the name must not have surrounding whitespace",
        ));
    }
    Ok(Some(format!("Hello, {name}!")))
}
```

### 传入 NULL 时会发生什么

参数是否可空由 **参数类型** 决定，这条规则对标量与聚合一样：

- **`name: String`** —— NULL 行被 duckfn 的参数读取层短路成 SQL `NULL`，函数体根本不会执行到那一行。
  多数情况下你要的就是这个。
- **`name: Option<String>`** —— NULL 以 `None` 进函数体，语义由你决定（返回 NULL、换成默认值、统计
  NULL 行数……）。

### 错误与 panic

遇到处理不了的值就 `Err(duck_error("…"))`，整条查询会带着你的消息失败。函数体里的 `panic!` 会被捕获、
作为 DuckDB 错误上报，而不是展开着穿过 FFI 边界。错误信息是给用户看的：用英文写，并带上函数名做前缀，
这样单独一行报错也读得懂。

## 聚合：逐行输入 + 一个状态

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
2. **从 `src/extension/functions/` 里抄最接近的那个示例**，只改业务逻辑，不要凭印象自创签名。
3. **挂进模块树**：在 `src/extension/functions/mod.rs` 里加一行 `mod my_function;`。crate root 不用动。
4. **写文档元数据**：属性上的 `description`、`comment`、`example` / `examples`：

   ```rust
   #[duck_scalar_function(
       description = "One line for the function table of the community-extension page",
       comment = "The detail that does not fit the one-liner",
       example = "SELECT my_greet('world')"
   )]
   ```

   DuckDB 的 C 扩展 API 没有设置描述与示例的接口，所以这段文本是社区扩展页 `Added Functions` 表的唯一
   来源。`just docs_csv` 会把它导出到 `target/function_descriptions.csv`（见[社区扩展](./community-extension.md)）。
   文案用英文写 —— 它会被原样贴到那个页面上。
5. **补测试**（见[测试](./testing.md)），然后跑 `just lint`。

### 命名

每个 SQL 名字共用一个短前缀（模板里是 `my_`），前缀之后的部分要能读出这个函数做什么。这个名字是用户要
敲的，所以别写成 `my_ext_my_thing` 这类堆砌。

属性宏默认拿 **Rust 函数名**当注册名，示例因此叫 `my_greet` 与 `my_sum`。同一个名字下需要多个签名
（参数类型或个数不同）时，用 `overloads_name = "…"` 把它们并成一个函数集，而不是各自注册一个名字。

宏还会为每个签名生成 `SQL_NAME` 常量。同一个名字要在多处出现时（错误信息前缀、日志、提示），读那个
常量，别再抄一份字面量；代价是这类函数得写成 `pub(super)`，因为生成的模块沿用函数的可见性。

### 配置参数

配置类参数（`DuckLazy<T>`）只在函数内部解析一次，逐行解析不会出现在 profile 里。模板没有用到它；那套
写法（加载时创建命名 STRUCT 类型）在 duckfn 指南里有，[duckfn_quantstats](https://github.com/shijianjs/duckfn-quantstats)
里也到处都是。
