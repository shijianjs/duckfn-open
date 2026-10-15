---
title: 从数据造文件名
sidebar_position: 5
description: op_sanitize_filename 与 op_sanitize_filename_check —— 把一个值变成一个可用的文件名。
---

# 从数据造文件名

这两个既不碰桌面，也不碰文件系统。它们关于一个普通字符串回答的，是另外两组一旦有名字来自数据本身就会
撞上的那个问题：这个值能当一个文件名吗？

答复的就是那个值本身 —— 一个改写好的字符串，或者关于递进来的那个字符串的 `true`/`false`。它不起进程、
不写文件，所以再空的机器也容得下它；而**一个不能用的名字并不是一句错误**：「改写之后什么都不剩」是一个
合法答案，不是被拒绝的参数。

## op_sanitize_filename

```text
op_sanitize_filename(value VARCHAR) -> VARCHAR
op_sanitize_filename(value VARCHAR, replacement VARCHAR) -> VARCHAR
op_sanitize_filename(value VARCHAR, replacement VARCHAR, truncate BOOLEAN, windows BOOLEAN) -> VARCHAR
```

把一个字符串改写成一个能当文件名用的东西。除此之外什么都不发生 —— 没有进程、没有文件、没有目录：值进去，
值出来。这套规则管的是所有让一个字符串当不了名字的东西：非法字符 `/ \ ? < > : * | "`、控制字符、整段只是
点的名字（`.` 和 `..`）、Windows 保留的设备名（`CON`、`PRN`、`AUX`、`NUL`、`COM0`–`COM9`、`LPT0`–`LPT9`，
以及这些名字后面再挂上后缀的）、结尾的点与空格，还有 255 **字节**这一刀。

一条查询会要它，是因为查询里的文件名常常来自数据本身 —— 客户名、报表标题、抓回来的那一列 `path`。先把
一个值变成一个合法的名字，才谈得上拼目录、拼 URL、交给下一个扩展。

```sql
SELECT op_sanitize_filename('quarterly/report:v2.csv');   -- 'quarterlyreportv2.csv'
SELECT op_sanitize_filename('  the report.  ');           -- '  the report' —— 结尾被去掉，开头不动
SELECT op_sanitize_filename('CON.txt');                   -- '' —— 保留设备名不是一个名字
SELECT op_sanitize_filename('..');                        -- ''
```

缺省跟着这个库自己那一套，只有**一处是有意的例外**。`replacement` 是空串，所以非法字符是被**删掉**的，
不是被替换的；`truncate` 是开的。`windows` 却不跟 —— 它**无论宿主平台是什么都钉死成 `true`**：这个库里它的
缺省是编译时所在的那个平台，而那会让同一条 SQL 在不同机器上给出不同的答案。想要 Unix 那套规则就把这一项
传成 `false` —— 在那套规则下 `CON.txt` 就是一个普通文件名：

```sql
SELECT op_sanitize_filename('a/b:c?.txt', '_');    -- 'a_b_c_.txt' —— 替换符让改过的地方看得见
SELECT op_sanitize_filename('sales|east', '-');    -- 'sales-east'；替换符可以是任意字符串，也包括 ''
SELECT op_sanitize_filename('CON.txt', '_', true, false);   -- 'CON.txt'
SELECT length(op_sanitize_filename(repeat('a', 300)));                 -- 255
SELECT length(op_sanitize_filename(repeat('a', 300), '_', false, true));  -- 300
```

替换符只管显示：`'_'` 让名字的轮廓还看得出来，`''`（缺省那一个，也可以显式写出来）是同一次改写，只是把
窟窿合上了。

**`truncate` 与 `windows` 只能一起给**，也就是四个参数那一档的第三、第四项；这里**有意没有三个参数的
形式**，所以 `op_sanitize_filename('a.txt', '_', true)` 是一句 binder 错误：

```sql
SELECT op_sanitize_filename('a.txt', '_', true);          -- Binder Error：truncate 与 windows 是一对
```

这是这份 API 里唯一的一处压缩，不是少了一个用例：两个 BOOLEAN 摆在可选位置上是五种组合，而其中只有这个库
自己缺省的那两种值得单独给一个更短的形式。

**任何**一个参数是 NULL，这一行就是 NULL。缺掉的那个开关没有东西可以猜 —— 一个被截断的名字，和一个根本
没被问过要不要截断的名字，是两件事。

这两个函数不是 [`op_write_file`](./write-file.md#op_write_file) 的助手。那一族自己的命名策略是住在写入器
内部的第二套缺省 —— `name` 把非法字符换成 `_`、不截断，`part` 把空格并成 `_`、最长 48 个字符 —— 它从来
不是一个 SQL 名字，两套也都不能顶替对方。

## op_sanitize_filename_check

```text
op_sanitize_filename_check(value VARCHAR) -> BOOLEAN
op_sanitize_filename_check(value VARCHAR, truncate BOOLEAN, windows BOOLEAN) -> BOOLEAN
```

什么都不改写的那一半：`true` 的意思是，在同样的开关下 `op_sanitize_filename` 会把它拿到的那个值原样还
回来。

```sql
SELECT op_sanitize_filename_check('report.csv');   -- true
SELECT op_sanitize_filename_check('a/b.txt');      -- false
SELECT op_sanitize_filename_check('CON.txt');      -- false
SELECT op_sanitize_filename_check('trail. ');      -- false
SELECT count(*) FROM files WHERE NOT op_sanitize_filename_check(name);   -- 有几条会被改动
```

最后那一行就是这一对存在的原因：一条查询能数出改写会动到多少条，而在这层一条都不动 —— 这是有人要决定
怎么处置这批数据之前的体检与过滤那一步。

这里没有 `replacement` 参数，因为一个什么都不改写的问题，没有「换成什么」这一项。其余全都从
[`op_sanitize_filename`](#op_sanitize_filename) 沿用过来：同样那两个开关，而且只能一起给（所以带选项那一档
是三个参数，不是两个）；`windows` 钉成 `true`，所以 `op_sanitize_filename_check('CON.txt')` 在 Linux 上也是
`false`，尽管那个平台什么都不保留 —— `op_sanitize_filename_check('CON.txt', true, false)` 答复 `true`；还有
任何一个参数是 NULL，这一行就是 NULL。

这里同样没有更短的形式，而有一处形状值得绊一下：

```sql
SELECT op_sanitize_filename_check('a.txt', true);         -- Binder Error：带选项的形式是三个参数
```

这两个函数里没有任何一条是关于*值*本身的错误：非法字符、保留设备名、三百个字符的标题，都得到一个答案，
而不是一句报错。
