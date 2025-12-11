import SwiftUI

struct ContentView: View {
    @State private var authService = AuthService.shared

    var body: some View {
        Group {
            if authService.isLoading {
                ProgressView("認証を確認中...")
            } else if authService.isAuthenticated {
                MainTabView()
            } else {
                LoginView()
            }
        }
        .task {
            await authService.checkAuth()
        }
    }
}

#Preview {
    ContentView()
}
