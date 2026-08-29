# Changelog | 更新日志

All notable changes to **this independent maintenance fork** will be documented in this file.

本文件记录**独立维护版**的所有重要变更。

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

---

## 关于版本号 | Versioning

独立维护版在上游版本号末尾追加修订段：`<上游版本>-fork.<修订号>`

- `0.9.40-fork.1` — 基于上游 `0.9.40` 的第 1 个独立维护版本
- `0.9.40-fork.2`、`0.9.40-fork.3` … — 同上游基线上的后续修订
- 合并上游 `0.9.41` 后 → `0.9.41-fork.1`（上游段完整保留）

---

## [0.9.40-fork.1] — 独立维护版基线（2026-08-29）

> 自本版本起，本仓库由 [Simulink](https://github.com/newmanyouning) 独立维护，
> 基于 [helixnow/deep-student](https://github.com/helixnow/deep-student) `v0.9.40`（fork 基线 commit `a942024d`）。
> 完整 git 历史与原贡献者信息全部保留；本项目继续以 AGPL-3.0-or-later 发布。

### Added 新增
- PaddleOCR 全栈集成（含 `paddleocr_split` 分片识别）
- research 研究模块（Rust `research/` + 前端类型定义 + 迁移 SQL）
- anki_service 独立服务模块
- 写入门控体系（`vfs/write_gate.rs`、`chat_v2/write_gate.rs`、`dstu/write_gate.rs`、`qbank_write_gate.rs`）
- 云同步信封与限速（`data_governance/sync/envelope.rs`、`permit.rs`）— 修复云同步卡死，WebDAV 坚果云滑动窗口限速
- e2e 测试基建（Playwright + Tauri fixture + 冒烟/旅程用例）
- OCR 页面同步与进度落盘（`ocrPageSync.ts`、`progressFlush.ts`）
- iPad 免费签名每周续签脚本（`renew-ipad.sh`）与文档

### Changed 调整
- 应用图标全套替换为 `dist/app-icon.png` + iOS 签名配置
- 移动端云凭据存储统一至活动槽位
- 会话加载兼容性：分阶段块恢复 + 宽松校验
- 清理废弃组件与 features 目录（notes/practice/skills-management/template-management/voice-input 等归档至 `_archive/`）
- 错误类型全面标准化（~637 命令/函数，详见仓库重构记录）

### Fixed 修复
- 对话显示与流式渲染时序问题
- OCR 按钮可见性与全链路触发逻辑
- PDF 阅读相关修复（original_path 安全策略等）

### Security 安全
- 桌面端自动更新改用本仓库独立签名密钥，与上游更新通道隔离

---

## 上游历史（0.8.9 – 0.9.40）

上游版本的全部历史变更记录已从本文件移除，完整内容见
[原项目 CHANGELOG](https://github.com/helixnow/deep-student/blob/main/CHANGELOG.md)，
或直接查阅本仓库保留的完整 git 历史（`git log`，含全部原贡献者署名）。

原项目及全部历史贡献版权归 DeepStudent 原贡献者所有，
本分支在其基础上以 AGPL-3.0-or-later 继续开发，归属声明见 [NOTICE.md](NOTICE.md)。
