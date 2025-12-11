import SwiftUI

struct BulkActionsSheet: View {
    @Environment(\.dismiss) private var dismiss
    @Bindable var viewModel: GalleryViewModel

    @State private var isProcessing = false
    @State private var errorMessage: String?
    @State private var showingDeleteConfirm = false

    var body: some View {
        NavigationStack {
            List {
                Section("評価を設定") {
                    ForEach(1...5, id: \.self) { rating in
                        Button {
                            performAction {
                                try await viewModel.bulkRate(rating)
                            }
                        } label: {
                            HStack {
                                ForEach(1...rating, id: \.self) { _ in
                                    Image(systemName: "star.fill")
                                        .foregroundStyle(.yellow)
                                }
                                Spacer()
                            }
                        }
                        .disabled(isProcessing)
                    }

                    Button("評価をクリア") {
                        performAction {
                            try await viewModel.bulkRate(0)
                        }
                    }
                    .disabled(isProcessing)
                }

                Section("お気に入り") {
                    Button {
                        performAction {
                            try await viewModel.bulkFavorite(true)
                        }
                    } label: {
                        Label("お気に入りに追加", systemImage: "heart.fill")
                    }
                    .disabled(isProcessing)

                    Button {
                        performAction {
                            try await viewModel.bulkFavorite(false)
                        }
                    } label: {
                        Label("お気に入りから削除", systemImage: "heart.slash")
                    }
                    .disabled(isProcessing)
                }

                Section {
                    Button(role: .destructive) {
                        showingDeleteConfirm = true
                    } label: {
                        Label("削除", systemImage: "trash")
                    }
                    .disabled(isProcessing)
                }

                if let errorMessage {
                    Section {
                        Text(errorMessage)
                            .foregroundStyle(.red)
                    }
                }
            }
            .navigationTitle("\(viewModel.selectedCount)件を操作")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("閉じる") {
                        dismiss()
                    }
                }
            }
            .overlay {
                if isProcessing {
                    ProgressView()
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .background(.ultraThinMaterial)
                }
            }
            .confirmationDialog(
                "選択した\(viewModel.selectedCount)件の画像を削除しますか？",
                isPresented: $showingDeleteConfirm,
                titleVisibility: .visible
            ) {
                Button("削除", role: .destructive) {
                    performAction {
                        try await viewModel.bulkDelete()
                    }
                }
            } message: {
                Text("削除した画像はゴミ箱に移動します")
            }
        }
    }

    private func performAction(_ action: @escaping () async throws -> Void) {
        isProcessing = true
        errorMessage = nil

        Task {
            do {
                try await action()
                dismiss()
            } catch {
                errorMessage = error.localizedDescription
            }
            isProcessing = false
        }
    }
}
