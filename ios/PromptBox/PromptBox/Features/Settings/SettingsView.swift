import SwiftUI

struct SettingsView: View {
    @State private var apiBaseURL: String = ""
    @State private var showingLogoutConfirm = false

    private let authService = AuthService.shared

    var body: some View {
        NavigationStack {
            Form {
                Section("アカウント") {
                    if let user = authService.currentUser {
                        LabeledContent("ユーザー名", value: user.username)
                    }

                    Button("ログアウト", role: .destructive) {
                        showingLogoutConfirm = true
                    }
                }

                Section("接続設定") {
                    TextField("API URL", text: $apiBaseURL)
                        .textContentType(.URL)
                        .autocapitalization(.none)
                        .autocorrectionDisabled()
                        .onSubmit {
                            saveAPIBaseURL()
                        }

                    Button("保存") {
                        saveAPIBaseURL()
                    }

                    Button("デフォルトに戻す") {
                        resetAPIBaseURL()
                    }
                }

                Section("アプリ情報") {
                    LabeledContent("バージョン", value: appVersion)
                    LabeledContent("ビルド", value: buildNumber)
                }
            }
            .navigationTitle("設定")
            .onAppear {
                loadAPIBaseURL()
            }
            .confirmationDialog(
                "ログアウトしますか？",
                isPresented: $showingLogoutConfirm,
                titleVisibility: .visible
            ) {
                Button("ログアウト", role: .destructive) {
                    Task {
                        await authService.logout()
                    }
                }
            }
        }
    }

    private var appVersion: String {
        Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0"
    }

    private var buildNumber: String {
        Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "1"
    }

    private func loadAPIBaseURL() {
        apiBaseURL = UserDefaults.standard.string(forKey: "apiBaseURL") ?? ""
    }

    private func saveAPIBaseURL() {
        if apiBaseURL.isEmpty {
            UserDefaults.standard.removeObject(forKey: "apiBaseURL")
        } else {
            UserDefaults.standard.set(apiBaseURL, forKey: "apiBaseURL")
        }
    }

    private func resetAPIBaseURL() {
        apiBaseURL = ""
        UserDefaults.standard.removeObject(forKey: "apiBaseURL")
    }
}

#Preview {
    SettingsView()
}
