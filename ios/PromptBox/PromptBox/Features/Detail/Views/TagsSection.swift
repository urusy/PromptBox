import SwiftUI

struct TagsSection: View {
    let tags: [String]
    @Binding var newTag: String
    let onAddTag: (String) -> Void
    let onRemoveTag: (String) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("タグ")
                .font(.headline)

            FlowLayout(spacing: 8) {
                ForEach(tags, id: \.self) { tag in
                    TagChip(name: tag) {
                        onRemoveTag(tag)
                    }
                }
            }

            HStack {
                TextField("新しいタグ", text: $newTag)
                    .textFieldStyle(.roundedBorder)
                    .autocorrectionDisabled()
                    .onSubmit {
                        if !newTag.isEmpty {
                            onAddTag(newTag)
                        }
                    }

                Button {
                    if !newTag.isEmpty {
                        onAddTag(newTag)
                    }
                } label: {
                    Image(systemName: "plus.circle.fill")
                }
                .disabled(newTag.isEmpty)
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(8)
    }
}

struct TagChip: View {
    let name: String
    let onRemove: () -> Void

    var body: some View {
        HStack(spacing: 4) {
            Text(name)
                .font(.caption)

            Button {
                onRemove()
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .font(.caption)
            }
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(Color.accentColor.opacity(0.2))
        .foregroundStyle(Color.accentColor)
        .cornerRadius(16)
    }
}

struct FlowLayout: Layout {
    var spacing: CGFloat = 8

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let result = FlowResult(in: proposal.width ?? 0, subviews: subviews, spacing: spacing)
        return result.size
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let result = FlowResult(in: bounds.width, subviews: subviews, spacing: spacing)
        for (index, subview) in subviews.enumerated() {
            subview.place(at: CGPoint(x: bounds.minX + result.positions[index].x,
                                      y: bounds.minY + result.positions[index].y),
                         proposal: .unspecified)
        }
    }

    struct FlowResult {
        var size: CGSize = .zero
        var positions: [CGPoint] = []

        init(in maxWidth: CGFloat, subviews: Subviews, spacing: CGFloat) {
            var x: CGFloat = 0
            var y: CGFloat = 0
            var rowHeight: CGFloat = 0

            for subview in subviews {
                let size = subview.sizeThatFits(.unspecified)
                if x + size.width > maxWidth && x > 0 {
                    x = 0
                    y += rowHeight + spacing
                    rowHeight = 0
                }
                positions.append(CGPoint(x: x, y: y))
                rowHeight = max(rowHeight, size.height)
                x += size.width + spacing
                self.size.width = max(self.size.width, x)
            }
            self.size.height = y + rowHeight
        }
    }
}

#Preview {
    TagsSection(
        tags: ["landscape", "portrait"],
        newTag: .constant(""),
        onAddTag: { _ in },
        onRemoveTag: { _ in }
    )
    .padding()
}
