import SwiftUI

struct SmartFoldersView: View {
    @State private var viewModel = SmartFoldersViewModel()
    @State private var selectedFolder: SmartFolder?

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading && viewModel.folders.isEmpty {
                    ProgressView("読み込み中...")
                } else if let error = viewModel.errorMessage, viewModel.folders.isEmpty {
                    ContentUnavailableView {
                        Label("エラー", systemImage: "exclamationmark.triangle")
                    } description: {
                        Text(error)
                    } actions: {
                        Button("再試行") {
                            Task { await viewModel.loadFolders() }
                        }
                    }
                } else if viewModel.folders.isEmpty {
                    ContentUnavailableView {
                        Label("スマートフォルダがありません", systemImage: "folder.badge.gearshape")
                    } description: {
                        Text("Webアプリからスマートフォルダを作成できます")
                    }
                } else {
                    folderList
                }
            }
            .navigationTitle("スマートフォルダ")
            .refreshable {
                await viewModel.loadFolders()
            }
            .navigationDestination(item: $selectedFolder) { folder in
                SmartFolderDetailView(folder: folder)
            }
        }
        .task {
            await viewModel.loadFolders()
        }
    }

    @ViewBuilder
    private var folderList: some View {
        List {
            ForEach(viewModel.folders) { folder in
                Button {
                    selectedFolder = folder
                } label: {
                    HStack {
                        Image(systemName: folder.icon ?? "folder.badge.gearshape")
                            .foregroundStyle(Color.accentColor)

                        Text(folder.name)

                        Spacer()

                        Image(systemName: "chevron.right")
                            .foregroundStyle(.secondary)
                    }
                }
                .foregroundStyle(.primary)
            }
            .onDelete { indexSet in
                for index in indexSet {
                    let folder = viewModel.folders[index]
                    Task {
                        await viewModel.deleteFolder(id: folder.id)
                    }
                }
            }
        }
    }
}

#Preview {
    SmartFoldersView()
}
