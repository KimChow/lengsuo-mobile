# mb-tasks-home 报告 —— 安卓「任务」Tab 首页静态版

日期：2026-10-24　worktree：`~/lengsuo-mobile-dev/worktrees/mb-tasks-home`（基于 main = `1d5ef2c`，detached HEAD，未 push）

## 一、产物清单（commit，均在 main 之上）

| commit | 内容 |
| --- | --- |
| `b62736d` feat(android/home): 任务首页文案资源 15 条 | `strings.xml` 追加 15 条 `home_*` 文案 |
| `2ae9e35` feat(android/home): TasksHomePage 静态首页 | 新建 `ui/home/TasksHomePage.kt` |
| `f692bef` feat(android/home): MainShell 接入任务 Tab 首页与数据断言 | `MainShell.kt` 接线 + 新建 `TasksHomeTest.kt` |
| `0df6c01` fix(android/home): 语音图标改 PlayArrow 占位 | KeyboardVoice 不在 icons-core，修正 |

相对 main 全部改动共 4 个文件 +270 行，`git diff 1d5ef2c..HEAD --stat` 核实**全部落在本卡允许路径内**：
`ui/home/`（新文件）、`ui/shell/MainShell.kt`（+2 行：1 行 import + 1 行 when 分支，其余零改动）、
`res/values/strings.xml`（+15）、`src/test/`（新文件）。
共享区（Cargo.toml / Cargo.lock / rust-toolchain.toml / scripts / .github）与 Gradle 依赖**零改动**；报告文件本身未提交（不在允许路径）。

## 二、门禁与构建证据（真跑）

环境：`ANDROID_HOME=$HOME/android-sdk`，gradle 8.11.1（wrapper）。

1. `./gradlew --no-daemon assembleDebug` ⇒ **BUILD SUCCESSFUL in 12s**（35 actionable tasks: 7 executed, 1 from cache, 27 up-to-date）。
   （首次跑 FAILED，原因 `Unresolved reference 'KeyboardVoice'`，见 §四图标占位，修正后成功。）
2. `./gradlew --no-daemon :app:testDebugUnitTest` ⇒ **BUILD SUCCESSFUL in 12s**（22 actionable tasks）。
   ⚠️ 诚实声明：app 模块**没有任何 testImplementation**（无 junit），且本卡禁增 Gradle 依赖、`app/build.gradle.kts` 不在允许路径
   ⇒ Gradle 实际执行 **0 条**用例，仅完成了测试源编译。
3. 断言真跑（补第 2 条的执行证据）：用编译产物直接 `java` 执行 `TasksHomeTest.main`：
   ```
   java -cp app/build/tmp/kotlin-classes/debugUnitTest:app/build/tmp/kotlin-classes/debug:\
   app/build/intermediates/compile_app_classes_jar/debug/classes.jar:\
   app/build/intermediates/compile_and_runtime_not_namespaced_r_class_jar/debug/processDebugResources/R.jar:\
   <kotlin-stdlib-2.0.21.jar> com.lengsuo.mobile.TasksHomeTest
   ⇒ TasksHomeTest: all checks passed　exit=0
   ```
   覆盖 4 条断言：胶囊集大小=4；顺序与文案资源一一对应；`cycle(1)` 整体左移一位；`cycle(0)/cycle(4)` 回原序。
   15 个 `R.string.home_*` 资源 id 由编译期校验（assembleDebug 通过即证存在）。
4. grep 自检：`grep -rn "数据分析\|幻灯片\|我帮你\|超能力" android/app/src/main/java/` ⇒ **无命中**（exit=1），文案只在 `strings.xml`。
5. 主题：代码路径只用 `MaterialTheme` token 与 `LocalLengsuoColors` token，本卡新代码**无任何硬编码 `Color(0xFF…)`**；
   `LengsuoColors` 明暗两套字段（`odcButtonBackground`/`mainSecondText`）均有 night 值。
   未跑模拟器（47 无 AVD，按铁律不声称真机验证）。

## 三、占位声明（按卡 §1.5 / §1.6）

1. **图标占位**：material-icons-core 1.7.5 AAR 实测仅 47 个 Filled 图标（解包 `classes.jar` 列举核实），
   **无 Mic、无 KeyboardVoice**（卡片给出的备选 KeyboardVoice 编译失败）。落地：
   抽屉=`Icons.Filled.Menu`（core 有）、附件=`Icons.Filled.Add`（core 有）、
   麦克风=**`Icons.Filled.PlayArrow` 占位**（语义近参考设计的 ⏯ 位），待共享区批准 `material-icons-extended` 后另立卡换 `Icons.Filled.Mic`。
   「☁ 工作模式 >」为纯文本（U+2601 Unicode 直进 strings.xml），非图标。
2. **Rive 占位**：3D 吉祥物为 48dp 高、`odcButtonBackground` 圆角块 + 居中灰字「WorkBuddy 吉祥物」，未引 Rive 依赖。
3. **banner 占位**：文案「限时体验：组建你的专属助理团队」为中性占位（材料无运营逐字文案，运营位走 COS 下发契约）；
   右上 × 可关闭（`remember` 本地态）。`home_banner_dismiss`（「关闭」）已预留，后续可作无障碍 contentDescription。
   卡内代码 × 按钮为 `Text("×")`，沿用。

## 四、对卡内代码的必要偏离（均已在 commit message / 此处声明）

1. `LocalLengsuoColors` 无 `cardBackground`/`textSecondary` 字段（按卡规则只改名不造色）：
   → `odcButtonBackground`（次级面背景，light #EBF0F7 / dark #4D5768）、`mainSecondText`。
2. 卡内「换一换」Row 同时渲染 Label+TextButton 两条「换一换」，疑似笔误 ⇒ 保留右侧一条 `TextButton`（`Spacer(weight)` 右对齐），文案未变。
3. `trailingIcon` lambda 内多余逗号语法错误 ⇒ 去除（卡内已有 ⚠️ 预案）。
4. `HOME_CAPSULES`/`cycle` 提为 `internal`（卡内 §3.4 预案）供测试断言。
5. 测试无 junit 可用 ⇒ 纯 Kotlin 断言 + `main()` 可执行（见 §二.2/2.3）。

## 五、未做项清单（各属后续卡，本卡禁区）

抽屉侧栏、工作模式切换弹层、会话流与输入发送（胶囊 onClick 留空、输入栏 `enabled=false`）、
网络请求、持久化、Rive 吉祥物动效、真实运营 banner 下发、`material-icons-extended` 图标替换。
