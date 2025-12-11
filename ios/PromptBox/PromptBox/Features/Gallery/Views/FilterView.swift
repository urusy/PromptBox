import SwiftUI

struct FilterView: View {
    @Environment(\.dismiss) private var dismiss
    @Binding var params: SearchParams
    let onApply: () -> Void

    @State private var localParams: SearchParams

    init(params: Binding<SearchParams>, onApply: @escaping () -> Void) {
        self._params = params
        self.onApply = onApply
        self._localParams = State(initialValue: params.wrappedValue)
    }

    var body: some View {
        NavigationStack {
            Form {
                Section("検索") {
                    TextField("キーワード", text: $localParams.query)
                        .autocorrectionDisabled()
                }

                Section("評価") {
                    Picker("評価", selection: $localParams.rating) {
                        Text("すべて").tag(nil as Int?)
                        ForEach(1...5, id: \.self) { rating in
                            HStack {
                                ForEach(1...rating, id: \.self) { _ in
                                    Image(systemName: "star.fill")
                                        .foregroundStyle(.yellow)
                                }
                            }
                            .tag(rating as Int?)
                        }
                    }
                }

                Section("お気に入り") {
                    Picker("お気に入り", selection: $localParams.isFavorite) {
                        Text("すべて").tag(nil as Bool?)
                        Text("お気に入りのみ").tag(true as Bool?)
                        Text("お気に入り以外").tag(false as Bool?)
                    }
                }

                Section("モデル") {
                    TextField("モデル名", text: Binding(
                        get: { localParams.modelName ?? "" },
                        set: { localParams.modelName = $0.isEmpty ? nil : $0 }
                    ))
                    .autocorrectionDisabled()
                }

                Section("並び順") {
                    Picker("並び順", selection: $localParams.sortBy) {
                        Text("作成日").tag(SearchParams.SortBy.createdAt)
                        Text("評価").tag(SearchParams.SortBy.rating)
                        Text("ファイル名").tag(SearchParams.SortBy.filename)
                        Text("ファイルサイズ").tag(SearchParams.SortBy.fileSize)
                    }

                    Picker("順序", selection: $localParams.sortOrder) {
                        Text("降順").tag(SearchParams.SortOrder.desc)
                        Text("昇順").tag(SearchParams.SortOrder.asc)
                    }
                }

                Section {
                    Button("リセット", role: .destructive) {
                        localParams.reset()
                    }
                }
            }
            .navigationTitle("フィルター")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("キャンセル") {
                        dismiss()
                    }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("適用") {
                        params = localParams
                        onApply()
                        dismiss()
                    }
                }
            }
        }
    }
}

#Preview {
    FilterView(params: .constant(SearchParams())) {}
}
