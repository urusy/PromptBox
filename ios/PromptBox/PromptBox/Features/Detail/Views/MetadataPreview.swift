import SwiftUI

struct MetadataPreview: View {
    let image: ImageDetail

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("メタデータ")
                    .font(.headline)
                Spacer()
                Image(systemName: "chevron.right")
                    .foregroundStyle(.secondary)
            }

            if let modelName = image.modelName {
                MetadataRow(label: "モデル", value: modelName)
            }

            if let sampler = image.samplerName {
                MetadataRow(label: "サンプラー", value: sampler)
            }

            if let steps = image.steps {
                MetadataRow(label: "ステップ", value: "\(steps)")
            }

            if let cfgScale = image.cfgScale {
                MetadataRow(label: "CFG Scale", value: String(format: "%.1f", cfgScale))
            }

            if let seed = image.seed {
                MetadataRow(label: "シード", value: "\(seed)")
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(8)
    }
}

struct MetadataRow: View {
    let label: String
    let value: String

    var body: some View {
        HStack {
            Text(label)
                .foregroundStyle(.secondary)
            Spacer()
            Text(value)
                .lineLimit(1)
        }
        .font(.caption)
    }
}
