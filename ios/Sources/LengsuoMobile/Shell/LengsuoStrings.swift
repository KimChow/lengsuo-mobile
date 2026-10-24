import Foundation

/// 主壳文案：逐字取自 android-devguide/15 §2 真机转录，⛔ 不得改动。
public enum LengsuoStrings {
    public static let tabTasks = "任务"
    public static let tabScheduled = "定时任务"
    public static let tabDiscover = "发现"
    public static let tabMe = "我的"
    public static let comingSoon = "敬请期待"
}

public extension LengsuoStrings {
    /// 定时任务页文案：与 mb-scheduled-android 卡 `sched_*` 同源，逐字一致，⛔ 不得改动。
    enum Sched {
        public static let title = "定时任务"
        public static let empty = "暂无定时任务"
        public static let fabAdd = "添加定时任务"
        public static let sheetCustom = "自定义添加"
        public static let sheetConversation = "以对话方式添加"
        public static let sheetCancel = "取消"
        public static let formHint = "定时任务仅支持「定时执行」一种触发形态"
        public static let namePlaceholder = "例如：每日同步项目动态"
        public static let promptPlaceholder = "在这里输入要让 AI 完成的内容，例如：汇总过去 24 小时的项目动态，并整理待跟进事项。"
        public static let scheduleTime = "执行频率"
        public static let create = "创建"
        public static let validity = "有效期"
        public static let validityLongTerm = "长期有效"
        public static let experts = "专家"
        public static let skills = "技能"
        public static let connectors = "连接器"
        public static let noSelection = "未选择 (0)"
        public static let cardRun = "运行"
        public static let actionMore = "更多操作"
        public static let errorNameRequired = "请填写定时任务名称"

        // 频率 7 档（顺序见 ScheduleFrequency）
        public static let frequencyDaily = "每天"
        public static let frequencyEvery30Minutes = "每 30 分钟"
        public static let frequencyEveryHour = "每 1 小时"
        public static let frequencyEvery2Hours = "每 2 小时"
        public static let frequencyEvery4Hours = "每 4 小时"
        public static let frequencyEvery8Hours = "每 8 小时"
        public static let frequencyBiweekly = "双周"
    }
}
