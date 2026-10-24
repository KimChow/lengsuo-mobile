# mb-chat-page 交付报告

任务：`mb-chat-page`（安卓对话页静态版 + 本地交互态）
worktree：`~/lengsuo-mobile-dev/worktrees/mb-chat-page`（detached HEAD，基于 main `82e37fe`）

## 一、产物清单（commits）

| commit | 内容 |
|---|---|
| `84dea46` | feat(android/chat): `strings.xml` 追加对话页文案 **17 条**（逐字照抄卡 §3.1，原有内容一行未动；`values-night/strings.xml` 仓内不存在 ⇒ 按卡不建） |
| `2f22d13` | feat(android/chat): 新包 `ui/chat/ChatPage.kt`（顶栏 / AI 提示条 / 消息区 / 工具条 5 键 / 输入区两态，纯 Compose，无导航库） |
| `a7e5e2e` | feat(android/chat): `DrawerSidebar.kt` 条目点击接线 `onOpenConversation: (Int) -> Unit = {}`（默认空实现）+ `ChatPageTest.kt` 纯数据断言 |
| `f67e933` | fix(android/chat): `ThumbDown` 不在 icons-core 集合 ⇒ `Close` 暂代；`ChatPhase` 由 internal 改 public（公开签名可见性约束） |

`ChatPage` 签名与卡 §3.2 逐字一致：`fun ChatPage(title: String, phase: ChatPhase, userMessage: String, aiReply: String)`；
示例数据 `CHAT_SAMPLE_TITLE / CHAT_SAMPLE_USER_MESSAGE / CHAT_SAMPLE_AI_REPLY` 逐字照抄 §3.3。

## 二、门禁证据（真跑）

环境：`source /home/ubuntu/android_env.sh`（gradle 8.11.1，`ANDROID_HOME=$HOME/android-sdk`），`--no-daemon`。

1. `gradle --no-daemon assembleDebug` ⇒ **BUILD SUCCESSFUL in 20s**（35 actionable tasks: 7 executed, 1 from cache, 27 up-to-date）。
   （首轮编译失败 2 处：`ThumbDown` 不在 core 集合、`ChatPhase` 可见性，见 `f67e933` 修复后通过。）
2. `gradle --no-daemon :app:testDebugUnitTest` ⇒ **BUILD SUCCESSFUL in 13s**（22 actionable tasks）。
   ⚠️ 诚实声明：app 模块无任何 testImplementation（无 junit，且本卡禁增 Gradle 依赖）⇒ Gradle 实际执行 **0 条**用例，仅完成测试源编译。
3. 断言真跑（补第 2 条的执行证据）：用编译产物直接 `java` 执行 main()，完整 classpath：
   ```
   KOTLIN_STDLIB=~/.gradle/caches/modules-2/files-2.1/org.jetbrains.kotlin/kotlin-stdlib/2.0.21/618b539767b4899b4660a83006e052b63f1db551/kotlin-stdlib-2.0.21.jar
   java -cp app/build/tmp/kotlin-classes/debugUnitTest:app/build/tmp/kotlin-classes/debug:\
   app/build/intermediates/compile_app_classes_jar/debug/classes.jar:\
   app/build/intermediates/compile_and_runtime_not_namespaced_r_class_jar/debug/processDebugResources/R.jar:\
   $KOTLIN_STDLIB com.lengsuo.mobile.ChatPageTest
   ⇒ ChatPageTest: all checks passed　exit=0
   ```
   同法复跑回归：`DrawerSidebarTest` ⇒ all checks passed，exit=0；`TasksHomeTest` ⇒ all checks passed，exit=0。
   覆盖断言（≥6 条）：
   - B1：`CHAT_SAMPLE_TITLE == CHAT_SAMPLE_USER_MESSAGE`；
   - B3：状态行两态 id 映射（SENDING=`chat_status_thinking` / DONE=`chat_status_done`）+ 输入占位两态 id 映射；
   - B7：档位两态 id 映射（SENDING=`chat_model_tier_sending` / DONE=`chat_model_tier_idle`）；
   - B4：工具条 5 键 id 列表逐字（复制/赞/踩/朗读/分享 顺序）且 size==5；`TOOLBAR_ICON_DP == 24`、`TOOLBAR_GAP_DP == 12`；
   - strings.xml 逐字对照：测试直接读 `values/strings.xml` 原文，对 17 条 `chat_*` 条目做逐字 `in` 断言（JVM 侧无法解析 `stringResource`，以「资源 id 映射 + 文件原文双保险」替代「映射函数返回字面文案」——这是唯一可行且不让文案落进 main 源码的方式）。
4. grep 自检：
   - `grep -rn "内容由 AI 生成\|正在思考\|发消息" app/src/main/java/` ⇒ **无命中**（exit=1），文案只在 strings.xml；
   - `grep -c "0x" app/src/main/java/com/lengsuo/mobile/ui/chat/ChatPage.kt` ⇒ **0**（无硬编码色值）。
