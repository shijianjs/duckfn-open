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
  `SELECT op_open_path(…)` 开始，但凡是依赖前一条语句的结果（例如先 `CREATE TABLE` 再查它），就在
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

**侧边栏分类的标签不在 i18n 内容目录里翻**：Docusaurus 只从英文那份目录树读 `_category_.json`（自动生成的
分类也一样），所以中文分类名要改 `i18n/zh-Hans/docusaurus-plugin-content-docs/current.json` 里的
`sidebar.<侧边栏>.category.<英文 label>` 条目 —— 那一条由 `npx docusaurus write-translations --locale zh-Hans`
抽出来（加了新分类就重跑一次，再把它翻掉）。在 i18n 内容目录下再放一份 `_category_.json` 是无效的。

`_category_.json` 只写三样：`label`、`position`、以及决定「点分类标题打开哪一页」的 `link`。`user-guide/functions`
这一份用的是 `generated-index`：Docusaurus 生成一页卡片列表（标题取分类 label，每张卡取各页 front matter 的
`description`），子项仍是分类下的五页文档。它不写 `slug`，所以地址是 Docusaurus 的默认形式
`/docs/category/functions`（中文侧 `/zh-Hans/docs/category/functions`）—— 首页、页脚与 README 里指向它的链接都按
这个地址写，改分类名或挪目录时要一起看。`generated-index` 的 `description` 同样走 `current.json`，键是
`sidebar.<侧边栏>.category.<label>.link.generated-index.description`。

不写 `link` 的分类是纯折叠项，标题点不开 —— 那是「目录里没有 index 文档」时的默认行为，不是哪一项配置写错了。

**标题里不要写 `{#自定义-id}`**：这份站点没有给 `.md` 启用 heading id 语法，MDX 会把 `{…}` 当成表达式
解析，`just docs_build` 直接失败（散文标题后面还跟行内代码时尤其容易撞上）。页内互链因此只有一种可靠的
写法 —— 链到**代码名标题**（`## op_open_path`、`## op_write_file_options` 这类），它的 slug 在中英两侧完全一样。
中文页要指回本文某个散文小节时，就写「见下文「小节名」」这种纯文字，不要造锚点。

## 验证

改完跑 `npm run build`（`just docs_build`）：`onBrokenLinks` 是 `throw`，中英文两侧的链接与锚点都要
能解析。页面结构或 `customFields` 动过时再在 `docs/` 下跑一次 `npm run typecheck`（`just` 里没有这一条，
Deploy Docs 工作流会跑它）—— 它比 build 更早发现 `docusaurus.config.ts` 里的类型问题。
