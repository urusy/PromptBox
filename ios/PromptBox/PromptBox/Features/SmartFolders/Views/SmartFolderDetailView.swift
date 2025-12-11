import SwiftUI

struct SmartFolderDetailView: View {
    let folder: SmartFolder

    @State private var images: [ImageItem] = []
    @State private var isLoading = false
    @State private var isLoadingMore = false
    @State private var errorMessage: String?
    @State private var currentPage = 1
    @State private var totalPages = 1
    @State private var selectedImage: ImageItem?

    private let service = SmartFolderService.shared

    private let columns = [
        GridItem(.adaptive(minimum: 150, maximum: 200), spacing: 8)
    ]

    var body: some View {
        Group {
            if isLoading && images.isEmpty {
                ProgressView("読み込み中...")
            } else if let error = errorMessage, images.isEmpty {
                ContentUnavailableView {
                    Label("エラー", systemImage: "exclamationmark.triangle")
                } description: {
                    Text(error)
                } actions: {
                    Button("再試行") {
                        Task { await loadImages(refresh: true) }
                    }
                }
            } else if images.isEmpty {
                ContentUnavailableView {
                    Label("画像がありません", systemImage: "photo.on.rectangle.angled")
                } description: {
                    Text("条件に一致する画像がありません")
                }
            } else {
                imageGrid
            }
        }
        .navigationTitle(folder.name)
        .refreshable {
            await loadImages(refresh: true)
        }
        .navigationDestination(item: $selectedImage) { image in
            ImageDetailView(
                imageId: image.id,
                images: images,
                onImageDeleted: {
                    Task { await loadImages(refresh: true) }
                }
            )
        }
        .task {
            await loadImages(refresh: true)
        }
    }

    @ViewBuilder
    private var imageGrid: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 8) {
                ForEach(images) { image in
                    ImageGridCell(
                        image: image,
                        isSelected: false,
                        isSelectionMode: false
                    )
                    .onTapGesture {
                        selectedImage = image
                    }
                    .onAppear {
                        Task {
                            await loadMoreIfNeeded(currentItem: image)
                        }
                    }
                }
            }
            .padding()

            if isLoadingMore {
                ProgressView()
                    .padding()
            }
        }
    }

    @MainActor
    private func loadImages(refresh: Bool = false) async {
        if refresh {
            currentPage = 1
            isLoading = true
        }

        errorMessage = nil

        do {
            let response = try await service.fetchSmartFolderImages(
                id: folder.id,
                page: currentPage
            )
            if refresh {
                images = response.items
            } else {
                images.append(contentsOf: response.items)
            }
            totalPages = response.totalPages
        } catch {
            errorMessage = error.localizedDescription
        }

        isLoading = false
        isLoadingMore = false
    }

    @MainActor
    private func loadMoreIfNeeded(currentItem: ImageItem) async {
        guard !isLoadingMore && currentPage < totalPages else { return }

        let thresholdIndex = images.index(images.endIndex, offsetBy: -5)
        if let currentIndex = images.firstIndex(where: { $0.id == currentItem.id }),
           currentIndex >= thresholdIndex {
            isLoadingMore = true
            currentPage += 1
            await loadImages()
        }
    }
}

#Preview {
    NavigationStack {
        SmartFolderDetailView(
            folder: SmartFolder(
                id: "1",
                name: "お気に入り",
                icon: nil,
                filters: SearchFilters(isFavorite: true),
                createdAt: Date(),
                updatedAt: Date()
            )
        )
    }
}