5. `git diff main --stat` ⇒ 改动只落 touches 内：`ui/chat/ChatPage.kt` 新 274 行、`DrawerSidebar.kt` +7/-1、`strings.xml` +17、`src/test/.../ChatPageTest.kt` 新 166 行。

## 三、验收句自查（B1-B5 / B7）

| 句 | 结论 | 证据 |
|---|---|---|
| B1 标题 = 首条用户消息 | ✅ | 测试断言相等；顶栏标题 Text 渲染 `title` 参数，示例 = `hello` |
| B2 用户气泡右对齐 / AI 正文左对齐无气泡 | ✅（布局代码级） | `Row(horizontalArrangement = Arrangement.End)` + `surfaceVariant` 圆角底；AI 正文裸 `Text` 无背景。静态版无真机渲染验证（见 §五） |
| B3 两态状态行 | ✅ | SENDING=`chat_status_thinking`；DONE=`chat_status_done` + `KeyboardArrowRight`（core 无 ChevronRight，取同形 KeyboardArrowRight） |
| B4 工具条 5 键 24dp / 间距 12dp | ✅ | 常量 `TOOLBAR_ICON_DP=24`、`TOOLBAR_GAP_DP=12`（84px/42px ÷ 3.5），测试断言相等 |
| B5 「内容由 AI 生成」提示条 | ✅ | 通栏 `chat_ai_banner`，位于消息区上方 |
| B7 档位两态 | ✅ | SENDING=`⚡ 快速 ▾` / DONE=`auto`（id 映射 + strings.xml 逐字双断言） |

颜色 token 声明：用户气泡浅灰底 = **`MaterialTheme.colorScheme.surfaceVariant`**；页面底 `palette.pageBackground`、提示条/输入框底 `palette.odcButtonBackground` 与 `surfaceVariant`、次级文字 `palette.mainSecondText`/`mainThirdText`、停止键圆底 `MaterialTheme.colorScheme.onSurface`——全部走 token，无硬编码（grep "0x" = 0）。

## 四、占位声明（点击无操作项）

- 顶栏 ☰（desc 复用 `home_top_drawer`）、⊕ 新建对话、⋯ 更多操作：均无操作（卡 §1.3 声明占位）；
- 输入区语音（`chat_input_voice`）、`+`（`chat_input_plus`）：无操作；输入框为静态 `Box` 非真实 TextField（静态版无键盘输入）；
- 工具条 5 键（复制/赞/踩/朗读/分享）：无操作；
- ⏹ 停止键：36dp 圆形（`onSurface` 底），**点击 = 无操作占位**——两态切换由 `phase` 参数驱动（`ChatPage` 无内部状态，`f67e933` 可见性约束也印证签名即公开 API）；若后续卡宿主接入，由调用方切 phase 演示发送中→已完成。

## 五、必须声明的偏离与未做项

1. **真机/模拟器：未跑**（47 无 AVD 常驻授权），全部为静态 + JVM 断言证据；B2 的「右对齐/无气泡」为布局代码级自查，无渲染截图。
2. **`onOpenConversation` 悬空交付**：按 §3.4 分支，`DrawerSidebar` 的调用点确在 `TasksHomePage.kt` 内（`TasksHomePage.kt:191`），而该文件属本卡禁区 ⇒ 仅在 `DrawerSidebar.kt` 追加 `onOpenConversation: (Int) -> Unit = {}` 参数并在条目点击回调链上调用（既有调用零破坏）；`TasksHomePage` 未传 ⇒ 当前点击抽屉条目不会真的打开 `ChatPage`，属预期悬空（宿主接线属后续卡）。
3. **图标暂代**（core 集合缺失，仓内仅 `material-icons-core`，本卡禁增依赖；同 TasksHome「麦克风→PlayArrow」先例）：
   - 复制：`ContentCopy` ✗ → `Edit`；踩：`ThumbDown` ✗ → `Close`；朗读：`VolumeUp` ✗ → `PlayArrow`；已完成右箭头：`ChevronRight` ✗ → `KeyboardArrowRight`。赞/踩/分享/☰/⊕/⋯ 用 core 原生。desc 文案全部正确（来自 strings.xml），仅图形暂代，待扩展图标库批准后替换。
4. **`ChatPhase` 为 public**（卡写「顶层 internal」）：`ChatPage` 公开签名含此类型，Kotlin 编译器拒绝 internal 暴露 ⇒ 改 public，已在代码注释与 `f67e933` 说明。
5. **返回键退出对话页**：未做 `BackHandler`——`ChatPage` 无宿主（见第 2 条），返回行为由后续宿主页 state 切换承担。
6. 未做项清单（各属后续卡）：真实消息发送、网络请求、AI 回复生成、模型档位真实切换、持久化、导航框架、更多对话条目形态、扩展图标库替换暂代图标、`values-night/strings.xml`（仓内无此文件，按卡不建）。
