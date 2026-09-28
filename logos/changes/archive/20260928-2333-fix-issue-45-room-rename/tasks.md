# 实现任务

## [delta] 规格变更

- [x] D1 产出 delta `deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md` — 房间重命名合同（owner 专属、name trim 后 1–64、即时一致、viewer 不可改、diagram 标题与 room.name 解耦）
- [x] D2 产出 delta `deltas/api/rooms.yaml` — `/api/v1/rooms/{roomId}` 新增 patch（renameRoom，200 RoomDetail / 403 / 404 / 422）
- [x] D3 产出 delta `deltas/test/core-S04-test-cases.md` — 新 UT（后端校验/权限）+ ST（列表卡片重命名入口、即时一致、viewer 无入口）
- [x] D4 产出 delta `deltas/scenario/core-S04-room-lifecycle.json` — 编排补 rename 步骤（owner 200 / viewer 403 / 非法名 422 / GET 回读一致）
- [x] D5 产出 delta `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 镜像重建注记（无 migration、无新 smoke 用例）
- [x] D6 读回验证：从磁盘读回全部 delta 文件确认落盘

## [code] 代码实现

- [x] C1 后端 `rename_room` service + `PATCH /rooms/{room_id}` handler（owner 校验、name 1–64、404/403/422 语义）+ UT + reporter
- [x] C2 前端房间列表卡片「重命名」入口（owner 可见）+ PATCH 调用 + 列表即时刷新 + ST + reporter
- [x] C3 S04 编排脚本补 rename 步骤并跑通
- [x] C4 全量回归：backend 测试 + 前端 lib + e2e spec-parity-d 全绿

## [deploy] 部署任务

- [x] E1 verify PASS 且用户授权后：重建 compose 镜像（前后端），确认 :9080 health/SPA 200 且 PATCH 端点可用 —— 已完成：health 200 / SPA 200 / PATCH 匿名 401（路由已注册）

## [follow-up] 收尾

- [x] F1 verify PASS 后带验收评论关闭 GitHub issue #45 —— 已评论+关闭
- [x] F2 smoke PASS 后 `openlogos archive fix-issue-45-room-rename`，随后 git push

## 人类确认点

- [x] H1 用户确认本提案后再产出 delta（/goal「继续拉取issue,并分析解决issue,直到关闭」全程授权）
- [x] H2 delta 完成后 `openlogos merge fix-issue-45-room-rename`（/goal 授权）
- [x] H3 merge 后自动提交规格文档并按合并规格分批实现（C1～C4 每批闭环），完成后自动提交代码
- [x] H4 实现完成后 `openlogos verify`（Gate 3.6 PASS：515 通过/0 失败/34 跳过，覆盖 549/549）（/goal 授权；nice -n 10 + --test-threads=2）
- [x] H5 verify PASS 后按部署方案执行部署（/goal 授权）
- [x] H6 部署完成后 `openlogos smoke`（/goal 授权）—— Gate 3.8 PASS 10/10
- [x] H7 smoke PASS 后 `openlogos archive` + `git push`（/goal 授权）
