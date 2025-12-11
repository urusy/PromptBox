import Foundation

@Observable
final class BulkService {
    static let shared = BulkService()

    private let apiClient = APIClient.shared

    private init() {}

    func bulkUpdate(
        ids: [String],
        rating: Int? = nil,
        isFavorite: Bool? = nil,
        needsImprovement: Bool? = nil,
        addTags: [String]? = nil,
        removeTags: [String]? = nil
    ) async throws -> MessageResponse {
        let request = BulkUpdateRequest(
            ids: ids,
            rating: rating,
            isFavorite: isFavorite,
            needsImprovement: needsImprovement,
            addTags: addTags,
            removeTags: removeTags
        )
        return try await apiClient.request(endpoint: .bulkUpdate, body: request)
    }

    func bulkRate(ids: [String], rating: Int) async throws -> MessageResponse {
        try await bulkUpdate(ids: ids, rating: rating)
    }

    func bulkFavorite(ids: [String], isFavorite: Bool) async throws -> MessageResponse {
        try await bulkUpdate(ids: ids, isFavorite: isFavorite)
    }

    func bulkTag(ids: [String], addTags: [String]?, removeTags: [String]?) async throws -> MessageResponse {
        try await bulkUpdate(ids: ids, addTags: addTags, removeTags: removeTags)
    }

    func bulkDelete(ids: [String], permanent: Bool = false) async throws -> MessageResponse {
        let request = BulkDeleteRequest(ids: ids, permanent: permanent)
        return try await apiClient.request(endpoint: .bulkDelete, body: request)
    }

    func bulkRestore(ids: [String]) async throws -> MessageResponse {
        let request = BulkRestoreRequest(ids: ids)
        return try await apiClient.request(endpoint: .bulkRestore, body: request)
    }
}
