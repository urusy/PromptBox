import Foundation

// 一覧表示用（ImageListResponse from backend）
struct ImageItem: Codable, Identifiable, Sendable, Hashable {
    let id: String
    let sourceTool: String
    let modelType: String?
    let storagePath: String
    let thumbnailPath: String
    let width: Int
    let height: Int
    let modelName: String?
    let rating: Int
    let isFavorite: Bool
    let createdAt: Date

    enum CodingKeys: String, CodingKey {
        case id
        case sourceTool = "source_tool"
        case modelType = "model_type"
        case storagePath = "storage_path"
        case thumbnailPath = "thumbnail_path"
        case width
        case height
        case modelName = "model_name"
        case rating
        case isFavorite = "is_favorite"
        case createdAt = "created_at"
    }

    var imageURL: URL? {
        let baseURL = AppConfig.apiBaseURL.replacingOccurrences(of: "/api", with: "")
        return URL(string: "\(baseURL)/storage/\(storagePath)")
    }

    var thumbnailURL: URL? {
        let baseURL = AppConfig.apiBaseURL.replacingOccurrences(of: "/api", with: "")
        return URL(string: "\(baseURL)/storage/\(thumbnailPath)")
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: ImageItem, rhs: ImageItem) -> Bool {
        lhs.id == rhs.id
    }
}

// LoRA情報
struct LoraInfo: Codable, Sendable {
    let name: String
    let weight: Double?
    let weightClip: Double?
    let hash: String?

    enum CodingKeys: String, CodingKey {
        case name
        case weight
        case weightClip = "weight_clip"
        case hash
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        name = try container.decode(String.self, forKey: .name)
        // weightはIntまたはDoubleの可能性がある
        if let intWeight = try? container.decode(Int.self, forKey: .weight) {
            weight = Double(intWeight)
        } else {
            weight = try container.decodeIfPresent(Double.self, forKey: .weight)
        }
        if let intWeightClip = try? container.decode(Int.self, forKey: .weightClip) {
            weightClip = Double(intWeightClip)
        } else {
            weightClip = try container.decodeIfPresent(Double.self, forKey: .weightClip)
        }
        hash = try container.decodeIfPresent(String.self, forKey: .hash)
    }
}

// ControlNet情報
struct ControlNetInfo: Codable, Sendable {
    let model: String
    let weight: Double?
    let guidanceStart: Double?
    let guidanceEnd: Double?
    let preprocessor: String?

    enum CodingKeys: String, CodingKey {
        case model
        case weight
        case guidanceStart = "guidance_start"
        case guidanceEnd = "guidance_end"
        case preprocessor
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        model = try container.decode(String.self, forKey: .model)
        // weightはIntまたはDoubleの可能性がある
        if let intWeight = try? container.decode(Int.self, forKey: .weight) {
            weight = Double(intWeight)
        } else {
            weight = try container.decodeIfPresent(Double.self, forKey: .weight)
        }
        if let intStart = try? container.decode(Int.self, forKey: .guidanceStart) {
            guidanceStart = Double(intStart)
        } else {
            guidanceStart = try container.decodeIfPresent(Double.self, forKey: .guidanceStart)
        }
        if let intEnd = try? container.decode(Int.self, forKey: .guidanceEnd) {
            guidanceEnd = Double(intEnd)
        } else {
            guidanceEnd = try container.decodeIfPresent(Double.self, forKey: .guidanceEnd)
        }
        preprocessor = try container.decodeIfPresent(String.self, forKey: .preprocessor)
    }
}

// Embedding情報
struct EmbeddingInfo: Codable, Sendable {
    let name: String
    let hash: String?
}

// 詳細表示用（ImageResponse from backend）
struct ImageDetail: Codable, Identifiable, Sendable {
    let id: String
    let sourceTool: String
    let modelType: String?
    let hasMetadata: Bool
    let originalFilename: String
    let storagePath: String
    let thumbnailPath: String
    let fileHash: String
    let width: Int
    let height: Int
    let fileSizeBytes: Int
    let positivePrompt: String?
    let negativePrompt: String?
    let modelName: String?
    let samplerName: String?
    let scheduler: String?
    let steps: Int?
    let cfgScale: Double?
    let seed: Int64?
    let loras: [LoraInfo]
    let controlnets: [ControlNetInfo]
    let embeddings: [EmbeddingInfo]
    let modelParams: [String: AnyCodable]
    let workflowExtras: [String: AnyCodable]
    let rawMetadata: [String: AnyCodable]?
    let rating: Int
    let isFavorite: Bool
    let needsImprovement: Bool
    let userTags: [String]
    let userMemo: String?
    let createdAt: Date
    let updatedAt: Date
    let deletedAt: Date?
    let prevId: String?
    let nextId: String?

