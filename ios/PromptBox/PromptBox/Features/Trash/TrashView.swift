import SwiftUI

struct TrashView: View {
    @State private var images: [ImageItem] = []
    @State private var isLoading = false
    @State private var isLoadingMore = false
    @State private var errorMessage: String?
    @State private var currentPage = 1
    @State private var totalPages = 1
    @State private var showingEmptyConfirm = false

    private let trashService = TrashService.shared

    private let columns = [
        GridItem(.adaptive(minimum: 150, maximum: 200), spacing: 8)
    ]

    var body: some View {
        NavigationStack {
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
                            Task { await loadTrash(refresh: true) }
                        }
                    }
                } else if images.isEmpty {
                    ContentUnavailableView {
                        Label("ゴミ箱は空です", systemImage: "trash")
                    } description: {
                        Text("削除した画像はここに表示されます")
                    }
                } else {
                    trashContent
                }
            }
            .navigationTitle("ゴミ箱")
            .toolbar {
                if !images.isEmpty {
                    ToolbarItem(placement: .topBarTrailing) {
                        Button("空にする", role: .destructive) {
                            showingEmptyConfirm = true
                        }
                    }
                }
            }
            .refreshable {
                await loadTrash(refresh: true)
            }
            .confirmationDialog(
                "ゴミ箱を空にしますか？",
                isPresented: $showingEmptyConfirm,
                titleVisibility: .visible
            ) {
                Button("空にする", role: .destructive) {
                    Task { await emptyTrash() }
                }
            } message: {
                Text("この操作は取り消せません")
            }
        }
        .task {
            await loadTrash(refresh: true)
        }
    }

    @ViewBuilder
    private var trashContent: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 8) {
                ForEach(images) { image in
                    TrashImageCell(
                        image: image,
                        onRestore: {
                            Task { await restoreImage(image) }
                        },
                        onDelete: {
                            Task { await deleteImage(image) }
                        }
                    )
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
    private func loadTrash(refresh: Bool = false) async {
        if refresh {
            currentPage = 1
            isLoading = true
        }

        errorMessage = nil

        do {
            let response = try await trashService.fetchTrash(page: currentPage)
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
            await loadTrash()
        }
    }

    @MainActor
    private func restoreImage(_ image: ImageItem) async {
        do {
            try await trashService.restoreImage(id: image.id)
            images.removeAll { $0.id == image.id }
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    @MainActor
    private func deleteImage(_ image: ImageItem) async {
        do {
            try await trashService.deleteImagePermanently(id: image.id)
            images.removeAll { $0.id == image.id }
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    @MainActor
    private func emptyTrash() async {
        do {
            let ids = images.map { $0.id }
            try await trashService.emptyTrash(ids: ids)
            images.removeAll()
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}

struct TrashImageCell: View {
    let image: ImageItem
    let onRestore: () -> Void
    let onDelete: () -> Void

    @State private var showingDeleteConfirm = false

    var body: some View {
        VStack(spacing: 8) {
            ImageGridCell(
                image: image,
                isSelected: false,
                isSelectionMode: false
            )

            HStack(spacing: 12) {
                Button {
                    onRestore()
                } label: {
                    Label("復元", systemImage: "arrow.uturn.backward")
                        .font(.caption)
                }

                Button(role: .destructive) {
                    showingDeleteConfirm = true
                } label: {
                    Label("削除", systemImage: "trash")
                        .font(.caption)
                }
            }
        }
        .confirmationDialog(
            "完全に削除しますか？",
            isPresented: $showingDeleteConfirm,
            titleVisibility: .visible
        ) {
            Button("削除", role: .destructive) {
                onDelete()
            }
        } message: {
            Text("この操作は取り消せません")
        }
    }
}

#Preview {
    TrashView()
}
