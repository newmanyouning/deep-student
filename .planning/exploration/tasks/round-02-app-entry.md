# Round 02: 前端入口与路由诊断

**层级**: 1.2 — 项目骨架
**预计文件数**: 12-20
**状态**: ⏳ 待执行

## 目标

梳理前端应用入口、App Shell、路由系统和配置层。

## 扫描文件清单

### A 组：入口文件

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `src/main.tsx` | React 挂载逻辑、全局 Provider 嵌套 |
| 2 | `src/App.tsx` | 根组件、路由定义 |

### B 组：App Shell

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 3 | `src/app/shell/` 全部 .tsx 文件 | 主布局结构 |
| 4 | `src/app/navigation/` 全部 .tsx 文件 | 导航/侧边栏逻辑 |
| 5 | `src/app/components/` 全部 .tsx 文件 | Shell 级组件 |
| 6 | `src/app/services/` 全部 .ts 文件 | Shell 级服务 |

### C 组：前端配置

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 7 | `src/config/` 全部 .ts 文件 | 全局配置常量 |

### D 组：Polyfills & Shims

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 8 | `src/polyfills/` 全部文件 | 浏览器兼容补丁 |
| 9 | `src/shims/` 全部文件 | 类型声明垫片 |

## 诊断要点

1. **Provider 嵌套顺序**: 记录 Context Provider 的嵌套层级和顺序
2. **路由结构**: 识别路由模式（Hash/Browser）、路由表定义方式
3. **布局层级**: 记录 Shell → Navigation → Content 的组件层级
4. **全局配置**: 记录 config/ 中的配置常量及其用途
5. **兼容性处理**: 记录 polyfills/shims 覆盖的场景

## 输出格式

产出 `round-02-app-entry.md`：

```markdown
# Round 02: 前端入口与路由 — 诊断报告

**日期**: YYYY-MM-DD

## React 入口分析
- main.tsx 挂载流程
- Provider 嵌套层级图
- 初始化副作用

## App Shell 结构
- 布局组件树
- 导航模式
- 响应式断点

## 路由系统
- 路由模式
- 路由表定义
- 懒加载路由

## 配置常量
- [列出所有配置项及其用途]

## Polyfills / Shims
- [列出及用途]

## 发现的问题
- [ ] ...
```
