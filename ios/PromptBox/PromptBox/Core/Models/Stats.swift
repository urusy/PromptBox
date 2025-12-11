import Foundation

// Backend StatsResponse structure
struct Stats: Decodable, Sendable {
    let overview: StatsOverview
    let byModelType: [CountItem]
    let bySourceTool: [CountItem]
    let byModelName: [CountItem]
    let bySampler: [CountItem]
    let byLora: [CountItem]
    let byRating: [RatingDistribution]
    let dailyCounts: [TimeSeriesItem]

    enum CodingKeys: String, CodingKey {
        case overview
        case byModelType = "by_model_type"
        case bySourceTool = "by_source_tool"
        case byModelName = "by_model_name"
        case bySampler = "by_sampler"
        case byLora = "by_lora"
        case byRating = "by_rating"
        case dailyCounts = "daily_counts"
    }
}

struct StatsOverview: Decodable, Sendable {
    let totalImages: Int
    let totalFavorites: Int
    let totalRated: Int
    let totalUnrated: Int
    let avgRating: Double?

    enum CodingKeys: String, CodingKey {
        case totalImages = "total_images"
        case totalFavorites = "total_favorites"
        case totalRated = "total_rated"
        case totalUnrated = "total_unrated"
        case avgRating = "avg_rating"
    }
}

struct CountItem: Decodable, Identifiable, Sendable {
    let name: String
    let count: Int

    var id: String { name }
}

struct RatingDistribution: Decodable, Identifiable, Sendable {
    let rating: Int
    let count: Int

    var id: Int { rating }
}

struct TimeSeriesItem: Decodable, Identifiable, Sendable {
    let date: String
    let count: Int

    var id: String { date }
}
