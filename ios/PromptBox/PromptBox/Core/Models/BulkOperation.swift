import Foundation

// Matches backend BatchUpdateRequest
struct BulkUpdateRequest: Encodable {
    let ids: [String]
    let rating: Int?
    let isFavorite: Bool?
    let needsImprovement: Bool?
    let addTags: [String]?
    let removeTags: [String]?

    enum CodingKeys: String, CodingKey {
        case ids
        case rating
        case isFavorite = "is_favorite"
        case needsImprovement = "needs_improvement"
        case addTags = "add_tags"
        case removeTags = "remove_tags"
    }

    init(
        ids: [String],
        rating: Int? = nil,
        isFavorite: Bool? = nil,
        needsImprovement: Bool? = nil,
        addTags: [String]? = nil,
        removeTags: [String]? = nil
    ) {
        self.ids = ids
        self.rating = rating
        self.isFavorite = isFavorite
        self.needsImprovement = needsImprovement
        self.addTags = addTags
        self.removeTags = removeTags
    }
}

// Matches backend BatchDeleteRequest
struct BulkDeleteRequest: Encodable {
    let ids: [String]
    let permanent: Bool

    init(ids: [String], permanent: Bool = false) {
        self.ids = ids
        self.permanent = permanent
    }
}

// Matches backend BatchRestoreRequest
struct BulkRestoreRequest: Encodable {
    let ids: [String]
}

// Matches backend MessageResponse
struct MessageResponse: Decodable {
    let message: String
}
