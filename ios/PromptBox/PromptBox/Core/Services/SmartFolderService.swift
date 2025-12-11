import Foundation

@Observable
final class SmartFolderService {
    static let shared = SmartFolderService()

    private let apiClient = APIClient.shared

    private init() {}

    func fetchSmartFolders() async throws -> [SmartFolder] {
        try await apiClient.request(endpoint: .smartFolders)
    }

    func fetchSmartFolder(id: String) async throws -> SmartFolder {
        try await apiClient.request(endpoint: .smartFolder(id: id))
    }

    func createSmartFolder(name: String, icon: String? = nil, filters: SearchFilters) async throws -> SmartFolder {
        let request = CreateSmartFolderRequest(name: name, icon: icon, filters: filters)
        return try await apiClient.request(endpoint: .createSmartFolder, body: request)
    }

    func updateSmartFolder(id: String, name: String? = nil, icon: String? = nil, filters: SearchFilters? = nil) async throws -> SmartFolder {
        let request = UpdateSmartFolderRequest(name: name, icon: icon, filters: filters)
        return try await apiClient.request(endpoint: .updateSmartFolder(id: id), body: request)
    }

    func deleteSmartFolder(id: String) async throws {
        try await apiClient.requestVoid(endpoint: .deleteSmartFolder(id: id))
    }

    func fetchSmartFolderImages(id: String, page: Int = 1, perPage: Int = 24) async throws -> ImageListResponse {
        try await apiClient.request(
            endpoint: .smartFolderImages(id: id),
            queryItems: [
                URLQueryItem(name: "page", value: String(page)),
                URLQueryItem(name: "per_page", value: String(perPage))
            ]
        )
    }
}
