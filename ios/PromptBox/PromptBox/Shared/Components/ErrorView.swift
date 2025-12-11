import SwiftUI

struct ErrorView: View {
    let message: String
    let retryAction: (() -> Void)?

    init(_ message: String, retryAction: (() -> Void)? = nil) {
        self.message = message
        self.retryAction = retryAction
    }

    var body: some View {
        ContentUnavailableView {
            Label("エラー", systemImage: "exclamationmark.triangle")
        } description: {
            Text(message)
        } actions: {
            if let retryAction {
                Button("再試行", action: retryAction)
                    .buttonStyle(.borderedProminent)
            }
        }
    }
}

#Preview {
    ErrorView("ネットワークエラーが発生しました") {
        print("Retry tapped")
    }
}
