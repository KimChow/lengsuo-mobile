# mb-drawer-sidebar 报告 —— 安卓「任务」页侧栏抽屉（静态 + 本地交互）

日期：2026-10-24　worktree：`~/lengsuo-mobile-dev/worktrees/mb-drawer-sidebar`（基于 main = `159ea01`，detached HEAD，未 push）

## 一、产物清单（4 commit，均在 main 之上）

| commit | 内容 |
| --- | --- |
| `8a00611` feat(android/drawer): 侧栏抽屉文案资源 4 条 | `strings.xml` 追加 `drawer_cloud_group` / `drawer_new_task` / `drawer_empty_cloud` / `drawer_task_group`（文案逐字取自实拍转录） |
| `8f4a778` feat(android/drawer): DrawerSidebar 抽屉面板 | 新建 `ui/drawer/DrawerSidebar.kt`（面板 + 遮罩 + 示例数据 + 纯函数谓词） |
| `e909f1e` feat(android/drawer): 首页 ☰ 接入抽屉与数据断言 | `TasksHomePage.kt` 顶栏 ☰ 加 onClick + Box 叠加抽屉；新建 `DrawerSidebarTest.kt` |
| `7f4c5da` fix(android/drawer): 注释改写 | 注释中与文案同字样的词改写，避免验收 grep 误命中 |

`git diff main --stat` 核实**全部改动只落在本卡允许路径内**（4 文件 +258 行）：
`ui/drawer/`（新）、`ui/home/TasksHomePage.kt`（+19）、`res/values/strings.xml`（+4）、`src/test/`（新）。
共享区（Cargo.toml / Cargo.lock / rust-toolchain.toml / scripts / .github）**零改动**；**零新增 Gradle 依赖**。
`MainShell.kt` 未改（抽屉从 TasksHomePage 顶栏打开，不经 MainShell）。报告文件本身未提交（不在允许路径）。

## 二、门禁与构建证据（真跑）

环境：`ANDROID_HOME=$HOME/android-sdk`，gradle 8.11.1，`--no-daemon`。

1. `gradle --no-daemon assembleDebug` ⇒ **BUILD SUCCESSFUL in 14s**（35 actionable tasks: 4 executed, 31 up-to-date）。
2. `gradle --no-daemon :app:testDebugUnitTest` ⇒ **BUILD SUCCESSFUL in 9s**（22 actionable tasks）。
   ⚠️ 诚实声明：app 模块无任何 testImplementation（无 junit，且本卡禁增 Gradle 依赖）⇒ Gradle 实际执行 **0 条**用例，仅完成测试源编译。
3. 断言真跑（补第 2 条的执行证据）：用编译产物直接 `java` 执行 main()：
   ```
   java -cp app/build/tmp/kotlin-classes/debugUnitTest:app/build/tmp/kotlin-classes/debug:\
   app/build/intermediates/compile_app_classes_jar/debug/classes.jar:\
   app/build/intermediates/compile_and_runtime_not_namespaced_r_class_jar/debug/processDebugResources/R.jar:\
   <kotlin-stdlib-2.0.21.jar> com.lengsuo.mobile.DrawerSidebarTest
   ⇒ DrawerSidebarTest: all checks passed　exit=0
   ```
   同法复跑 `TasksHomeTest` ⇒ all checks passed，exit=0（回归不破）。
   覆盖 5 条断言：示例集 size==3；标题逐字 `hello/周报整理/旅行计划`；`isSelected` 真值表 `(0,0)=true/(1,0)=false/(5,null)=false`；
   `isEmptyState(emptyList())=true / isEmptyState(示例集)=false`；`DRAWER_WIDTH_DP == 340`。
4. grep 自检：
   - `grep -rn "云端\|新建任务\|暂无" app/src/main/java/` ⇒ **无命中**（exit=1），文案只在 `strings.xml`；
     为满足此条，`7f4c5da` 把 DrawerSidebar.kt **注释**里的同字样词改写（grep 会连注释一起命中）。
   - `grep -c "0x" app/src/main/java/com/lengsuo/mobile/ui/drawer/DrawerSidebar.kt` ⇒ **0**（无硬编码色值；
     颜色全走 `MaterialTheme.colorScheme` / `LocalLengsuoColors` token，遮罩用 `Color.Black.copy(alpha = 0.4f)`，条目未选中用 `Color.Transparent`）。
5. 未跑模拟器/真机（47 无 AVD，按铁律不声称真机验证）。

## 三、实现方式与占位声明（按卡 §1.4 / §1.5）

1. **抽屉实现方式**：`Box + Row 条件渲染`（`if (!visible) return` 直接显隐，省略开关动画），**未用** `ModalNavigationDrawer`。
2. **图标声明**：`Icons.Filled.Add`（core 集合）用于「+」胶囊；「☁」「▾」为 Unicode 字符直进 strings.xml（U+2601 等），非图标。
3. **占位/未做交互**（均为静态占位，数据来自写死示例 `DRAWER_SAMPLE_CONVERSATIONS`）：
   - 「☁ 云端 ▾」分组下拉：**不做真实下拉**，点击无操作；
   - 「新建任务」胶囊：点击**仅关抽屉**，无跳转；
   - 「任务 ▾」分组标题右侧列表管理图标：材料未给，**不做**；
   - 对话条目：点击 = 切换选中（本地 state，再点取消），无会话跳转；
   - 空态/有数据态：由同一 state 驱动，`conversations.isEmpty()` 切换（TasksHomePage 当前传非空示例 ⇒ 有数据态）。
4. **宽度取整**：实测 1201px ÷ 3.5 = 343dp，实现取 `DRAWER_WIDTH_DP = 340`。

## 四、对卡内代码的必要偏离（均在此声明）

1. **不新增 `drawer_open`**：复用既有 `home_top_drawer`（同文案「打开侧栏」，顶栏 ☰ 沿用），strings.xml 实追加 4 条（卡内注已许可，二选一）。
2. `TasksHomePage.kt` 除卡面给的 3 处改动（头部状态、Box 包裹、☰ onClick）外，另加了一个
   `selectedConversation` 本地 state（§1.5 要求条目点击切换选中态，该 state 必须活在调用方；`onSelectConversation` 再点取消）。
   Column 未整体重排缩进（遵守「其余逻辑一行不动」，Kotlin 语法不受缩进影响）。
3. `isSelected` / `isEmptyState` / `DRAWER_WIDTH_DP` / `DRAWER_SAMPLE_CONVERSATIONS` 提为 `internal` 供测试断言（照 TasksHomeTest 先例）。
4. 抽屉可见性关闭即整体不组合（`if (!visible) return`），符合「直接显隐可省略动画」的卡面许可。

## 五、未做项清单（各属后续卡，本卡禁区）

真实会话数据、网络请求、云端/本地分组真实切换、新建任务跳转（接会话流）、列表管理入口、
持久化、抽屉开合动画、B1-B5/B7 对话页条目（属对话页卡）。
