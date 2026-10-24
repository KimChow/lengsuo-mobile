import SwiftUI

/// 任务首页静态骨架（android-devguide/15 §2 首行 + §6）。
/// 吉祥物/banner/发送逻辑均为占位；⛔ 假运营数据。
public struct TasksHomePage: View {
    @State private var chips: [String] = Self.initialChips

    static let initialChips = [
        "进行数据分析及可视化",
        "进行文档处理",
        "做一份幻灯片",
        "制作工作台",
    ]

    public init() {}

    public var body: some View {
        VStack(spacing: 0) {
            topBar
            ScrollView {
                VStack(spacing: 24) {
                    greeting
                    chipGrid
                }.padding(.horizontal, 16)
            }
            inputBar
        }
    }

    private var topBar: some View {
        HStack {
            Button {} label: { Image(systemName: "line.3.horizontal") }
            Spacer()
            Text("WorkBuddy").font(.headline)
            Spacer()
            HStack(spacing: 2) {
                Text("☁ 工作模式").font(.footnote)
                Image(systemName: "chevron.right").font(.caption2)
            }.foregroundStyle(.secondary)
        }.padding(.horizontal, 16).padding(.vertical, 10)
    }

    private var greeting: some View {
        VStack(spacing: 8) {
            Image(systemName: "face.smiling").font(.system(size: 56)) // 吉祥物占位（Rive 后续卡）
            Text("WorkBuddy, 我帮你").font(.title2.bold())
        }.padding(.top, 24)
    }

    private var chipGrid: some View {
        VStack(spacing: 10) {
            ForEach(chips, id: \.self) { c in
                Text(c)
                    .font(.subheadline)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 12)
                    .background(.quaternary, in: Capsule())
            }
            Button("换一换") {
                chips.shuffle()
            }
            .font(.footnote)
        }
    }

    private var inputBar: some View {
        HStack(spacing: 12) {
            Image(systemName: "plus")
            Text("发消息或按住说话")
                .font(.subheadline).foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
            Image(systemName: "mic")
            Image(systemName: "play.fill")
        }
        .padding(.horizontal, 16).padding(.vertical, 12)
        .background(.bar)
    }
}
