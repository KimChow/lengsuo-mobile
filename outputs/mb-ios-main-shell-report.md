# mb-ios-main-shell 交付报告

- 卡片：`mb-ios-main-shell`（iOS 主壳：底部四 Tab 导航骨架，SwiftUI，iOS 17）
- worktree：`~/lengsuo-mobile-dev/worktrees/mb-ios-main-shell`（108 Mac mini，Intel x86_64，Xcode 15.4 + iOS 17.5 SDK/运行时）
- 分支：`mb-ios-main-shell`（本卡分支，未 push / 未合流）
- 基线：`be135a0`（= main @ 骨架卡 `6e3e3b1` 之后）
- 交付 commits（3 个，按铁律分批）：
  - `153775b` feat(ios/shell): MainTab 四 Tab 枚举 + LengsuoStrings 逐字文案常量
  - `f910151` feat(ios/shell): MainShell 底部四 Tab 主壳（TabView + 占位页）
  - `7a26d6c` test(ios/shell): MainShellTests 纯数据断言（Tab 顺序/首 Tab/文案）
- 本报告不入 git（按任务书要求）。

## 一、交付物清单（全部按任务书 §三 逐字照抄，零改动）

| 文件 | commit | 说明 |
|---|---|---|
| `ios/Sources/LengsuoMobile/Shell/LengsuoStrings.swift` | 153775b | 文案常量（5 条，逐字取自 android-devguide/15 §2） |
| `ios/Sources/LengsuoMobile/Shell/MainTab.swift` | 153775b | 四 Tab 枚举（CaseIterable 顺序 = 任务→定时任务→发现→我的） |
| `ios/Sources/LengsuoMobile/Shell/MainShell.swift` | f910151 | TabView 四项 + 选中态 `@State selection = .tasks` + 占位页 |
| `ios/Tests/LengsuoMobileTests/MainShellTests.swift` | 7a26d6c | 纯数据断言 3 例（Tab 顺序 / 首 Tab / 文案逐字） |

既有冒烟 `LengsuoMobileTests.swift` 未动（铁律：只增不改）。`Package.swift` 未动（铁律 5：零依赖，targets/products 原样）。

## 二、四 Tab 对齐声明（android-devguide/15 §2）

Tab 顺序自左到右 = **任务 → 定时任务 → 发现 → 我的**，与真机转录逐字一致：

| 序 | Tab 文案 | 枚举 case | 上位文档依据（§2 逐字摘录） |
|---|---|---|---|
| 1 | 任务 | `.tasks` | 「任务（AI 助手首页）｜会话入口」 |
| 2 | 定时任务 | `.scheduled` | 「定时任务｜自动化列表｜闹钟图标（分册11）」 |
| 3 | 发现 | `.discover` | 「发现｜生态市场｜搜索 + 资料库/专家/技能/连接器/项目（分册12）」 |
| 4 | 我的 | `.me` | 「我的｜个人中心」 |

- 首 Tab = 任务（`selection` 初值 `.tasks`，`MainTab.allCases.first == .tasks` 有测试锁定）。
- 安全区：iOS 形态 = 系统 `TabView` 底部 Tab 栏，高度由 UIKit 自管，未硬编码 175px（任务书 §1.2 语义对齐）。
- 4 个 Tab 本卡全部为占位页 = 居中「<Tab 名> · 敬请期待」，零假数据（铁律 7）。
- 颜色只用系统语义色（`.primary`/`.secondary`）与 SwiftUI 默认背景，零硬编码色值，主题体系留给 mb-ios-theme。

## 三、图标占位差异声明（任务书 §1.4，近似占位，不冒充 1:1）

参考端图标族（闹钟、抽屉等）在 SF Symbols 无逐一对应实证，本卡图标为**近似占位**：

| Tab | 本卡 SF Symbol | 参考端 |
|---|---|---|
| 任务 | `house` | AI 助手首页（无对应实证） |
| 定时任务 | `calendar` | 闹钟图标（分册11，无对应实证） |
| 发现 | `magnifyingglass` | 搜索 + 生态市场（无对应实证） |
| 我的 | `person` | 个人中心（无对应实证） |

