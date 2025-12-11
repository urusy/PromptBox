import Foundation

@Observable
final class ImageService {
    static let shared = ImageService()

    private let apiClient = APIClient.shared

    private init() {}

    func fetchImages(params: SearchParams) async throws -> ImageListResponse {
        try await apiClient.request(
            endpoint: .images,
            queryItems: params.queryItems
        )
    }

    func fetchImage(id: String) async throws -> ImageDetail {
        try await apiClient.request(endpoint: .image(id: id))
    }

    func updateImage(id: String, rating: Int? = nil, isFavorite: Bool? = nil, memo: String? = nil) async throws -> ImageDetail {
        return try await apiClient.request(
            endpoint: .image(id: id),
            body: ImageUpdateRequest(rating: rating, isFavorite: isFavorite, userMemo: memo)
        )
    }

    func deleteImage(id: String) async throws {
        try await apiClient.requestVoid(endpoint: .image(id: id))
    }

    func addTag(imageId: String, tagName: String) async throws -> ImageDetail {
        try await apiClient.request(
            endpoint: .image(id: imageId),
            body: TagUpdateRequest(userTags: [tagName])
        )
    }

    func removeTag(imageId: String, tagName: String) async throws -> ImageDetail {
        // For removing, we need to get current tags and remove the specified one
        let current = try await fetchImage(id: imageId)
        let newTags = current.userTags.filter { $0 != tagName }
        return try await apiClient.request(
            endpoint: .image(id: imageId),
            body: TagUpdateRequest(userTags: newTags)
        )
    }
}

private struct ImageUpdateRequest: Encodable {
    let rating: Int?
    let isFavorite: Bool?
    let userMemo: String?

    enum CodingKeys: String, CodingKey {
        case rating
        case isFavorite = "is_favorite"
        case userMemo = "user_memo"
    }
}

private struct TagUpdateRequest: Encodable {
    let userTags: [String]

    enum CodingKeys: String, CodingKey {
        case userTags = "user_tags"
    }
}
