import Foundation

enum AppConfig {
    static var apiBaseURL: String {
        #if DEBUG
        return UserDefaults.standard.string(forKey: "apiBaseURL") ?? "http://localhost:8000/api"
        #else
        return UserDefaults.standard.string(forKey: "apiBaseURL") ?? "https://promptbox.example.com/api"
        #endif
    }

    static let defaultPerPage = 24
    static let maxBulkSize = 500
}
