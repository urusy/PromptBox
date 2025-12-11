import Foundation

@Observable
final class StatsService {
    static let shared = StatsService()

    private let apiClient = APIClient.shared

    private init() {}

    func fetchStats() async throws -> Stats {
        try await apiClient.request(endpoint: .stats)
    }
}
