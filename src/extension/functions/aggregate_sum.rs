// ============================================================================
// 示例 2：聚合函数（整组进、一个值出）
//
// 聚合函数的签名比标量多一个「状态」参数：带 `&mut` 的那个就是聚合状态（位置随意），其余参数是逐行
// 的输入列。宏按签名顺序把输入与 `&mut state` 依次交给函数体，`update` 一行调一次。
//
// 状态类型要满足三件事：
//   1. `Default + Clone + Debug`（宏生成的包装结构体 derive 了它们）；
//   2. 实现 `DuckAggregateState`：`combine` 描述「合并两个状态」，`result` 描述「状态出结果」；
//      只用简单字段的话，用 `simple_combine` / `simple_result` 这两个简化版即可；
//   3. `Output` 决定 SQL 返回类型。
//
// 这里实现的是 SQL `sum` 的那套语义：**跳过 NULL**、**空输入返回 NULL 而不是 0**。后者靠状态里多记
// 一个「读到过几行」来区分，正好演示「状态可以带不止一个字段」。
//
// Example 2: aggregate functions (a whole group in, one value out).
//
// An aggregate has one more parameter than a scalar: the one taken by `&mut` is the aggregate state
// (anywhere in the signature) and the rest are per-row input columns. The macro hands the inputs and
// `&mut state` to the body in signature order, once per row.
//
// The state type has three requirements: `Default + Clone + Debug` (derived on the wrapper struct the
// macro generates), an implementation of `DuckAggregateState` (`combine` merges two states, `result`
// turns one into the output — or the simplified `simple_combine` / `simple_result` pair for plain
// fields), and an `Output` type that decides the SQL return type.
//
// What is implemented here is SQL `sum`: **NULL inputs are skipped** and **an empty group is NULL
// rather than 0**. The latter is why the state counts the rows it saw — a small demonstration that a
// state may carry more than one field.
// ============================================================================

use duckfn::{DuckAggregateState, DuckOptionResult, duck_aggregate_function};

/// 求和状态：总和 + 参与行数（行数用来区分「一行都没读到」和「读到过、和为 0」）。
///
/// The sum state: the total plus how many rows took part (the row count is what tells "no row at all"
/// apart from "rows seen, total is 0").
#[derive(Default, Debug, Clone)]
struct SumState {
    total: f64,
    rows: i64,
}

impl DuckAggregateState for SumState {
    /// 聚合的 SQL 返回类型。
    ///
    /// The SQL return type of the aggregate.
    type Output = f64;

    fn simple_combine(&mut self, other: &Self) {
        self.total += other.total;
        self.rows += other.rows;
    }

    /// 覆盖 `result` 而不是 `simple_result`：需要在「空输入」时返回 `Ok(None)`（SQL NULL），
    /// 而 `simple_result` 只能给出一个永不为 NULL 的值。
    ///
    /// `result` is overridden instead of `simple_result` because an empty group has to come back as
    /// `Ok(None)` (SQL NULL), while `simple_result` can only produce a never-NULL value.
    fn result(&self) -> DuckOptionResult<f64> {
        if self.rows == 0 {
            Ok(None)
        } else {
            Ok(Some(self.total))
        }
    }
}

/// `sum(x)`：`input` 不是 `Option`，所以 NULL 行在读取层就被跳过，函数体「看不见」NULL —— 这正是
/// SQL 聚合的惯例。想在函数体里看见 NULL（比如统计 NULL 的行数），把入参写成 `Option<f64>`。
///
/// ```sql
/// SELECT my_sum(x) FROM (VALUES (1.5), (2.5), (3.0)) t(x);  -- 7.0
/// SELECT my_sum(x) FROM (VALUES (1.0), (NULL)) t(x);        -- 1.0（NULL 被跳过）
/// SELECT my_sum(x) FROM (VALUES (NULL::DOUBLE)) t(x);       -- NULL（整组没有有效行）
/// ```
///
/// `sum(x)`: `input` is not an `Option`, so NULL rows are skipped by the reader and the body never
/// sees a NULL — which is exactly the SQL aggregate convention. Write the parameter as
/// `Option<f64>` to see NULLs in the body (to count them, say).
#[duck_aggregate_function(
    description = "Sums a DOUBLE column, skipping NULLs, the simplest possible aggregate function",
    comment = "An empty group, or one where every row is NULL, yields NULL rather than 0",
    examples = [
        "SELECT my_sum(x) FROM (VALUES (1.5), (2.5), (3.0)) t(x)",
        "SELECT my_sum(x) FROM (VALUES (1.0), (NULL)) t(x)"
    ]
)]
fn my_sum(input: f64, state: &mut SumState) {
    state.total += input;
    state.rows += 1;
}
