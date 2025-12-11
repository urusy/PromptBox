import Foundation

struct SmartFolder: Codable, Identifiable, Sendable, Hashable {
    let id: String
    let name: String
    let icon: String?
    let filters: SearchFilters
    let createdAt: Date
    let updatedAt: Date

    enum CodingKeys: String, CodingKey {
        case id
        case name
        case icon
        case filters
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

// Matches backend SearchFilters schema
struct SearchFilters: Codable, Sendable, Hashable {
    var q: String?
    var sourceTool: String?
    var modelType: String?
    var modelName: String?
    var samplerName: String?
    var minRating: Int?
    var exactRating: Int?
    var isFavorite: Bool?
    var needsImprovement: Bool?
    var tags: [String]?
    var loraName: String?
    var isXyzGrid: Bool?
    var isUpscaled: Bool?
    var minWidth: Int?
    var minHeight: Int?
    var sortBy: String?
    var sortOrder: String?

    enum CodingKeys: String, CodingKey {
        case q
        case sourceTool = "source_tool"
        case modelType = "model_type"
        case modelName = "model_name"
        case samplerName = "sampler_name"
        case minRating = "min_rating"
        case exactRating = "exact_rating"
        case isFavorite = "is_favorite"
        case needsImprovement = "needs_improvement"
        case tags
        case loraName = "lora_name"
        case isXyzGrid = "is_xyz_grid"
        case isUpscaled = "is_upscaled"
        case minWidth = "min_width"
        case minHeight = "min_height"
        case sortBy = "sort_by"
        case sortOrder = "sort_order"
    }

    init(
        q: String? = nil,
        sourceTool: String? = nil,
        modelType: String? = nil,
        modelName: String? = nil,
        samplerName: String? = nil,
        minRating: Int? = nil,
        exactRating: Int? = nil,
        isFavorite: Bool? = nil,
        needsImprovement: Bool? = nil,
        tags: [String]? = nil,
        loraName: String? = nil,
        isXyzGrid: Bool? = nil,
        isUpscaled: Bool? = nil,
        minWidth: Int? = nil,
        minHeight: Int? = nil,
        sortBy: String? = nil,
        sortOrder: String? = nil
    ) {
        self.q = q
        self.sourceTool = sourceTool
        self.modelType = modelType
        self.modelName = modelName
        self.samplerName = samplerName
        self.minRating = minRating
        self.exactRating = exactRating
        self.isFavorite = isFavorite
        self.needsImprovement = needsImprovement
        self.tags = tags
        self.loraName = loraName
        self.isXyzGrid = isXyzGrid
        self.isUpscaled = isUpscaled
        self.minWidth = minWidth
        self.minHeight = minHeight
        self.sortBy = sortBy
        self.sortOrder = sortOrder
    }
}

struct CreateSmartFolderRequest: Encodable {
    let name: String
    let icon: String?
    let filters: SearchFilters
}

struct UpdateSmartFolderRequest: Encodable {
    let name: String?
    let icon: String?
    let filters: SearchFilters?
}
