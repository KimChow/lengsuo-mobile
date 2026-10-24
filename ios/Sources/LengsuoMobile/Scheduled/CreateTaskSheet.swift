import SwiftUI

/// 创建表单弹层：名称 + 快捷档 + 时间；提交前校验（validate()）。
struct CreateTaskSheet: View {
    @State private var draft = ScheduleDraft()
    @State private var errorMessage: String?
    var onCreated: (StoredTask) -> Void
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            Form {
                TextField("请填写任务名称", text: $draft.name)
                Picker("重复", selection: $draft.frequency) {
                    ForEach(ScheduleQuickFrequency.allCases) { f in
                        Text(f.title).tag(f)
                    }
                }
                if draft.frequency == .weekly {
                    Picker("星期", selection: $draft.weekday) {
                        Text("周一").tag(1); Text("周二").tag(2); Text("周三").tag(3)
                        Text("周四").tag(4); Text("周五").tag(5); Text("周六").tag(6); Text("周日").tag(7)
                    }
                }
                DatePicker("时间", selection: Binding(
                    get: { Calendar.current.date(from: DateComponents(hour: draft.hour, minute: draft.minute)) ?? Date() },
                    set: { c in
                        let comp = Calendar.current.dateComponents([.hour, .minute], from: c)
                        draft.hour = comp.hour ?? 8; draft.minute = comp.minute ?? 0
                    }), displayedComponents: .hourAndMinute)
                if let e = errorMessage {
                    Text(e).font(.footnote).foregroundStyle(.red)
                }
            }
            .navigationTitle("新建定时任务")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button("取消") { dismiss() } }
                ToolbarItem(placement: .confirmationAction) {
                    Button("保存") {
                        if let e = draft.validate() { errorMessage = e } else {
                            let t = ScheduledTaskStore.add(name: draft.name, draft: draft)
                            onCreated(t); dismiss()
                        }
                    }
                }
            }
        }
    }
}