全部为系统 SF Symbols，零第三方依赖、零资源文件。正式图标体系由后续设计卡替换。

## 四、验收证据（全部真跑，108 本机）

### 4.1 验证通道偏离声明（重要）

任务书 §四 写的裸命令 `cd ios && swift build && swift test` **在宿主 macOS 上对本包不可行**，实测证据：

```
$ swift build --package-path ios          # 宿主裸跑
ios/Sources/LengsuoMobile/Shell/MainShell.swift:33:10: error:
  'frame(minWidth:idealWidth:maxWidth:...)' is only available in macOS 10.15 or newer
error: fatalError        # EXIT=1
```

原因：`Package.swift` 声明 `platforms: [.iOS(.v17)]`（iOS-only，骨架卡 `6e3e3b1` 既有设定，铁律 5 禁改），宿主构建回落到旧 macOS 部署目标，SwiftUI（TabView/frame 等）全部不可用。骨架卡当时能过是因为根模块只 import Foundation。修法（platforms 加 `.macOS(.v14)`）属于 Package.swift 改动，**本卡未动，移交基建卡决策**。

因此本卡采用的真实验证通道（Package.swift 零改动）：

1. **构建**：`swift build --destination <iOS 17.5 Simulator SDK>` —— 命令仍是 `swift build`，加 `--destination` 指向 iPhoneSimulator17.5 SDK，产物为 iOS 模拟器目标；
2. **测试**：`xcodebuild test -scheme LengsuoMobile -destination 'platform=iOS Simulator,name=iPhone 15,OS=17.5'` —— 官方支持的 SwiftPM 包 iOS 测试通道。

另注：`swift test --destination <同上>` 在 SwiftPM 5.10 下**只构建不执行**（链接成功后输出 `Build complete! (1.18s)` 即静默 EXIT=1，`--verbose` 日志在 Build complete 后无任何运行命令）——这是 SwiftPM 对自定义模拟器 destination 的能力缺口，非本包问题；测试目标在其下已能完整编译+链接（补 `-F`/`-I`/`-L` 指向平台 XCTest.framework 与 XCTest.swiftmodule 后），说明包本身无问题。

### 4.2 swift build（--destination，iOS 17.5 Simulator SDK）真实输出尾部

```
$ swift build --package-path ios --destination /tmp/ios-sim-destination.json
Building for debugging...
[0/2] Write sources
[1/2] Write swift-version-33747A42983211AE.txt
[3/6] Emitting module LengsuoMobile
[4/6] Compiling LengsuoMobile LengsuoStrings.swift
[5/6] Compiling LengsuoMobile LengsuoMobile.swift
[6/6] Compiling LengsuoMobile MainShell.swift
[7/7] Compiling LengsuoMobile MainTab.swift
Build complete! (8.65s)
```

**EXIT=0，Build complete。**

### 4.3 测试执行（xcodebuild test，iPhone 15 · iOS 17.5 模拟器）真实输出

```
$ cd ios && xcodebuild test -scheme LengsuoMobile \
    -destination 'platform=iOS Simulator,name=iPhone 15,OS=17.5' \
    -derivedDataPath /tmp/mb-ios-dd
Test Suite 'All tests' started at 2026-10-24 19:33:08.240.
Test Suite 'LengsuoMobileTests.xctest' started at 2026-10-24 19:33:08.242.
Test Suite 'LengsuoMobileTests' started at 2026-10-24 19:33:08.242.
Test Suite 'LengsuoMobileTests' passed at 2026-10-24 19:33:08.249.
	 Executed 1 test, with 0 failures (0 unexpected) in 0.005 (0.007) seconds
Test Suite 'MainShellTests' started at 2026-10-24 19:33:08.250.
Test Suite 'MainShellTests' passed at 2026-10-24 19:33:08.290.
	 Executed 3 tests, with 0 failures (0 unexpected) in 0.038 (0.040) seconds
Test Suite 'LengsuoMobileTests.xctest' passed at 2026-10-24 19:33:08.290.
	 Executed 4 tests, with 0 failures (0 unexpected) in 0.043 (0.049) seconds
Test Suite 'All tests' passed at 2026-10-24 19:33:08.291.
	 Executed 4 tests, with 0 failures (0 unexpected) in 0.043 (0.051) seconds
** TEST SUCCEEDED **
```

