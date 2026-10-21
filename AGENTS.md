# lengsuo-mobile AI 与协作规则

## 1. 仓库归属（用户已确定）

- 本地：`D:/OneDrive/project/lengsuo-mobile`；远端：GitHub `KimChow/lengsuo-mobile`（**公开**，源码可查看）。
- 本仓承载：**移动端 companion App**（原生 Swift / Kotlin）+ **Rust 协议栈 FFI 绑定层**。
- **2026-10-21 用户拍板**：本仓**公开**（GitHub）。连带约束见 §3。
- 兄弟仓：`../lengsuo-client`（远端 `KimChow/lengsuo-network`，公开；**跨端协议栈在此仓**
  的 `crates/lengsuo-link-core`）、`../lengsuo-agent`（公开；智能体运行时）、
  `../lengsuo-browser`（公开；Chromium 内核）、`../lengsuo`（GitLab 私有 `sales/lengsuo-core`；
  私有业务与服务端权威逻辑、**协调面与中继**）。
- 跨端链路的仓库归属定稿见 `../lengsuo-client/outputs/repo-layout-decision.md`。

## 2. 依赖方向（不可越界）

```
lengsuo-network（公开）
  crates/lengsuo-link-core   ← 跨端协议核心（无 IO）
        │ git revision 固定
        ▼
  lengsuo-mobile/ffi/         ← 本仓的 Rust 绑定层
        │ FFI（C ABI 或 uniffi）
        ▼
  ios/ (Swift)   android/ (Kotlin)
```

- ⛔ **本仓不得反向依赖** `lengsuo-network` 之外的任何 lengsuo 仓；
  尤其**不得**依赖 `lengsuo-agent`（智能体运行时是 PC 端的事）。
- ⛔ **移动端不依赖 `lengsuo-tunnel`**（有 IO、只在 PC 端用），也不需要八协议栈。
- 跨仓引用以**完整 Git revision 固定**；发布不得使用本地相对路径依赖。

## 3. 公开仓的额外约束

- **源码可查看 ≠ 授予开源许可**；不得描述为已开源。
- ⛔ **禁止提交**：真实凭据、私钥、签名证书、生产配置、真实用户数据、未脱敏日志/抓包、
  `*.keystore`、`*.p12`、`*.mobileprovision`、`.env*`、设备长期密钥。
  设备长期密钥必须**运行时生成**并存入平台安全存储（iOS Keychain / Android Keystore）。
- ⛔ 服务端地址可写在配置里，但**凭据必须服务端下发**，不得硬编码为秘密。
- 本仓**不含私有业务**：协调面、计费、权限判定一律留在 GitLab 私有仓。

## 4. 文件与构建限制

