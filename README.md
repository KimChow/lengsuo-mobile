# lengsuo-mobile

棱梭移动端 companion App（原生 Swift / Kotlin）+ Rust 协议栈 FFI 绑定层。

> **文档不入本仓**：`docs/` 全部只保留在本地开发工作区（用于开发进展与验收记录），
> 由独立 git 仓同步到 GitLab 私有仓；**不随本仓库分发，也不保留在本仓 git 历史中**。
> 需要查阅请找开发方索取。

- 远端：GitHub `KimChow/lengsuo-mobile`（**公开**，源码可查看）。
- 定位：移动端是 **companion**，远程授权范围 = **会话查看 / 工具审批 / 通知**
  （见 `../lengsuo/docs/v1-scope-freeze.md` N11）。
- 连接形态：**中继先通 → P2P 直连升级**；中继是**常备兜底**，不是过渡品；
  两者共用同一套端到端加密与授权模型。
- 协议栈**不在本仓**：跨端链路协议核心在 `../lengsuo-client`（远端 `KimChow/lengsuo-network`）
  的 `crates/lengsuo-link-core`（**无 IO**，编成静态库经 FFI 调用）。
  ⇒ **协议只有一份实现**，移动端不重写密码学。

## 目录

| 路径 | 内容 | 状态 |
|---|---|---|
| `ffi/` | Rust ⇄ Swift/Kotlin 绑定层（`uniffi` 或手写 C ABI，**待定**） | 骨架 |
| `ios/` | Swift App 壳（UI / 安全存储 / 推送） | 骨架 |
| `android/` | Kotlin App 壳（同上） | 骨架 |
| `docs/` | 跨端协议摘要、移动端设计（仅本地，不入库） | 骨架 |

## 构建现状

| 目标 | 环境 | 状态 |
|---|---|---|
| Rust `ffi/` | 任意带 Rust 工具链处 | ✅ 可编译测试 |
| Android | 需 Android SDK + NDK | ⚠️ 待装 SDK |
| iOS | **需 macOS + Xcode** | ⛔ **当前环境不可编译**（47 无 Xcode；本机无 macOS） |

⇒ iOS 相关产出必须写明「只做静态检查，无真机构建/真机截图」，
协议一致性靠**跨端测试向量**保证，不靠真机。

## 文档

- 协作规则与硬约束：[AGENTS.md](AGENTS.md)。
- 跨端链路的仓库归属定稿：`../lengsuo-client/outputs/repo-layout-decision.md`。
- 完整落地规划：`../lengsuo-client/outputs/cross-endpoint-p2p-plan.md`。

源码可查看不等于授予开源许可。
