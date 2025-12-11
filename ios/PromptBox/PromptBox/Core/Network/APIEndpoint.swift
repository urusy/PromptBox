import Foundation

enum APIEndpoint {
    // Auth
    case login
    case logout
    case me

    // Images
    case images
    case image(id: String)
    case updateImage(id: String)
    case deleteImage(id: String)
    case imageFile(path: String)
    case thumbnail(path: String)

    // Bulk operations (matches backend /bulk/ prefix)
    case bulkUpdate
    case bulkDelete
    case bulkRestore

    // Tags
    case tags

    // Smart Folders
    case smartFolders
    case createSmartFolder
    case smartFolder(id: String)
    case updateSmartFolder(id: String)
    case deleteSmartFolder(id: String)
    case smartFolderImages(id: String)

    // Stats
    case stats

    // Trash
    case trash
    case trashRestore(id: String)
    case trashDelete(id: String)
    case trashEmpty

    var path: String {
        switch self {
        case .login:
            return "/auth/login"
        case .logout:
            return "/auth/logout"
        case .me:
            return "/auth/me"
        case .images:
            return "/images"
        case .image(let id):
            return "/images/\(id)"
        case .updateImage(let id):
            return "/images/\(id)"
        case .deleteImage(let id):
            return "/images/\(id)"
        case .imageFile(let path):
            return "/storage/\(path)"
        case .thumbnail(let path):
            return "/storage/thumbnails/\(path)"
        case .bulkUpdate:
            return "/bulk/update"
        case .bulkDelete:
            return "/bulk/delete"
        case .bulkRestore:
            return "/bulk/restore"
        case .tags:
            return "/tags"
        case .smartFolders, .createSmartFolder:
            return "/smart-folders"
        case .smartFolder(let id), .updateSmartFolder(let id), .deleteSmartFolder(let id):
            return "/smart-folders/\(id)"
        case .smartFolderImages(let id):
            return "/smart-folders/\(id)/images"
        case .stats:
            return "/stats"
        case .trash:
            return "/images"  // Use images endpoint with include_deleted=true
        case .trashRestore(let id):
            return "/images/\(id)/restore"
        case .trashDelete(let id):
            return "/images/\(id)"
        case .trashEmpty:
            return "/bulk/delete"
        }
    }

    var method: String {
        switch self {
        case .login, .bulkUpdate, .bulkDelete, .bulkRestore, .trashRestore, .createSmartFolder:
            return "POST"
        case .logout, .trashDelete, .deleteImage, .deleteSmartFolder:
            return "DELETE"
        case .updateSmartFolder, .updateImage:
            return "PUT"
        case .trashEmpty:
            return "POST"
        default:
            return "GET"
        }
    }

    func url(baseURL: String) -> URL? {
        URL(string: baseURL + path)
    }
}
