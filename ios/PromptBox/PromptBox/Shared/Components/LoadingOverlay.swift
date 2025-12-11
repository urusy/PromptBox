import SwiftUI

struct LoadingOverlay: View {
    let message: String?

    init(_ message: String? = nil) {
        self.message = message
    }

    var body: some View {
        ZStack {
            Color.black.opacity(0.3)
                .ignoresSafeArea()

            VStack(spacing: 16) {
                ProgressView()
                    .scaleEffect(1.5)

                if let message {
                    Text(message)
                        .font(.subheadline)
                }
            }
            .padding(24)
            .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 16))
        }
    }
}

extension View {
    func loadingOverlay(isLoading: Bool, message: String? = nil) -> some View {
        self.overlay {
            if isLoading {
                LoadingOverlay(message)
            }
        }
    }
}

#Preview {
    Color.blue
        .loadingOverlay(isLoading: true, message: "読み込み中...")
}
