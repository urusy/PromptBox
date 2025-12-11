import Foundation

@Observable
final class SmartFoldersViewModel {
    var folders: [SmartFolder] = []
    var isLoading = false
    var errorMessage: String?

    private let service = SmartFolderService.shared

    @MainActor
    func loadFolders() async {
        isLoading = true
        errorMessage = nil

        do {
            folders = try await service.fetchSmartFolders()
        } catch {
            errorMessage = error.localizedDescription
        }

        isLoading = false
    }

    @MainActor
    func deleteFolder(id: String) async {
        do {
            try await service.deleteSmartFolder(id: id)
            folders.removeAll { $0.id == id }
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
