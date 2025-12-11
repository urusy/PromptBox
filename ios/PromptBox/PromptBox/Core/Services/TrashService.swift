import Foundation

@Observable
final class TrashService {
    static let shared = TrashService()

    private let apiClient = APIClient.shared
    private let bulkService = BulkService.shared

    private init() {}

    func fetchTrash(page: Int = 1, perPage: Int = 24) async throws -> ImageListResponse {
        // Use images endpoint with include_deleted=true to get deleted images
        try await apiClient.request(
            endpoint: .images,
            queryItems: [
                URLQueryItem(name: "include_deleted", value: "true"),
                URLQueryItem(name: "page", value: String(page)),
                URLQueryItem(name: "per_page", value: String(perPage))
            ]
        )
    }

    func restoreImage(id: String) async throws {
        try await apiClient.requestVoid(endpoint: .trashRestore(id: id))
    }

    func deleteImagePermanently(id: String) async throws {
        // Use bulk delete with permanent=true for single image
        _ = try await bulkService.bulkDelete(ids: [id], permanent: true)
    }

    func emptyTrash(ids: [String]) async throws {
        // Use bulk delete with permanent=true
        _ = try await bulkService.bulkDelete(ids: ids, permanent: true)
    }

    func restoreAll(ids: [String]) async throws {
        _ = try await bulkService.bulkRestore(ids: ids)
    }
}
