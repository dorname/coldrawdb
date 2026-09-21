# 实现任务

## [code] 发布执行
- [x] 删除 GitHub Release `v0.2.0` 与远程/本地 tag `v0.2.0`
- [x] 在当前 `main` HEAD 重建附注 tag `v0.2.0` 并推送（触发 release.yml + docker.yml）
- [x] 等待 CI 完成；核对 Release 资产与 tag 指向 HEAD
