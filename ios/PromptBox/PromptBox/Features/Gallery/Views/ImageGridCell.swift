import SwiftUI
import Kingfisher

struct ImageGridCell: View {
    let image: ImageItem
    let isSelected: Bool
    let isSelectionMode: Bool

    var body: some View {
        ZStack(alignment: .topTrailing) {
            KFImage(image.thumbnailURL ?? image.imageURL)
                .placeholder {
                    Rectangle()
                        .fill(Color.gray.opacity(0.2))
                        .overlay {
                            ProgressView()
                        }
                }
                .resizable()
                .aspectRatio(1, contentMode: .fill)
                .clipped()
                .cornerRadius(8)

            // Selection indicator
            if isSelectionMode {
                Circle()
                    .fill(isSelected ? Color.accentColor : Color.white.opacity(0.8))
                    .frame(width: 24, height: 24)
                    .overlay {
                        if isSelected {
                            Image(systemName: "checkmark")
                                .font(.caption.bold())
                                .foregroundStyle(.white)
                        }
                    }
                    .padding(6)
            }

            // Indicators
            VStack(alignment: .trailing, spacing: 4) {
                Spacer()

                HStack(spacing: 4) {
                    if image.isFavorite {
                        Image(systemName: "heart.fill")
                            .font(.caption)
                            .foregroundStyle(.red)
                    }

                    if image.rating > 0 {
                        HStack(spacing: 1) {
                            Image(systemName: "star.fill")
                                .font(.caption2)
                            Text("\(image.rating)")
                                .font(.caption2)
                        }
                        .foregroundStyle(.yellow)
                    }
                }
                .padding(4)
                .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 4))
            }
            .padding(6)
        }
        .overlay {
            if isSelected {
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color.accentColor, lineWidth: 3)
            }
        }
    }
}

#Preview {
    ImageGridCell(
        image: ImageItem(
            id: "1",
            sourceTool: "comfyui",
            modelType: "sdxl",
            storagePath: "test/test.png",
            thumbnailPath: "test/test_thumb.webp",
            width: 512,
            height: 512,
            modelName: "sd_xl_base",
            rating: 5,
            isFavorite: true,
            createdAt: Date()
        ),
        isSelected: false,
        isSelectionMode: false
    )
    .frame(width: 150, height: 150)
}
