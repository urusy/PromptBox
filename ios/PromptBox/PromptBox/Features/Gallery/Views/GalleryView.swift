import SwiftUI

struct GalleryView: View {
    @State private var viewModel = GalleryViewModel()
    @State private var selectedImage: ImageItem?
    @State private var showingFilter = false
    @State private var showingBulkActions = false

    private let columns = [
        GridItem(.adaptive(minimum: 150, maximum: 200), spacing: 8)
    ]

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading && viewModel.images.isEmpty {
                    ProgressView("読み込み中...")
                } else if let error = viewModel.errorMessage, viewModel.images.isEmpty {
                    ContentUnavailableView {
                        Label("エラー", systemImage: "exclamationmark.triangle")
                    } description: {
                        Text(error)
                    } actions: {
                        Button("再試行") {
                            Task { await viewModel.refresh() }
                        }
                    }
                } else if viewModel.images.isEmpty {
                    ContentUnavailableView {
                        Label("画像がありません", systemImage: "photo.on.rectangle.angled")
                    } description: {
                        Text("検索条件を変更してください")
                    }
                } else {
                    imageGrid
                }
            }
            .navigationTitle("ギャラリー")
            .toolbar {
                toolbarContent
            }
            .refreshable {
                await viewModel.refresh()
            }
            .sheet(isPresented: $showingFilter) {
                FilterView(params: $viewModel.searchParams) {
                    Task { await viewModel.refresh() }
                }
            }
            .sheet(isPresented: $showingBulkActions) {
                BulkActionsSheet(viewModel: viewModel)
            }
            .navigationDestination(item: $selectedImage) { image in
                ImageDetailView(
                    imageId: image.id,
                    images: viewModel.images,
                    onImageDeleted: {
                        Task { await viewModel.refresh() }
                    }
                )
            }
        }
        .task {
            if viewModel.images.isEmpty {
                await viewModel.loadImages(refresh: true)
            }
        }
    }

    @ViewBuilder
    private var imageGrid: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 8) {
                ForEach(viewModel.images) { image in
                    ImageGridCell(
                        image: image,
                        isSelected: viewModel.selectedImageIds.contains(image.id),
                        isSelectionMode: viewModel.isSelectionMode
                    )
                    .onTapGesture {
                        if viewModel.isSelectionMode {
                            viewModel.toggleSelection(for: image.id)
                        } else {
                            selectedImage = image
                        }
                    }
                    .onLongPressGesture {
                        viewModel.isSelectionMode = true
                        viewModel.toggleSelection(for: image.id)
                    }
                    .onAppear {
                        Task {
                            await viewModel.loadMoreIfNeeded(currentItem: image)
                        }
                    }
                }
            }
            .padding()

            if viewModel.isLoadingMore {
                ProgressView()
                    .padding()
            }
        }
    }

    @ToolbarContentBuilder
    private var toolbarContent: some ToolbarContent {
        ToolbarItem(placement: .topBarLeading) {
            if viewModel.isSelectionMode {
                Button("キャンセル") {
                    viewModel.clearSelection()
                }
            }
        }

        ToolbarItem(placement: .topBarTrailing) {
            if viewModel.isSelectionMode {
                Button("操作") {
                    showingBulkActions = true
                }
                .disabled(viewModel.selectedImageIds.isEmpty)
            } else {
                Menu {
                    Button {
                        showingFilter = true
                    } label: {
                        Label("フィルター", systemImage: "line.3.horizontal.decrease.circle")
                    }

                    Button {
                        viewModel.isSelectionMode = true
                    } label: {
                        Label("選択", systemImage: "checkmark.circle")
                    }
                } label: {
                    Image(systemName: "ellipsis.circle")
                }
            }
        }

        if viewModel.isSelectionMode {
            ToolbarItem(placement: .bottomBar) {
                Text("\(viewModel.selectedCount)件選択中")
            }
        }
    }
}

#Preview {
    GalleryView()
}
