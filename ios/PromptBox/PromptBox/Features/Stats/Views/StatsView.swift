import SwiftUI

struct StatsView: View {
    @State private var viewModel = StatsViewModel()

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading && viewModel.stats == nil {
                    ProgressView("読み込み中...")
                } else if let error = viewModel.errorMessage, viewModel.stats == nil {
                    ContentUnavailableView {
                        Label("エラー", systemImage: "exclamationmark.triangle")
                    } description: {
                        Text(error)
                    } actions: {
                        Button("再試行") {
                            Task { await viewModel.loadStats() }
                        }
                    }
                } else if let stats = viewModel.stats {
                    statsContent(stats)
                }
            }
            .navigationTitle("統計")
            .refreshable {
                await viewModel.loadStats()
            }
        }
        .task {
            await viewModel.loadStats()
        }
    }

    @ViewBuilder
    private func statsContent(_ stats: Stats) -> some View {
        List {
            Section("概要") {
                StatRow(label: "画像数", value: "\(stats.overview.totalImages)")
                StatRow(label: "お気に入り", value: "\(stats.overview.totalFavorites)")
                StatRow(label: "評価済み", value: "\(stats.overview.totalRated)")
                StatRow(label: "未評価", value: "\(stats.overview.totalUnrated)")
                if let avgRating = stats.overview.avgRating {
                    StatRow(label: "平均評価", value: String(format: "%.2f", avgRating))
                }
            }

            Section("評価分布") {
                ForEach(stats.byRating) { item in
                    HStack {
                        HStack(spacing: 2) {
                            if item.rating == 0 {
                                Text("未評価")
                            } else {
                                ForEach(1...item.rating, id: \.self) { _ in
                                    Image(systemName: "star.fill")
                                        .foregroundStyle(.yellow)
                                        .font(.caption)
                                }
                            }
                        }
                        Spacer()
                        Text("\(item.count)")
                            .foregroundStyle(.secondary)
                    }
                }
            }

            if !stats.byModelName.isEmpty {
                Section("使用モデル (Top 10)") {
                    ForEach(stats.byModelName) { model in
                        HStack {
                            Text(model.name)
                                .lineLimit(1)
                            Spacer()
                            Text("\(model.count)")
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }

            if !stats.bySampler.isEmpty {
                Section("サンプラー (Top 10)") {
                    ForEach(stats.bySampler) { sampler in
                        HStack {
                            Text(sampler.name)
                                .lineLimit(1)
                            Spacer()
                            Text("\(sampler.count)")
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }

            if !stats.byLora.isEmpty {
                Section("LoRA (Top 10)") {
                    ForEach(stats.byLora) { lora in
                        HStack {
                            Text(lora.name)
                                .lineLimit(1)
                            Spacer()
                            Text("\(lora.count)")
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }

            if !stats.bySourceTool.isEmpty {
                Section("ソースツール") {
                    ForEach(stats.bySourceTool) { tool in
                        HStack {
                            Text(tool.name)
                            Spacer()
                            Text("\(tool.count)")
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }

            if !stats.byModelType.isEmpty {
                Section("モデルタイプ") {
                    ForEach(stats.byModelType) { modelType in
                        HStack {
                            Text(modelType.name)
                            Spacer()
                            Text("\(modelType.count)")
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }
        }
    }
}

struct StatRow: View {
    let label: String
    let value: String

    var body: some View {
        HStack {
            Text(label)
            Spacer()
            Text(value)
                .foregroundStyle(.secondary)
        }
    }
}

#Preview {
    StatsView()
}
