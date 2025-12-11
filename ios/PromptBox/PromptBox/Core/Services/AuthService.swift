import Foundation
import KeychainAccess

@Observable
final class AuthService {
    static let shared = AuthService()

    private(set) var currentUser: User?
    private(set) var isAuthenticated = false
    private(set) var isLoading = false

    private let keychain = Keychain(service: "com.promptbox.ios")
    private let apiClient = APIClient.shared

    private init() {}

    @MainActor
    func checkAuth() async {
        isLoading = true
        defer { isLoading = false }

        do {
            let response: MeResponse = try await apiClient.request(endpoint: .me)
            currentUser = User(id: response.username, username: response.username, createdAt: nil)
            isAuthenticated = true
        } catch {
            currentUser = nil
            isAuthenticated = false
        }
    }

    @MainActor
    func login(username: String, password: String) async throws {
        isLoading = true
        defer { isLoading = false }

        let request = LoginRequest(username: username, password: password)
        let response: LoginResponse = try await apiClient.request(
            endpoint: .login,
            body: request
        )

        currentUser = User(id: response.username, username: response.username, createdAt: nil)
        isAuthenticated = true

        // Save credentials to keychain for future auto-login attempts
        try? keychain.set(username, key: "username")
    }

    @MainActor
    func logout() async {
        isLoading = true
        defer { isLoading = false }

        do {
            try await apiClient.requestVoid(endpoint: .logout)
        } catch {
            // Ignore logout errors
        }

        currentUser = nil
        isAuthenticated = false

        // Clear keychain
        try? keychain.remove("username")
    }

    func getSavedUsername() -> String? {
        try? keychain.get("username")
    }
}
