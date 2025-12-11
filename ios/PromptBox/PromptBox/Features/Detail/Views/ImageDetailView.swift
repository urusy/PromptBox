import SwiftUI
import Kingfisher

struct ImageDetailView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var viewModel: ImageDetailViewModel
    @State private var showingDeleteConfirm = false
    @State private var showingMetadata = false

    let onImageDeleted: () -> Void

    init(imageId: String, images: [ImageItem] = [], onImageDeleted: @escaping () -> Void = {}) {
        self._viewModel = State(initialValue: ImageDetailViewModel(imageId: imageId, images: images))
        self.onImageDeleted = onImageDeleted
    }

    var body: some View {
        Group {
            if viewModel.isLoading && viewModel.image == nil {
                ProgressView()
            } else if let image = viewModel.image {
                imageContent(image)
            } else if let error = viewModel.errorMessage {
                ContentUnavailableView {
                    Label("エラー", systemImage: "exclamationmark.triangle")
                } description: {
                    Text(error)
                }
            }
        }
        .navigationTitle(viewModel.image?.originalFilename ?? "")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            toolbarContent
        }
        .task {
            await viewModel.loadImage()
        }
        .confirmationDialog(
            "この画像を削除しますか？",
            isPresented: $showingDeleteConfirm,
            titleVisibility: .visible
        ) {
            Button("削除", role: .destructive) {
                Task {
                    try await viewModel.deleteImage()
                    onImageDeleted()
                    dismiss()
                }
            }
        } message: {
            Text("削除した画像はゴミ箱に移動します")
        }
        .sheet(isPresented: $showingMetadata) {
            if let image = viewModel.image {
                MetadataSheet(image: image)
            }
        }
        .gesture(
            DragGesture(minimumDistance: 50)
                .onEnded { value in
                    handleSwipe(value)
                }
        )
    }

    @ViewBuilder
    private func imageContent(_ image: ImageDetail) -> some View {
        ScrollView {
            VStack(spacing: 16) {
                // Image
                KFImage(image.imageURL)
                    .placeholder {
                        ProgressView()
                    }
                    .resizable()
                    .aspectRatio(contentMode: .fit)
                    .cornerRadius(8)

                // Quick actions
                HStack(spacing: 24) {
                    // Rating
                    HStack(spacing: 4) {
                        ForEach(1...5, id: \.self) { rating in
                            Button {
                                Task { await viewModel.updateRating(rating) }
                            } label: {
                                Image(systemName: image.rating >= rating ? "star.fill" : "star")
                                    .foregroundStyle(.yellow)
                            }
                        }
                    }

                    // Favorite
                    Button {
                        Task { await viewModel.toggleFavorite() }
                    } label: {
                        Image(systemName: image.isFavorite ? "heart.fill" : "heart")
                            .foregroundStyle(image.isFavorite ? .red : .gray)
                    }

                    Spacer()

                    // Delete
                    Button(role: .destructive) {
                        showingDeleteConfirm = true
                    } label: {
                        Image(systemName: "trash")
                    }
                }
                .font(.title2)
                .padding(.horizontal)

                // Tags
                TagsSection(
                    tags: image.userTags,
                    newTag: $viewModel.newTag,
                    onAddTag: { tag in
                        Task { await viewModel.addTag(tag) }
                    },
                    onRemoveTag: { tag in
                        Task { await viewModel.removeTag(tag) }
                    }
                )

                // Memo
                MemoSection(
                    memo: image.userMemo ?? "",
                    onSave: { memo in
                        Task { await viewModel.updateMemo(memo) }
                    }
                )

                // Metadata preview
                if image.hasMetadata {
                    MetadataPreview(image: image)
                        .onTapGesture {
                            showingMetadata = true
                        }
                }
            }
            .padding()
        }
    }

    @ToolbarContentBuilder
    private var toolbarContent: some ToolbarContent {
        ToolbarItem(placement: .topBarTrailing) {
            HStack {
                Button {
                    navigatePrevious()
                } label: {
                    Image(systemName: "chevron.left")
                }
                .disabled(!viewModel.hasPrevious)

                Button {
                    navigateNext()
                } label: {
                    Image(systemName: "chevron.right")
                }
                .disabled(!viewModel.hasNext)
            }
        }
    }

    private func handleSwipe(_ value: DragGesture.Value) {
        let horizontalAmount = value.translation.width
        if horizontalAmount > 50 {
            navigatePrevious()
        } else if horizontalAmount < -50 {
            navigateNext()
        }
    }

    private func navigatePrevious() {
        // Navigation is handled by parent view
    }

    private func navigateNext() {
        // Navigation is handled by parent view
    }
}

#Preview {
    NavigationStack {
        ImageDetailView(imageId: "test")
    }
}
