import SwiftUI

struct MemoSection: View {
    let memo: String
    let onSave: (String) -> Void

    @State private var editedMemo: String
    @State private var isEditing = false
    @FocusState private var isFocused: Bool

    init(memo: String, onSave: @escaping (String) -> Void) {
        self.memo = memo
        self.onSave = onSave
        self._editedMemo = State(initialValue: memo)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("メモ")
                    .font(.headline)

                Spacer()

                if isEditing {
                    Button("保存") {
                        onSave(editedMemo)
                        isEditing = false
                        isFocused = false
                    }
                }
            }

            if isEditing {
                TextEditor(text: $editedMemo)
                    .frame(minHeight: 80)
                    .focused($isFocused)
                    .onAppear { isFocused = true }
            } else {
                Text(memo.isEmpty ? "タップしてメモを追加" : memo)
                    .foregroundStyle(memo.isEmpty ? .secondary : .primary)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        isEditing = true
                    }
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(8)
        .onChange(of: memo) { _, newValue in
            editedMemo = newValue
        }
    }
}

#Preview {
    VStack {
        MemoSection(memo: "", onSave: { _ in })
        MemoSection(memo: "This is a test memo", onSave: { _ in })
    }
    .padding()
}
