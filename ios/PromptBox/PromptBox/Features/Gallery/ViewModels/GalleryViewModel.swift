import Foundation

@Observable
final class GalleryViewModel {
    var images: [ImageItem] = []
    var isLoading = false
    var isLoadingMore = false
    var errorMessage: String?
    var searchParams = SearchParams()
    var totalPages = 1
    var selectedImageIds: Set<String> = []
    var isSelectionMode = false

    private let imageService = ImageService.shared
    private let bulkService = BulkService.shared

    var hasMorePages: Bool {
        searchParams.page < totalPages
    }

    var selectedCount: Int {
        selectedImageIds.count
    }

    @MainActor
    func loadImages(refresh: Bool = false) async {
        if refresh {
            searchParams.page = 1
            isLoading = true
        }

        errorMessage = nil

        do {
            let response = try await imageService.fetchImages(params: searchParams)
            if refresh {
                images = response.items
            } else {
                images.append(contentsOf: response.items)
            }
            totalPages = response.totalPages
        } catch {
            errorMessage = error.localizedDescription
        }

        isLoading = false
        isLoadingMore = false
    }

    @MainActor
    func loadMoreIfNeeded(currentItem: ImageItem) async {
        guard !isLoadingMore && hasMorePages else { return }

        let thresholdIndex = images.index(images.endIndex, offsetBy: -5)
        if let currentIndex = images.firstIndex(where: { $0.id == currentItem.id }),
           currentIndex >= thresholdIndex {
            isLoadingMore = true
            searchParams.page += 1
            await loadImages()
        }
    }

    @MainActor
    func refresh() async {
        await loadImages(refresh: true)
    }

    func toggleSelection(for imageId: String) {
        if selectedImageIds.contains(imageId) {
            selectedImageIds.remove(imageId)
        } else {
            if selectedImageIds.count < AppConfig.maxBulkSize {
                selectedImageIds.insert(imageId)
            }
        }
    }

    func selectAll() {
        for image in images {
            if selectedImageIds.count >= AppConfig.maxBulkSize { break }
            selectedImageIds.insert(image.id)
        }
    }

    func clearSelection() {
        selectedImageIds.removeAll()
        isSelectionMode = false
    }

    @MainActor
    func bulkRate(_ rating: Int) async throws {
        guard !selectedImageIds.isEmpty else { return }
        _ = try await bulkService.bulkRate(ids: Array(selectedImageIds), rating: rating)
        await refresh()
        clearSelection()
    }

    @MainActor
    func bulkFavorite(_ isFavorite: Bool) async throws {
        guard !selectedImageIds.isEmpty else { return }
        _ = try await bulkService.bulkFavorite(ids: Array(selectedImageIds), isFavorite: isFavorite)
        await refresh()
        clearSelection()
    }

    @MainActor
    func bulkDelete() async throws {
        guard !selectedImageIds.isEmpty else { return }
        _ = try await bulkService.bulkDelete(ids: Array(selectedImageIds))
        await refresh()
        clearSelection()
    }

    @MainActor
    func bulkAddTag(_ tag: String) async throws {
        guard !selectedImageIds.isEmpty else { return }
        _ = try await bulkService.bulkTag(ids: Array(selectedImageIds), addTags: [tag], removeTags: nil)
        await refresh()
        clearSelection()
    }
}
