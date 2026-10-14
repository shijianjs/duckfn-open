---
title: 打印到 stdout
sidebar_position: 3
description: op_print —— 把文本直接写到 stdout，绕开结果集的排版，并把那段文本还回去。
---

# 打印到 stdout

## op_print

```text
op_print(text VARCHAR) -> VARCHAR
op_print(text VARCHAR, tail VARCHAR) -> VARCHAR
```

把文本直接写到 stdout，并把这段文本原样返回。控制台拿到的是它本来的样子，为的是绕开这件事：DuckDB CLI
会对结果*排版* —— 列宽、边框、折行 —— 而字符画（`textplot` 这类扩展产出的 ASCII 图或盲文图）是一行一行
读的，把它塞进一个 VARCHAR 单元格里就把它毁掉了。这个值并不会因此丢掉：调用可以留在投影里，或者被下游
接着拼下去，文本都还在。

```sql
SELECT op_print(textplot(y)) FROM series;           -- 控制台一份图，列里还是同一个字符串
SELECT op_print('rows: ' || count(*)) FROM t;       -- 一句备注，同时也是这一列的值
```

因为文本会回来，留在投影里的调用也就让结果集里多了一份 —— 照旧被排版过的那份。这就是「返回值」的代价。
只想让控制台看见它，就别把它放进 SELECT 列表：`SELECT count(*) FROM series WHERE op_print(textplot(y))
IS NOT NULL` 每张图都打出来，而结果集里只剩一个计数。

两个签名按「文本后面跟着什么」分开：没有 `tail` 时补一个换行，好让下一行输出不跟在这一行后面；给了
`tail` 就原样写它 —— `''` 表示「接着这一行往下写」，`chr(10) || chr(10)` 表示「留一个空行」。tail 不进
返回值。每次调用都会 flush，因为 Rust 按行缓冲 stdout，一个不带换行的 tail 否则会把文本留在缓冲区里。

它是个标量函数，所以按行求值：一个多行的查询打出来的是一份按执行顺序滚动的日志，而不是一份排好版的
报表 —— 报表那是结果集的活。有一个例外要提前算进去：参数是常量时，DuckDB 会把调用折叠成一次求值，所以
`SELECT op_print('x') FROM t` 无论 `t` 有多少行都只打一行。想要一行打一行，就得引用一个列
（`op_print('x: ' || a)`）。

NULL 进、NULL 出：没有输入的那一行什么都不打，答复也还是 NULL。
