---
title: 函数
sidebar_position: 3
description: my_extension 注册的每一个 SQL 函数，都配了能在浏览器里直接跑的例子。
---

# 函数

加载扩展会注册三个函数。它们和 DuckDB 自带的函数一样用：可以放进投影、`WHERE` 条件或 `GROUP BY`，
也能和内置函数随意组合。

| 函数 | 类别 | 签名 | 说明 |
| --- | --- | --- | --- |
| `my_greet` | 标量 | `VARCHAR -> VARCHAR` | 向名字问好；永不为 NULL。 |
| `my_greet_checked` | 标量 | `VARCHAR -> VARCHAR` | 同上，但会校验名字。 |
| `my_sum` | 聚合 | `DOUBLE -> DOUBLE` | 对一列求和，跳过 NULL。 |

## my_greet

```text
my_greet(name VARCHAR) -> VARCHAR
```

向 `name` 问好，永不为 NULL。

```sql {"type":"duckfn","show":"table"}
SELECT my_greet('world') AS greeting;
```

## my_greet_checked

```text
my_greet_checked(name VARCHAR) -> VARCHAR
```

和 `my_greet` 一样，但会校验名字：空串返回 SQL `NULL`；名字首尾有空格会让整条查询失败。

```sql {"type":"duckfn","show":"table"}
SELECT name AS input, my_greet_checked(name) AS greeting
FROM (VALUES ('world'), ('')) t(name);
```

```sql {"type":"duckfn","expect":"error"}
SELECT my_greet_checked(' x ');    -- 报错：名字首尾不允许有空格
```

## my_sum

```text
my_sum(value DOUBLE) -> DOUBLE
```

一个聚合函数。它忽略 NULL 输入，整组都没有可用值时返回 SQL `NULL` 而不是 `0`。

```sql {"type":"duckfn","show":"table"}
SELECT grp, my_sum(x) AS total
FROM (VALUES ('rows', 1.5::DOUBLE), ('rows', 2.5), ('all NULL', NULL::DOUBLE)) t(grp, x)
GROUP BY grp
ORDER BY grp;
```

## 几点说明

- **NULL 进、NULL 出。** `my_greet_checked('')` 是 `NULL` 而不是空串；`my_sum` 跳过 NULL 行 ——
  所以整组都是 NULL 时结果是 `NULL`，这与 `0` 不是一回事。
- **被拒绝的输入会让整条语句失败。** 报错信息里带函数名，查询剩下的部分不会再执行。
- **数字按类型打印。** `my_sum` 返回 `DOUBLE`，所以 `7` 会显示成 `7.0`；想要别的样子就转一下
  （`my_sum(x)::DECIMAL(10,1)`）。
