import SwiftUI

/// 创建表单（与 Android 卡同字段同文案）：
/// 名称/内容 placeholder + 频率 7 档 Picker（默认每天）+ 有效期=长期有效 + 专家·技能·连接器=未选择 (0) + 创建。
/// 「创建」校验通过 ⇒ 回调插列表首位并 dismiss；名称空 ⇒ 「请填写定时任务名称」。
struct ScheduledFormView: View {
    var onCreate: (String, String, ScheduleFrequency) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var prompt = ""
    @State private var frequency: ScheduleFrequency = ScheduleFrequency.default
    @State private var showNameError = false

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    Text(LengsuoStrings.Sched.formHint)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
                Section {
                    TextField(LengsuoStrings.Sched.namePlaceholder, text: $name)
                    if showNameError {
                        Text(LengsuoStrings.Sched.errorNameRequired)
                            .font(.footnote)
                            .foregroundStyle(.red)
                    }
                    TextField(LengsuoStrings.Sched.promptPlaceholder, text: $prompt, axis: .vertical)
                        .lineLimit(4)
                }
                Section {
                    Picker(LengsuoStrings.Sched.scheduleTime, selection: $frequency) {
                        ForEach(ScheduleFrequency.allCases) { option in
                            Text(option.title).tag(option)
                        }
                    }
                }
                Section {
                    Text(LengsuoStrings.Sched.validityLongTerm)
                        .foregroundStyle(.secondary)
                } header: {
                    Text(LengsuoStrings.Sched.validity)
                }
                Section {
                    selectionRow(LengsuoStrings.Sched.experts)
                    selectionRow(LengsuoStrings.Sched.skills)
                    selectionRow(LengsuoStrings.Sched.connectors)
                }
            }
            .navigationTitle(LengsuoStrings.Sched.fabAdd)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(LengsuoStrings.Sched.sheetCancel) { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(LengsuoStrings.Sched.create) { handleCreate() }
                }
            }
        }
    }

    /// 专家/技能/连接器行：真实选择属后续卡，本卡占位「未选择 (0)」。
    private func selectionRow(_ title: String) -> some View {
        HStack {
            Text(title)
            Spacer()
            Text(LengsuoStrings.Sched.noSelection)
                .foregroundStyle(.secondary)
        }
    }

    /// 与 Android 卡一致：名称与内容均非空才可创建；名称空 ⇒ 「请填写定时任务名称」。
    private func handleCreate() {
        guard isFormCreatable(name: name, prompt: prompt) else {
            showNameError = name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            return
        }
        onCreate(name, prompt, frequency)
        dismiss()
    }
}
