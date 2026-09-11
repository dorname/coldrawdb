# Delta: reposition-referenced-source — core-02-product-vision.md

> 目标主文档：`logos/resources/prd/1-product-requirements/core-02-product-vision.md`
> 提案：reposition-referenced-source | 日期：2026-09-11

## MODIFIED — 1.1 产品定位

### 1.1 产品定位

**coldrawdb** 是一款**自托管、浏览器端**的数据库 ER 图设计工具。整体产品理念借鉴 drawDB 与 PDManer 两个优秀的开源建模工具；开发过程中参考了 drawDB 的源码作为能力对齐参照，但未直接复用、未引用其任何代码。**全部代码均为纯 Rust 重新实现**，与二者的代码库不存在派生关系。

**一句话定位**：让数据库设计者无需账号、无需联网、无需客户端安装，即可在浏览器中完成 ER 图的全生命周期管理（设计 → 导入 → 导出 → 持久化 → 分享）。

核心差异点（相对 drawDB / PDManer 等同类工具）：

- ✅ **真实后端持久化**：基于 SQLite + actix-web，自动保存到服务端，不再受 IndexedDB 缓存清理影响
- ✅ **跨设备访问**：通过 share link 与 HTTP API 在任何浏览器加载同一张图
- ✅ **409 revision 乐观锁**：服务端检测并发覆盖，UI 引导用户决策
- ✅ **Rust 全栈**：单一语言栈（WASM 前端 + actix-web 后端），类型安全 + 部署简单
