import Foundation

@Observable
final class ImageDetailViewModel {
    var image: ImageDetail?
    var isLoading = false
    var errorMessage: String?
    var newTag = ""

    private let imageService = ImageService.shared

    let imageId: String
    let images: [ImageItem]

    init(imageId: String, images: [ImageItem] = []) {
        self.imageId = imageId
        self.images = images
    }

    var currentIndex: Int? {
        images.firstIndex(where: { $0.id == imageId })
    }

    var hasPrevious: Bool {
        guard let index = currentIndex else { return false }
        return index > 0
    }

    var hasNext: Bool {
        guard let index = currentIndex else { return false }
        return index < images.count - 1
    }

    var previousImageId: String? {
        guard let index = currentIndex, index > 0 else { return nil }
        return images[index - 1].id
    }

    var nextImageId: String? {
        guard let index = currentIndex, index < images.count - 1 else { return nil }
        return images[index + 1].id
    }

    @MainActor
    func loadImage() async {
        isLoading = true
        errorMessage = nil

        do {
            image = try await imageService.fetchImage(id: imageId)
        } catch {
            errorMessage = error.localizedDescription
        }

        isLoading = false
    }

    @MainActor
    func updateRating(_ rating: Int) async {
        guard let currentImage = image else { return }
        do {
            image = try await imageService.updateImage(
                id: currentImage.id,
                rating: rating == currentImage.rating ? 0 : rating
            )
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    @MainActor
    func toggleFavorite() async {
        guard let currentImage = image else { return }
        do {
            image = try await imageService.updateImage(
                id: currentImage.id,
                isFavorite: !currentImage.isFavorite
            )
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    @MainActor
    func updateMemo(_ memo: String) async {
        guard let currentImage = image else { return }
        do {
            image = try await imageService.updateImage(
                id: currentImage.id,
                memo: memo.isEmpty ? nil : memo
            )
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    @MainActor
    func addTag(_ tagName: String) async {
        guard let currentImage = image, !tagName.isEmpty else { return }
        do {
            image = try await imageService.addTag(imageId: currentImage.id, tagName: tagName)
            newTag = ""
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    @MainActor
    func removeTag(_ tagName: String) async {
        guard let currentImage = image else { return }
        do {
            image = try await imageService.removeTag(imageId: currentImage.id, tagName: tagName)
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    @MainActor
    func deleteImage() async throws {
        guard let currentImage = image else { return }
        try await imageService.deleteImage(id: currentImage.id)
    }
}
