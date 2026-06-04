# Round 01: 根配置与入口文件诊断

**层级**: 1.1 — 项目骨架
**预计文件数**: 10-16
**状态**: ⏳ 待执行

## 目标

扫描项目根目录的配置文件，确认构建工具链、类型系统、代码规范的实际配置状态。

## 扫描文件清单

### 必读文件（逐个 Read）

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `vite.config.ts` | 构建配置: plugins, resolve.alias, manualChunks, base, server 配置 |
| 2 | `tsconfig.json` | TS 编译选项: target, module, paths, strict, skipLibCheck |
| 3 | `tsconfig.node.json` | Node 端 TS 配置 |
| 4 | `tailwind.config.js` | Tailwind 配置: content paths, theme 扩展, plugins |
| 5 | `eslint.config.js` | ESLint flat config: rules, plugins, ignores |
| 6 | `postcss.config.js` | PostCSS 插件配置 |
| 7 | `index.html` | HTML 入口: meta 标签, script 引用, CSP |
| 8 | `.env.example` | 环境变量模板: 有哪些配置项 |
| 9 | `.gitignore` | Git 忽略规则 |
| 10 | `.gitattributes` | Git 属性 |
| 11 | `.stylelintrc.json` | CSS lint 规则 |
| 12 | `.stylelintignore` | CSS lint 忽略 |
| 13 | `.release-channel` | 发布频道标记 |
| 14 | `release-please-config.json` | Release Please 配置 |
| 15 | `vitest.config.ts` | Vitest 测试配置 |
| 16 | `vitest.setup.ts` | Vitest setup |
| 17 | `.vscode/settings.json` (如有) | VS Code 工作区配置 |
| 18 | `.vsconfig` | VS 配置 |

### 快速浏览（仅看结构）

| # | 目录 |
|---|------|
| - | `public/` — 静态资源清单 |

## 诊断要点

每个文件阅读后记录以下信息：

1. **异常配置**: 不常见的配置选项、workaround、TODO 注释
2. **版本锁定**: 精确版本号 vs 范围版本
3. **历史遗留**: 注释掉的代码、迁移过程中的临时配置
4. **不一致**: 与其他配置文件矛盾的地方

## 输出格式

产出 `round-01-root-config.md`，结构如下：

```markdown
# Round 01: 根配置与入口文件 — 诊断报告

**日期**: YYYY-MM-DD
**执行人**: [AI/人]

## 构建系统 (Vite)
- 配置摘要
- 发现的插件列表
- resolve.alias 路径映射
- 异常点

## TypeScript 配置
- 编译目标
- strict 模式状态
- paths 映射
- 已知 TS 错误数量及原因

## 代码规范 (ESLint / Stylelint)
- 规则集
- 自定义规则/插件
- 忽略范围

## 环境变量
- 配置项清单及分类

## 发现的问题
- [ ] 问题1
- [ ] 问题2
- ...

## 建议优先处理
1. ...
```
