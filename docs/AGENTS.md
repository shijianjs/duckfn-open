# AGENTS.md —— docs 站约定

这份文件是给改 `docs/` 的人（含 AI agent）看的：站上的 SQL 代码块现在怎么写，以及几条约定。
站点本身的布局、命令与部署见 [`README.md`](./README.md)；仓库级的构建/发版流程见根目录的
[`AGENTS.md`](../AGENTS.md)。

## SQL 块是静态代码块

本扩展只构建桌面平台（见根目录 AGENTS.md 的「排除 wasm 构建」），文档站里也就没有 DuckDB-Wasm：
`docusaurus.config.ts` 的 remarkPlugins 中没有 `remarkRunnableSql`，`sql` 围栏渲染成普通的代码块，
没有 Run 按钮，也不会替读者跑任何东西。

因此写示例时要把它当「读者会自己粘贴执行」的片段：

- **不写 JSON info string**（`{"type":"duckfn",…}`）。那是可运行块的开关，这里留着它不会报错，
  只会在页面上显示成围栏的一部分。
- **前置条件写进正文**，不要假设环境。桌面平台与 `LOAD` 的来路在用户指南里讲过，块里可以直接从
  `SELECT open_path(…)` 开始，但凡是依赖前一条语句的结果（例如先 `CREATE TABLE` 再查它），就在
  同一个块里把那几条按顺序写全。
- **输出写在块外的正文里**，用一句话说明预期结果。以前由 `expect` 字段与 `npm test` 兜住的那层
  校验现在没有了：SQL 与预期结果是否对得上，改的人要自己在本地 DuckDB 里跑一遍。
- **示例数据用真实数据集**，不要自造的玩具数据（如 `[[1,2],[3,4]]`）；能一条语句讲清楚就不要铺
  一段建表。
- 想演示失败路径（比如空字符串参数）就在正文里写清报什么错，而不是配一个 `"expect":"error"` 块。

`static/` 现在只有图片。若哪天页面要读 `static/` 下的数据文件，注意 Docusaurus 会把它复制进每个
locale 的输出（英文 `/<repo>/data/x.tsv`、中文 `/<repo>/zh-Hans/data/x.tsv`），正文里的链接要按当前
语言的 baseUrl 走相对路径，写死前缀会在另一个语言下 404。

## 中英双语要成对改

站点有两个 locale（英文 + `i18n/zh-Hans/…`）。同一页在 `docs/` 与
`docs/i18n/zh-Hans/docusaurus-plugin-content-docs/current/` 下各有一份，**改了一边就
按同样的相对路径改另一边**：`id` / `slug` / `sidebar_position` 保持一致，页内链接用相对
文件路径，代码块里的 SQL 是代码、照抄，只翻注释。

## 验证

改完跑 `npm run build`（`just docs_build`）：`onBrokenLinks` 是 `throw`，中英文两侧的链接与锚点都要
能解析。页面结构或 `customFields` 动过时再跑一次 `npm run typecheck`（`just docs_typecheck`）——
它比 build 更早发现 `docusaurus.config.ts` 里的类型问题。