- 单文件 **700 行硬上限**、**400 行软目标**，空行与注释计入；**Markdown 同样计入**。
- dev/release profile 显式 `debug = 0` + `strip = "debuginfo"`。
- 禁止在生产网络代码中使用 `unsafe`（Rust 侧）；例外必须先说明并取得确认。
- ⛔ **禁止新增 GPL/AGPL/SSPL 链接依赖**；逐项核对许可。
- 门禁：
  - Rust 侧（`ffi/`）：`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、
    `cargo test --workspace --locked`。
  - Swift 侧：`swift build` / `swift test`（**需 macOS + Xcode**；无该环境时只做语法/类型检查，
    **不得声称已编译验证**）。
  - Kotlin 侧：`gradle test`（需 Android SDK/NDK）。
  - 通用：`python scripts/check_line_limit.py`（若仓内有该脚本）、
    `python scripts/check_no_source_leak.py`（若仓内有该脚本）。

## 5. 平台硬约束（**必须先认清**）

| 平台 | 可编译环境 | 状态 |
|---|---|---|
| Rust `ffi/` | Linux / Windows / macOS | ✅ 可全量编译测试 |
| Android（Kotlin/Gradle） | 需 Android SDK + NDK | ⚠️ 需装 SDK（研究机 47 上是新环境） |
| iOS（Swift/Xcode） | **需 macOS + Xcode** | ⛔ **研究机 47 无 Xcode；本机无 macOS ⇒ 当前无法编译** |

⇒ **iOS 相关工作必须**：① 写清「只做静态检查，无真机构建/真机截图」；
② 协议一致性靠**跨端测试向量**保证（Rust 侧产向量，Swift 侧跑同一批向量），
**不靠真机**；③ 不得把「未编译」写成「已验证」。

## 6. AI 工作与测试限制

- 新功能按已确认接口与测试计划实现；**先复现失败再修**；**先写测试再写实现**（TDD）。
- 写代码必须有测试用例；**基线不绿不得合并**，无测试的功能代码视为未完成。
- 文档随代码同批次更新（「代码 ↔ 文档 ↔ 测试」三者一致才算完成）。
  - **`docs/` 只留本地、不入库**：完整口径见 §7「文档放置规则」。
- 异步读写测试必须有超时；`read_to_end` 前必须确保写端关闭。
- ⛔ 不并发对同一 target 执行构建；不反复轮询或 sleep。
- ⛔ 不提交构建产物、依赖缓存、IDE 配置、`.DS_Store`、`local.properties`。
- 不部署、不改系统安全设置、不终止用户进程。

## 7. 文档放置规则（docs/ 只留本地，不入库）

**一句话：`docs/` 是本地工作区资产，只给智能体和开发方看，不随仓库分发，也不出现在 git 历史里。**

### 7.1 什么写哪里

| 内容 | 位置 | 是否入库 |
|---|---|---|
| 进度台账、验收记录、任务书、对照资料、研究稿、状态快照 | `docs/` | **不入库** |
| 对外说明、装了什么、怎么构建、当前状态 | `README.md` | 入库 |
| 协作规则、门禁、边界、验收判据 | `AGENTS.md` | 入库 |
| 代码里的行为注释、类型说明、错误码语义 | 源码内 | 入库 |

**判据一句话：写给「读者和下一个智能体」看的东西进 `docs/`；写给「装这份代码的人」看的进 README 与规则文件。**

### 7.2 硬性要求

- `docs/` 全量在 `.gitignore` 里，**取消索引跟踪**；仓库历史中**不得残留**任何 `docs/` 路径。
- 新增文档**直接写进 `docs/`**，不要先写到仓根再补 gitignore。
- 写文档前先看 `docs/` 已有同类文件，**更新而不是新建**；确需新建时用 `主题-类型.md`（如 `ffi-decision.md`、`mobile-roadmap.md`）。
- 文档里**不写对标产品名**，只用中性代号（`对标端` / `参照平台`）；来源标识只记在本地工作区。
- 提交前自检：`git status` 里**不应出现 `docs/`**；若出现说明误加，`git rm -r --cached docs` 撤回。
- 不做 `git add .` / `git add -A` 盲加；**按文件路径逐个 add**。

### 7.3 两台机器的文档同步

- 本机与 47 **各有一份 `docs/`**，内容取**并集**（本机独有 + 47 独有），不是互相覆盖。
- 同步用文件级传输，**不走 git**（git 里已经没有 docs 了）：
  ```bash
  # 本机 -> 47
  (cd <repo> && tar -cf - docs) | ssh 47 "cd <repo> && tar -xf -"
  # 47 -> 本机
  ssh 47 "cd <repo> && tar -cf - docs" | (cd <repo> && tar -xf -)
  ```
- 同步**只做文件搬运，不做删除**；清理旧文档由人工在两侧分别处理。
- 依赖 `docs/` 存在才通过的测试要加 `skipUnless(DOCS.is_dir())`，否则 CI 会因缺文件变红。

### 7.4 与门禁的关系

- `scripts/check_no_source_leak.py` 管**内容脱敏**（不许出现对标产品名）。
- 本节管**边界**（文档不进仓、不进历史）。两者独立，缺一不可。
- 公开仓的 README 顶部保留一句「文档不入库，需要请向开发方索取」，让 clone 者知道去哪找文档。

## 8. 参考来源不外泄

- 闭源对标产品的**产品名·品牌·专有文件名·内部目录名**不得进仓与 git 历史
  ⇒ 中性代号（`对标端`/`参照平台`）。**技术结论必须保留**（错误码语义·字段名·协议·判据·版本号）；
  **开源参照保留署名**（wireguard-go / RustCrypto 等，许可义务）。
  判据：**闭源→脱敏，开源→署名**。
- 门禁 `scripts/check_no_source_leak.py`（禁词表**仓外**）。

## 9. 当前状态（2026-10-21）

- 仓刚建立，**只有骨架**：目录结构 + 本文件 + README。
- 跨端协议栈依赖 `lengsuo-network` 的 `crates/lengsuo-link-core`，
  该 crate **当前只有占位实现**（协议帧编解码 `lk-frames` 卡进行中）。
- ⛔ **不得**在 `lengsuo-network` 尚无协议实现时，本仓就写死协议细节 ⇒
  协议以**跨端契约文档 + 测试向量**为准，向量未就位前不做协议相关实现。
