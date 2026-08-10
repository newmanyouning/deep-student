# _archive/ — 仓库内代码归档区

> 创建: 2026-08-10 19:55 CST | 分支: pr/3-pdf-reading

本目录存放**从现役代码中摘除但保留在仓库内**的文件。与 `.gitignore` 里的 `archive/` (本地弃置桶, 不进 git) 不同, 本目录**提交进仓库**。

## 规则

1. **不参与编译**: 本目录在仓库根级, `tsconfig.json` 的 `include: ["src"]` 不覆盖; tailwind content (`./src/**`)、vitest include (`tests/`+`src/**`) 均不覆盖; eslint 已加入 ignores
2. **保持原始相对路径**: 归档文件放在 `_archive/<批次>/src/...` 下, 路径与其在 `src/` 中的原位置一致, 恢复时直接移回
3. **每个批次一个 MANIFEST.md**: 记录每个文件的原路径、归档原因、恢复注意事项
4. **恢复方法**: `git mv _archive/<批次>/src/<路径> src/<路径>` + 按 MANIFEST 的"恢复注意"重新接线 (如恢复 import/注册)

## 批次索引

| 批次 | 内容 | MANIFEST |
|------|------|----------|
| 2026-08-10-dead-code | 功能精简: E 类孤儿成品 (前后端完工未接线) 归档 | [MANIFEST.md](2026-08-10-dead-code/MANIFEST.md) |
