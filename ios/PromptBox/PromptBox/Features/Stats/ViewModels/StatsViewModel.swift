import Foundation

@Observable
final class StatsViewModel {
    var stats: Stats?
    var isLoading = false
    var errorMessage: String?

    private let statsService = StatsService.shared

    @MainActor
    func loadStats() async {
        isLoading = true
        errorMessage = nil

        do {
            stats = try await statsService.fetchStats()
        } catch {
            errorMessage = error.localizedDescription
        }

        isLoading = false
    }
}
