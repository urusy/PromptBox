import SwiftUI

struct MetadataSheet: View {
    @Environment(\.dismiss) private var dismiss
    let image: ImageDetail

    var body: some View {
        NavigationStack {
            List {
                if let prompt = image.positivePrompt, !prompt.isEmpty {
                    Section("プロンプト") {
                        Text(prompt)
                            .textSelection(.enabled)
                    }
                }

                if let negativePrompt = image.negativePrompt, !negativePrompt.isEmpty {
                    Section("ネガティブプロンプト") {
                        Text(negativePrompt)
                            .textSelection(.enabled)
                    }
                }

                Section("生成設定") {
                    if let modelName = image.modelName {
                        LabeledContent("モデル", value: modelName)
                    }
                    if let sampler = image.samplerName {
                        LabeledContent("サンプラー", value: sampler)
                    }
                    if let scheduler = image.scheduler {
                        LabeledContent("スケジューラー", value: scheduler)
                    }
                    if let steps = image.steps {
                        LabeledContent("ステップ", value: "\(steps)")
                    }
                    if let cfgScale = image.cfgScale {
                        LabeledContent("CFG Scale", value: String(format: "%.1f", cfgScale))
                    }
                    if let seed = image.seed {
                        LabeledContent("シード", value: "\(seed)")
                            .textSelection(.enabled)
                    }
                    LabeledContent("サイズ", value: "\(image.width) x \(image.height)")
                }

                Section("ファイル情報") {
                    LabeledContent("ファイル名", value: image.originalFilename)
                    LabeledContent("ソース", value: image.sourceTool)
                    if let modelType = image.modelType {
                        LabeledContent("モデルタイプ", value: modelType)
                    }
                    LabeledContent("ファイルサイズ", value: formatFileSize(image.fileSizeBytes))
                    LabeledContent("作成日", value: formatDate(image.createdAt))
                }
            }
            .navigationTitle("メタデータ")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("閉じる") {
                        dismiss()
                    }
                }
            }
        }
    }

    private func formatFileSize(_ bytes: Int) -> String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useMB, .useKB]
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(bytes))
    }

    private func formatDate(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}
