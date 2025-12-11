import Foundation

struct SearchParams: Equatable {
    var query: String = ""
    var rating: Int? = nil
    var isFavorite: Bool? = nil
    var tags: [String] = []
    var modelName: String? = nil
    var sortBy: SortBy = .createdAt
    var sortOrder: SortOrder = .desc
    var page: Int = 1
    var perPage: Int = AppConfig.defaultPerPage

    enum SortBy: String, CaseIterable {
        case createdAt = "created_at"
        case rating = "rating"
        case filename = "filename"
        case fileSize = "file_size"
    }

    enum SortOrder: String, CaseIterable {
        case asc = "asc"
        case desc = "desc"
    }

    var queryItems: [URLQueryItem] {
        var items: [URLQueryItem] = []

        if !query.isEmpty {
            items.append(URLQueryItem(name: "q", value: query))
        }
        if let rating {
            items.append(URLQueryItem(name: "rating", value: String(rating)))
        }
        if let isFavorite {
            items.append(URLQueryItem(name: "is_favorite", value: isFavorite ? "true" : "false"))
        }
        for tag in tags {
            items.append(URLQueryItem(name: "tags", value: tag))
        }
        if let modelName, !modelName.isEmpty {
            items.append(URLQueryItem(name: "model_name", value: modelName))
        }
        items.append(URLQueryItem(name: "sort_by", value: sortBy.rawValue))
        items.append(URLQueryItem(name: "sort_order", value: sortOrder.rawValue))
        items.append(URLQueryItem(name: "page", value: String(page)))
        items.append(URLQueryItem(name: "per_page", value: String(perPage)))

        return items
    }

    mutating func reset() {
        query = ""
        rating = nil
        isFavorite = nil
        tags = []
        modelName = nil
        sortBy = .createdAt
        sortOrder = .desc
        page = 1
    }
}