    enum CodingKeys: String, CodingKey {
        case id
        case sourceTool = "source_tool"
        case modelType = "model_type"
        case hasMetadata = "has_metadata"
        case originalFilename = "original_filename"
        case storagePath = "storage_path"
        case thumbnailPath = "thumbnail_path"
        case fileHash = "file_hash"
        case width
        case height
        case fileSizeBytes = "file_size_bytes"
        case positivePrompt = "positive_prompt"
        case negativePrompt = "negative_prompt"
        case modelName = "model_name"
        case samplerName = "sampler_name"
        case scheduler
        case steps
        case cfgScale = "cfg_scale"
        case seed
        case loras
        case controlnets
        case embeddings
        case modelParams = "model_params"
        case workflowExtras = "workflow_extras"
        case rawMetadata = "raw_metadata"
        case rating
        case isFavorite = "is_favorite"
        case needsImprovement = "needs_improvement"
        case userTags = "user_tags"
        case userMemo = "user_memo"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
        case deletedAt = "deleted_at"
        case prevId = "prev_id"
        case nextId = "next_id"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(String.self, forKey: .id)
        sourceTool = try container.decode(String.self, forKey: .sourceTool)
        modelType = try container.decodeIfPresent(String.self, forKey: .modelType)
        hasMetadata = try container.decode(Bool.self, forKey: .hasMetadata)
        originalFilename = try container.decode(String.self, forKey: .originalFilename)
        storagePath = try container.decode(String.self, forKey: .storagePath)
        thumbnailPath = try container.decode(String.self, forKey: .thumbnailPath)
        fileHash = try container.decode(String.self, forKey: .fileHash)
        width = try container.decode(Int.self, forKey: .width)
        height = try container.decode(Int.self, forKey: .height)
        fileSizeBytes = try container.decode(Int.self, forKey: .fileSizeBytes)
        positivePrompt = try container.decodeIfPresent(String.self, forKey: .positivePrompt)
        negativePrompt = try container.decodeIfPresent(String.self, forKey: .negativePrompt)
        modelName = try container.decodeIfPresent(String.self, forKey: .modelName)
        samplerName = try container.decodeIfPresent(String.self, forKey: .samplerName)
        scheduler = try container.decodeIfPresent(String.self, forKey: .scheduler)
        steps = try container.decodeIfPresent(Int.self, forKey: .steps)

        // cfg_scaleはDecimal型（文字列として返される可能性がある）
        if let stringValue = try? container.decode(String.self, forKey: .cfgScale) {
            cfgScale = Double(stringValue)
        } else if let intValue = try? container.decode(Int.self, forKey: .cfgScale) {
            cfgScale = Double(intValue)
        } else {
            cfgScale = try container.decodeIfPresent(Double.self, forKey: .cfgScale)
        }

        seed = try container.decodeIfPresent(Int64.self, forKey: .seed)
        loras = try container.decode([LoraInfo].self, forKey: .loras)
        controlnets = try container.decode([ControlNetInfo].self, forKey: .controlnets)
        embeddings = try container.decode([EmbeddingInfo].self, forKey: .embeddings)
        modelParams = try container.decode([String: AnyCodable].self, forKey: .modelParams)
        workflowExtras = try container.decode([String: AnyCodable].self, forKey: .workflowExtras)
        rawMetadata = try container.decodeIfPresent([String: AnyCodable].self, forKey: .rawMetadata)
        rating = try container.decode(Int.self, forKey: .rating)
        isFavorite = try container.decode(Bool.self, forKey: .isFavorite)
        needsImprovement = try container.decode(Bool.self, forKey: .needsImprovement)
        userTags = try container.decode([String].self, forKey: .userTags)
        userMemo = try container.decodeIfPresent(String.self, forKey: .userMemo)
        createdAt = try container.decode(Date.self, forKey: .createdAt)
        updatedAt = try container.decode(Date.self, forKey: .updatedAt)
        deletedAt = try container.decodeIfPresent(Date.self, forKey: .deletedAt)
        prevId = try container.decodeIfPresent(String.self, forKey: .prevId)
        nextId = try container.decodeIfPresent(String.self, forKey: .nextId)
    }

    var imageURL: URL? {
        let baseURL = AppConfig.apiBaseURL.replacingOccurrences(of: "/api", with: "")
        return URL(string: "\(baseURL)/storage/\(storagePath)")
    }

    var thumbnailURL: URL? {
        let baseURL = AppConfig.apiBaseURL.replacingOccurrences(of: "/api", with: "")
        return URL(string: "\(baseURL)/storage/\(thumbnailPath)")
    }
}

// 任意のJSON値をデコードするためのラッパー
struct AnyCodable: Codable, Sendable {
    let value: Any

    init(_ value: Any) {
        self.value = value
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        if container.decodeNil() {
            self.value = NSNull()
        } else if let bool = try? container.decode(Bool.self) {
            self.value = bool
        } else if let int = try? container.decode(Int.self) {
            self.value = int
        } else if let double = try? container.decode(Double.self) {
            self.value = double
        } else if let string = try? container.decode(String.self) {
            self.value = string
        } else if let array = try? container.decode([AnyCodable].self) {
            self.value = array.map { $0.value }
        } else if let dictionary = try? container.decode([String: AnyCodable].self) {
            self.value = dictionary.mapValues { $0.value }
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unable to decode value")
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        switch value {
        case is NSNull:
            try container.encodeNil()
        case let bool as Bool:
            try container.encode(bool)
        case let int as Int:
            try container.encode(int)
        case let double as Double:
            try container.encode(double)
        case let string as String:
            try container.encode(string)
        case let array as [Any]:
            try container.encode(array.map { AnyCodable($0) })
        case let dictionary as [String: Any]:
            try container.encode(dictionary.mapValues { AnyCodable($0) })
        default:
            throw EncodingError.invalidValue(value, EncodingError.Context(codingPath: encoder.codingPath, debugDescription: "Unable to encode value"))
        }
    }
}

struct PaginatedResponse<T: Decodable>: Decodable {
    let items: [T]
    let total: Int
    let page: Int
    let perPage: Int
    let totalPages: Int

    enum CodingKeys: String, CodingKey {
        case items
        case total
        case page
        case perPage = "per_page"
        case totalPages = "total_pages"
    }
}

typealias ImageListResponse = PaginatedResponse<ImageItem>
