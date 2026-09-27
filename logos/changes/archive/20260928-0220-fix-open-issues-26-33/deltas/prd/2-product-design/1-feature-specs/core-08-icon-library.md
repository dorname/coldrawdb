# Delta — core-08-icon-library.md（fix-open-issues-26-33 / #33）

> 模块：core | 提案：fix-open-issues-26-33
> 目标：语义图标（PK / FK / NOT NULL / UNIQUE）与 ToolRail 图标统一视觉语言——尺寸、描边、色板、激活态一致，避免「字块角标」与线型图标混搭。

## ADDED — §11 语义图标统一口径（fix-open-issues-26-33 / #33）

> 现状：PK 等徽章为字块角标风格，与 ToolRail 线型图标、关系端点记号视觉语言不统一。本节定义统一口径，参数真值见 `core-07-design-tokens.md` §15.5。

| 规则 | 规格 |
|---|---|
| 徽章族 | PK / FK / NOT NULL / UNIQUE 徽章为同一图标族：统一外接尺寸（`canvas.badge.size` 12px）、统一描边（1.5px）、统一圆角与内边距；优先使用线条图标（钥匙 / 链环 / 星号 / 菱形）而非文字块 |
| 徽章色板 | 仅从设计 token 语义色阶取值（PK=warning 系、FK=info 系、NN/UQ=neutral 系）；亮/暗主题成对；禁止散值 hex |
| 关系端点 | 基数记号（一/多）采用同一 crow's foot 几何族，尺寸 `canvas.rel.endpoint-size`；与选中态、LOD 线宽补偿（R-LOD-04）协同缩放 |
| ToolRail | 全部工具图标统一 stroke（1.5px）与尺寸（20px / IconBox 24px）；激活态 = 主色描边 + `primary.1` 浅底；新增/替换图标须先登记本规格 §4 清单 |
| 清单更新 | §4.4 画布对象与 §4.5 字段类型徽章清单按本口径核对：缺失的统一族图标（如 FK 徽章独立图标）以 ADDED 补登；被替换的字块角标注记为 deprecated（保留兼容渲染一个迭代） |
| 落地顺序 | 主原型先行（`core-01-editor-prototype.html`），前端 canvas 绘制与 SVG 组件随后对齐；原型与生产共用同一 token 真值 |

**验收口径**：画布上任取表卡、PK/FK 徽章、关系端点、ToolRail 图标，目视同属一套视觉语言（尺寸/描边/色板一致）；亮/暗主题切换无风格漂移。
