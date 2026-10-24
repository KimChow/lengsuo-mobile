import Foundation

/// 抽屉文案：逐字取自本仓 android strings.xml drawer_*，⛔ 不得改动。
public enum DrawerCopy {
    public static let cloudGroup = "☁ 云端 ▾"
    public static let newTask = "新建任务"
    public static let emptyCloud = "暂无云端任务"
    public static let taskGroup = "任务 ▾"

    /// 示例对话（DrawerSidebar.kt DRAWER_SAMPLE_CONVERSATIONS 同款写死，后续卡接真实会话流）。
    public static let sampleConversations = ["hello", "周报整理", "旅行计划"]
}