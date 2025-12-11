import SwiftUI

struct LoginView: View {
    @State private var username = ""
    @State private var password = ""
    @State private var isLoading = false
    @State private var errorMessage: String?

    private let authService = AuthService.shared

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("ユーザー名", text: $username)
                        .textContentType(.username)
                        .autocapitalization(.none)
                        .autocorrectionDisabled()

                    SecureField("パスワード", text: $password)
                        .textContentType(.password)
                }

                if let errorMessage {
                    Section {
                        Text(errorMessage)
                            .foregroundStyle(.red)
                    }
                }

                Section {
                    Button(action: login) {
                        if isLoading {
                            ProgressView()
                                .frame(maxWidth: .infinity)
                        } else {
                            Text("ログイン")
                                .frame(maxWidth: .infinity)
                        }
                    }
                    .disabled(username.isEmpty || password.isEmpty || isLoading)
                }
            }
            .navigationTitle("PromptBox")
            .onAppear {
                if let savedUsername = authService.getSavedUsername() {
                    username = savedUsername
                }
            }
        }
    }

    private func login() {
        isLoading = true
        errorMessage = nil

        Task {
            do {
                try await authService.login(username: username, password: password)
            } catch let error as APIError {
                errorMessage = error.errorDescription
            } catch {
                errorMessage = error.localizedDescription
            }
            isLoading = false
        }
    }
}

#Preview {
    LoginView()
}