**EXIT=0。Executed 4 tests, 0 failures（原冒烟 1 例 + 新增 MainShellTests 3 例）**，与 §四 验收指标一致。

说明：测试为纯数据断言（枚举顺序/文案常量），跑在模拟器进程内但**不含 UI 视觉验收**；模拟器截图/真机验证属 L3 另卡，本卡不声称（铁律 8）。

### 4.4 destination JSON 全文（复现用；放在 /tmp，不入仓库）

```json
{
  "version": 1,
  "sdk": "/Applications/Xcode-15.4.0.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator17.5.sdk",
  "target": "x86_64-apple-ios17.5-simulator",
  "toolchain-bin-dir": "/Applications/Xcode-15.4.0.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin",
  "extra-cc-flags": [],
  "extra-swiftc-flags": [
    "-F", "/Applications/Xcode-15.4.0.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/Library/Frameworks",
    "-I", "/Applications/Xcode-15.4.0.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/usr/lib",
    "-L", "/Applications/Xcode-15.4.0.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/usr/lib"
  ],
  "extra-cxx-flags": [],
  "extra-cpp-flags": [],
  "extra-linker-settings": [],
  "dynamic-library-extension": "dylib"
}
```

（各字段均为 SwiftPM 5.10 必填；`-F/-I/-L` 为让测试目标找到平台 XCTest 框架与 Swift overlay，仅影响 swift test --destination 通道，swift build 不依赖后三个路径。）

## 五、行数自查（门禁 ≤400 软 / 700 硬）

```
$ wc -l ios/Sources/LengsuoMobile/Shell/*.swift ios/Tests/LengsuoMobileTests/MainShellTests.swift
      10 ios/Sources/LengsuoMobile/Shell/LengsuoStrings.swift
      35 ios/Sources/LengsuoMobile/Shell/MainShell.swift
      30 ios/Sources/LengsuoMobile/Shell/MainTab.swift
      18 ios/Tests/LengsuoMobileTests/MainShellTests.swift
      93 total
```

全部远低于门禁 ✅。

## 六、边界自查

- 只新增上述 4 个文件；`git status` 干净（.build/ 被 gitignore，DerivedData 放 /tmp）。
- 未触碰：`Package.swift`、`Cargo.toml`/`Cargo.lock`/`rust-toolchain.toml`、`scripts/`、`.github/`、`android/`、`ffi/`、既有冒烟测试。
- 未新增依赖（零第三方库，仅系统 SwiftUI/Foundation/XCTest）。
- 未 push / 未合流 / 未动主仓。
- 5 条文案逐字取自 android-devguide/15 §2 真机转录，未自编（`LengsuoStrings.swift` 内有⛔注释锁定，测试 `testTabOrderMatchesSpec`/`testComingSoonCopyVerbatim` 双重锁定）。

## 七、移交事项（给主理人 / 后续卡）

1. **Package.swift 需决策**（基建卡）：若希望宿主裸 `swift build`/`swift test` 可用，需在 `platforms` 加 `.macOS(.v14)`（或改用 xcodebuild 统一验收口径）；本卡遵守铁律未动，仅报告。
2. **图标**：SF Symbols 近似占位（§三 声明），正式图标族待设计卡。
3. **Me 页与其余三 Tab 实页**：全部为占位页，由后续卡逐个接入（Android 线 mb-main-shell 仅挂了「我的」实页，iOS 线 Me 页未建）。
4. **主题体系**：mb-ios-theme 卡接手（本卡只用系统语义色）。
5. **UI 视觉验收**：L3 另卡（本卡只做编译 + 单测断言）。
