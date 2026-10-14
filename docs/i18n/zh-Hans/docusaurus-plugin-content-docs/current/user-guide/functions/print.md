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

把文本直接写到 stdout，并把这段文本原样返回。控制台拿到的是它本来的样子：DuckDB CLI 会对结果*排版* ——
列宽、边框、折行 —— 而字符画是一行一行读的，塞进一个 VARCHAR 单元格就毁了。一个二维码最能说明问题：

```sql
INSTALL textplot FROM community;
LOAD textplot;

SELECT op_print(tp_qr('https://query.farm'));
-- 二维码，一行一组方块，就是它渲染出来的样子
```

控制台那一份才是重点：那正是这个函数存在的意义。而文本同时是返回值，所以同一次调用会在结果集里再放一份，
CLI 照旧按 VARCHAR 排版 —— 二维码放在那里会被折行，扫不出来。这一份想省掉很容易：这个调用就是一个普通的
VARCHAR 表达式，凡是能吃掉字符串的东西都能吃掉它：

```sql
-- 控制台是那个码，结果集里只回一个数字
SELECT length(op_print(tp_qr('https://query.farm')));

-- 想留住这个值：那句备注本身就成了这一列
SELECT op_print('rows: ' || count(*)) AS note FROM t;

-- 或者干脆别让它进 SELECT 列表
SELECT count(*) FROM (VALUES ('https://query.farm')) t(url) WHERE op_print(tp_qr(url)) IS NOT NULL;
```

两个签名按「文本后面跟着什么」分开：没有 `tail` 时补一个换行，好让下一行输出不跟在这一行后面；给了
`tail` 就原样写它 —— `''` 表示「接着这一行往下写」，`chr(10) || chr(10)` 表示「留一个空行」。tail 不进
返回值。每次调用都会 flush，因为 Rust 按行缓冲 stdout，一个不带换行的 tail 否则会把文本留在缓冲区里。

它是个标量函数，所以按行求值：一个多行的查询打出来的是一份按执行顺序滚动的日志，而不是一份排好版的
报表 —— 报表那是结果集的活。它也没有被折叠或缓存：`op_print` 注册成 volatile，所以 `SELECT op_print('x')
FROM t` 一行打一行，参数是不是常量都一样。同一个开关也解释了另外两件事 —— 折进别的表达式时它只求值一次
（`length(op_print(…))` 打一份，不是两份），而 `EXPLAIN` 底下一次都不打：只做规划、不执行的查询不会跑到它。

NULL 进、NULL 出：没有输入的那一行什么都不打，答复也还是 NULL。
