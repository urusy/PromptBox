# PromptBox iOS App 設計書

## 概要

本ドキュメントは、PromptBox WebアプリケーションをSwiftUIを使用してiPhone/iPad向けネイティブアプリとして開発するための設計書です。

## 1. アプリケーション概要

### 1.1 対象プラットフォーム
- iOS 17.0+
- iPadOS 17.0+

### 1.2 開発環境
- Xcode 15+
- Swift 5.9+
- SwiftUI

### 1.3 主要フレームワーク・ライブラリ

| カテゴリ | ライブラリ | 用途 |
|---------|-----------|------|
| ネットワーク | URLSession / Alamofire | API通信 |
| 画像キャッシュ | Kingfisher / SDWebImageSwiftUI | 画像読み込み・キャッシュ |
| チャート | Swift Charts | 統計グラフ表示 |
| 状態管理 | SwiftUI @Observable | アプリ状態管理 |
| データ永続化 | UserDefaults / SwiftData | 設定・キャッシュ保存 |
| キーチェーン | KeychainAccess | 認証情報保存 |

---

## 2. アーキテクチャ

### 2.1 全体構成

```
PromptBoxApp/
├── App/
│   ├── PromptBoxApp.swift          # エントリーポイント
│   └── AppDelegate.swift           # アプリデリゲート
│
├── Core/
│   ├── Network/
│   │   ├── APIClient.swift         # API通信クライアント
│   │   ├── APIEndpoints.swift      # エンドポイント定義
│   │   └── APIError.swift          # エラー定義
│   │
│   ├── Models/                     # データモデル
│   │   ├── Image.swift
│   │   ├── Stats.swift
│   │   ├── SmartFolder.swift
│   │   ├── SearchPreset.swift
│   │   └── User.swift
│   │
│   ├── Services/                   # ビジネスロジック
│   │   ├── AuthService.swift
│   │   ├── ImageService.swift
│   │   ├── StatsService.swift
│   │   ├── BatchService.swift
│   │   └── TagService.swift
│   │
│   └── Utilities/
│       ├── Extensions/
│       ├── Helpers/
│       └── Constants.swift
│
├── Features/
│   ├── Auth/
│   │   ├── LoginView.swift
│   │   └── LoginViewModel.swift
│   │
│   ├── Gallery/
│   │   ├── Views/
│   │   │   ├── GalleryView.swift
│   │   │   ├── ImageGridView.swift
│   │   │   ├── ImageCardView.swift
│   │   │   ├── SearchFilterView.swift
│   │   │   └── SelectionToolbarView.swift
│   │   └── GalleryViewModel.swift
│   │
│   ├── Detail/
│   │   ├── Views/
│   │   │   ├── ImageDetailView.swift
│   │   │   ├── MetadataView.swift
│   │   │   ├── TagEditorView.swift
│   │   │   └── MemoEditorView.swift
│   │   └── DetailViewModel.swift
│   │
│   ├── Stats/
│   │   ├── Views/
│   │   │   ├── StatsView.swift
│   │   │   ├── OverviewCardsView.swift
│   │   │   └── ChartViews/
│   │   └── StatsViewModel.swift
│   │
│   ├── SmartFolders/
│   │   ├── Views/
│   │   │   ├── SmartFoldersView.swift
│   │   │   └── FolderEditorView.swift
│   │   └── SmartFoldersViewModel.swift
│   │
│   ├── Trash/
│   │   ├── TrashView.swift
│   │   └── TrashViewModel.swift
│   │
│   ├── Duplicates/
│   │   ├── DuplicatesView.swift
│   │   └── DuplicatesViewModel.swift
│   │
│   └── Slideshow/
│       ├── SlideshowView.swift
│       └── SlideshowViewModel.swift
│
├── Shared/
│   ├── Components/
│   │   ├── StarRatingView.swift
│   │   ├── PaginationView.swift
│   │   ├── LoadingView.swift
│   │   ├── ErrorView.swift
│   │   └── ToastView.swift
│   │
│   ├── Modifiers/
│   │   └── ViewModifiers.swift
│   │
│   └── Styles/
│       └── AppStyles.swift
│
└── Resources/
    ├── Assets.xcassets
    ├── Localizable.strings
    └── Info.plist
```

### 2.2 MVVMアーキテクチャ

```
┌─────────────────────────────────────────────────────────┐
│                        View (SwiftUI)                    │
│  - UI表示                                                │
│  - ユーザーインタラクション                               │
│  - ViewModelへのバインディング                            │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                   ViewModel (@Observable)                │
│  - UIロジック                                            │
│  - 状態管理                                              │
│  - Serviceの呼び出し                                     │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                        Service                           │
│  - ビジネスロジック                                       │
│  - API呼び出し                                           │
│  - データ変換                                            │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                       APIClient                          │
│  - HTTP通信                                              │
│  - エラーハンドリング                                     │
│  - 認証管理                                              │
└─────────────────────────────────────────────────────────┘
```

---

## 3. データモデル

### 3.1 Image モデル

```swift
import Foundation

struct Image: Identifiable, Codable {
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
    let modelParams: [String: AnyCodable]?
    let workflowExtras: [String: AnyCodable]?
    let rawMetadata: [String: AnyCodable]?
    var rating: Int
    var isFavorite: Bool
    var needsImprovement: Bool
    var userTags: [String]
    var userMemo: String?
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
        case width, height
        case fileSizeBytes = "file_size_bytes"
        case positivePrompt = "positive_prompt"
        case negativePrompt = "negative_prompt"
        case modelName = "model_name"
        case samplerName = "sampler_name"
        case scheduler, steps
        case cfgScale = "cfg_scale"
        case seed, loras, controlnets, embeddings
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
}

struct ImageListItem: Identifiable, Codable {
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
        case width, height
        case modelName = "model_name"
        case rating
        case isFavorite = "is_favorite"
        case createdAt = "created_at"
    }
}

struct LoraInfo: Codable {
    let name: String
    let weight: Double?
}

struct ControlNetInfo: Codable {
    let model: String
    let weight: Double?
    let preprocessor: String?
}

struct EmbeddingInfo: Codable {
    let name: String
}
```

### 3.2 SearchParams モデル

```swift
struct ImageSearchParams: Codable {
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
    var includeDeleted: Bool?
    var page: Int = 1
    var perPage: Int = 24
    var sortBy: String = "created_at"
    var sortOrder: String = "desc"

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
        case includeDeleted = "include_deleted"
        case page
        case perPage = "per_page"
        case sortBy = "sort_by"
        case sortOrder = "sort_order"
    }
}
```

### 3.3 Stats モデル

```swift
struct StatsResponse: Codable {
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

struct StatsOverview: Codable {
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

struct CountItem: Codable, Identifiable {
    var id: String { name }
    let name: String
    let count: Int
}

struct RatingDistribution: Codable, Identifiable {
    var id: Int { rating }
    let rating: Int
    let count: Int
}

struct TimeSeriesItem: Codable, Identifiable {
    var id: String { date }
    let date: String
    let count: Int
}
```

### 3.4 SmartFolder & SearchPreset モデル

```swift
struct SmartFolder: Identifiable, Codable {
    let id: String
    var name: String
    var icon: String?
    var filters: SearchFilters
    let createdAt: Date
    let updatedAt: Date

    enum CodingKeys: String, CodingKey {
        case id, name, icon, filters
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

struct SearchFilters: Codable {
    var q: String?
    var sourceTool: String?
    var modelType: String?
    var modelName: String?
    var minRating: Int?
    var exactRating: Int?
    var isFavorite: Bool?
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
        case minRating = "min_rating"
        case exactRating = "exact_rating"
        case isFavorite = "is_favorite"
        case isXyzGrid = "is_xyz_grid"
        case isUpscaled = "is_upscaled"
        case minWidth = "min_width"
        case minHeight = "min_height"
        case sortBy = "sort_by"
        case sortOrder = "sort_order"
    }
}

struct SearchPreset: Identifiable, Codable {
    let id: String
    var name: String
    var filters: SearchFilters
    let createdAt: Date
    let updatedAt: Date

    enum CodingKeys: String, CodingKey {
        case id, name, filters
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}
```

---

## 4. ネットワーク層

### 4.1 APIClient

```swift
import Foundation

actor APIClient {
    static let shared = APIClient()

    private let baseURL: URL
    private let session: URLSession
    private var authToken: String?

    private init() {
        // 設定からベースURLを取得
        self.baseURL = URL(string: AppConfig.apiBaseURL)!

        let config = URLSessionConfiguration.default
        config.httpCookieStorage = HTTPCookieStorage.shared
        config.httpShouldSetCookies = true
        self.session = URLSession(configuration: config)
    }

    func setAuthToken(_ token: String?) {
        self.authToken = token
    }

    func request<T: Decodable>(
        endpoint: APIEndpoint,
        method: HTTPMethod = .get,
        body: Encodable? = nil
    ) async throws -> T {
        var request = URLRequest(url: baseURL.appendingPathComponent(endpoint.path))
        request.httpMethod = method.rawValue
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        if let token = authToken {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }

        if let body = body {
            request.httpBody = try JSONEncoder().encode(body)
        }

        let (data, response) = try await session.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }

        switch httpResponse.statusCode {
        case 200...299:
            let decoder = JSONDecoder()
            decoder.dateDecodingStrategy = .iso8601
            return try decoder.decode(T.self, from: data)
        case 401:
            throw APIError.unauthorized
        case 404:
            throw APIError.notFound
        case 422:
            throw APIError.validationError(try? JSONDecoder().decode(ValidationError.self, from: data))
        default:
            throw APIError.serverError(httpResponse.statusCode)
        }
    }
}

enum HTTPMethod: String {
    case get = "GET"
    case post = "POST"
    case put = "PUT"
    case patch = "PATCH"
    case delete = "DELETE"
}

enum APIError: Error, LocalizedError {
    case invalidResponse
    case unauthorized
    case notFound
    case validationError(ValidationError?)
    case serverError(Int)
    case networkError(Error)

    var errorDescription: String? {
        switch self {
        case .invalidResponse:
            return "Invalid server response"
        case .unauthorized:
            return "Authentication required"
        case .notFound:
            return "Resource not found"
        case .validationError(let error):
            return error?.detail.first?.msg ?? "Validation error"
        case .serverError(let code):
            return "Server error: \(code)"
        case .networkError(let error):
            return error.localizedDescription
        }
    }
}

struct ValidationError: Codable {
    let detail: [ValidationDetail]
}

struct ValidationDetail: Codable {
    let loc: [String]
    let msg: String
    let type: String
}
```

### 4.2 APIEndpoints

```swift
enum APIEndpoint {
    // Auth
    case login
    case logout
    case me

    // Images
    case images
    case image(id: String)
    case imageUpdate(id: String)
    case imageDelete(id: String)
    case imageRestore(id: String)

    // Batch
    case bulkUpdate
    case bulkDelete
    case bulkRestore

    // Stats
    case stats
    case ratingAnalysis
    case modelsForAnalysis
    case modelRatingDistribution

    // Smart Folders
    case smartFolders
    case smartFolder(id: String)

    // Search Presets
    case searchPresets
    case searchPreset(id: String)

    // Tags
    case tags

    // Duplicates
    case duplicatesInfo
    case duplicatesDeleteAll
    case duplicatesDeleteFile(filename: String)

    // Export
    case exportMetadata
    case exportPrompts

    var path: String {
        switch self {
        case .login: return "/auth/login"
        case .logout: return "/auth/logout"
        case .me: return "/auth/me"
        case .images: return "/images"
        case .image(let id): return "/images/\(id)"
        case .imageUpdate(let id): return "/images/\(id)"
        case .imageDelete(let id): return "/images/\(id)"
        case .imageRestore(let id): return "/images/\(id)/restore"
        case .bulkUpdate: return "/bulk/update"
        case .bulkDelete: return "/bulk/delete"
        case .bulkRestore: return "/bulk/restore"
        case .stats: return "/stats"
        case .ratingAnalysis: return "/stats/rating-analysis"
        case .modelsForAnalysis: return "/stats/models-for-analysis"
        case .modelRatingDistribution: return "/stats/model-rating-distribution"
        case .smartFolders: return "/smart-folders"
        case .smartFolder(let id): return "/smart-folders/\(id)"
        case .searchPresets: return "/search-presets"
        case .searchPreset(let id): return "/search-presets/\(id)"
        case .tags: return "/tags"
        case .duplicatesInfo: return "/duplicates/info"
        case .duplicatesDeleteAll: return "/duplicates/delete-all"
        case .duplicatesDeleteFile(let filename): return "/duplicates/\(filename)"
        case .exportMetadata: return "/export/metadata"
        case .exportPrompts: return "/export/prompts"
        }
    }
}
```

---

## 5. サービス層

### 5.1 AuthService

```swift
import Foundation

@Observable
class AuthService {
    static let shared = AuthService()

    private(set) var isAuthenticated = false
    private(set) var currentUser: User?

    private let keychain = KeychainAccess(service: "com.promptbox.app")

    private init() {
        // 起動時に保存済み認証情報を確認
        Task {
            await checkAuth()
        }
    }

    func login(username: String, password: String) async throws {
        let response: LoginResponse = try await APIClient.shared.request(
            endpoint: .login,
            method: .post,
            body: LoginRequest(username: username, password: password)
        )

        isAuthenticated = true
        currentUser = User(username: response.username)

        // Keychainに保存
        try keychain.set(username, key: "username")
    }

    func logout() async throws {
        try await APIClient.shared.request(
            endpoint: .logout,
            method: .post
        ) as EmptyResponse

        isAuthenticated = false
        currentUser = nil

        try keychain.remove("username")
    }

    func checkAuth() async {
        do {
            let user: User = try await APIClient.shared.request(endpoint: .me)
            isAuthenticated = true
            currentUser = user
        } catch {
            isAuthenticated = false
            currentUser = nil
        }
    }
}

struct LoginRequest: Codable {
    let username: String
    let password: String
}

struct LoginResponse: Codable {
    let username: String
}

struct User: Codable {
    let username: String
}

struct EmptyResponse: Codable {}
```

### 5.2 ImageService

```swift
import Foundation

class ImageService {
    static let shared = ImageService()

    private init() {}

    func fetchImages(params: ImageSearchParams) async throws -> PaginatedResponse<ImageListItem> {
        return try await APIClient.shared.request(
            endpoint: .images,
            queryParams: params.toQueryItems()
        )
    }

    func fetchImage(id: String, searchParams: ImageSearchParams? = nil) async throws -> Image {
        return try await APIClient.shared.request(
            endpoint: .image(id: id),
            queryParams: searchParams?.toQueryItems()
        )
    }

    func updateImage(id: String, update: ImageUpdate) async throws -> Image {
        return try await APIClient.shared.request(
            endpoint: .imageUpdate(id: id),
            method: .patch,
            body: update
        )
    }

    func deleteImage(id: String, permanent: Bool = false) async throws {
        try await APIClient.shared.request(
            endpoint: .imageDelete(id: id),
            method: .delete,
            queryParams: [URLQueryItem(name: "permanent", value: String(permanent))]
        ) as EmptyResponse
    }

    func restoreImage(id: String) async throws {
        try await APIClient.shared.request(
            endpoint: .imageRestore(id: id),
            method: .post
        ) as EmptyResponse
    }
}

struct ImageUpdate: Codable {
    var rating: Int?
    var isFavorite: Bool?
    var needsImprovement: Bool?
    var userTags: [String]?
    var userMemo: String?

    enum CodingKeys: String, CodingKey {
        case rating
        case isFavorite = "is_favorite"
        case needsImprovement = "needs_improvement"
        case userTags = "user_tags"
        case userMemo = "user_memo"
    }
}

struct PaginatedResponse<T: Codable>: Codable {
    let items: [T]
    let total: Int
    let page: Int
    let perPage: Int
    let pages: Int

    enum CodingKeys: String, CodingKey {
        case items, total, page
        case perPage = "per_page"
        case pages
    }
}
```

---

## 6. View実装

### 6.1 GalleryView

```swift
import SwiftUI

struct GalleryView: View {
    @State private var viewModel = GalleryViewModel()
    @State private var showFilters = false
    @State private var showSlideshow = false

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // 検索バー
                SearchBarView(
                    searchText: $viewModel.searchParams.q,
                    onSearch: { viewModel.search() },
                    onFilterTap: { showFilters = true }
                )

                // 画像グリッド
                if viewModel.isLoading && viewModel.images.isEmpty {
                    LoadingView()
                } else if viewModel.images.isEmpty {
                    EmptyStateView(message: "画像が見つかりません")
                } else {
                    ImageGridView(
                        images: viewModel.images,
                        selectedIds: $viewModel.selectedIds,
                        isSelectionMode: viewModel.isSelectionMode,
                        gridSize: viewModel.gridSize,
                        onImageTap: { image in
                            viewModel.selectedImage = image
                        },
                        onLoadMore: {
                            viewModel.loadMore()
                        }
                    )
                }
            }
            .navigationTitle("ギャラリー")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Menu {
                        Picker("表示サイズ", selection: $viewModel.gridSize) {
                            Text("小").tag(GridSize.small)
                            Text("中").tag(GridSize.medium)
                            Text("大").tag(GridSize.large)
                        }
                    } label: {
                        Image(systemName: "square.grid.3x3")
                    }
                }

                ToolbarItem(placement: .topBarTrailing) {
                    HStack {
                        Button {
                            viewModel.toggleSelectionMode()
                        } label: {
                            Image(systemName: viewModel.isSelectionMode ? "checkmark.circle.fill" : "checkmark.circle")
                        }

                        Button {
                            showSlideshow = true
                        } label: {
                            Image(systemName: "play.rectangle")
                        }
                        .disabled(viewModel.images.isEmpty)
                    }
                }
            }
            .sheet(isPresented: $showFilters) {
                SearchFilterView(
                    params: $viewModel.searchParams,
                    onApply: {
                        showFilters = false
                        viewModel.search()
                    }
                )
            }
            .fullScreenCover(isPresented: $showSlideshow) {
                SlideshowView(images: viewModel.images)
            }
            .navigationDestination(item: $viewModel.selectedImage) { image in
                ImageDetailView(
                    imageId: image.id,
                    searchParams: viewModel.searchParams
                )
            }
            .overlay(alignment: .bottom) {
                if viewModel.isSelectionMode {
                    SelectionToolbarView(
                        selectedCount: viewModel.selectedIds.count,
                        onRatingSet: { rating in
                            viewModel.setRatingForSelected(rating)
                        },
                        onFavorite: {
                            viewModel.setFavoriteForSelected(true)
                        },
                        onAddTag: { tag in
                            viewModel.addTagForSelected(tag)
                        },
                        onDelete: {
                            viewModel.deleteSelected()
                        },
                        onClose: {
                            viewModel.toggleSelectionMode()
                        }
                    )
                    .transition(.move(edge: .bottom))
                }
            }
        }
    }
}
```

### 6.2 ImageGridView

```swift
import SwiftUI

struct ImageGridView: View {
    let images: [ImageListItem]
    @Binding var selectedIds: Set<String>
    let isSelectionMode: Bool
    let gridSize: GridSize
    let onImageTap: (ImageListItem) -> Void
    let onLoadMore: () -> Void

    private var columns: [GridItem] {
        let count: Int
        switch gridSize {
        case .small:
            count = UIDevice.current.userInterfaceIdiom == .pad ? 6 : 4
        case .medium:
            count = UIDevice.current.userInterfaceIdiom == .pad ? 4 : 3
        case .large:
            count = UIDevice.current.userInterfaceIdiom == .pad ? 3 : 2
        }
        return Array(repeating: GridItem(.flexible(), spacing: 2), count: count)
    }

    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 2) {
                ForEach(images) { image in
                    ImageCardView(
                        image: image,
                        isSelected: selectedIds.contains(image.id),
                        isSelectionMode: isSelectionMode
                    )
                    .onTapGesture {
                        if isSelectionMode {
                            toggleSelection(image.id)
                        } else {
                            onImageTap(image)
                        }
                    }
                    .onLongPressGesture {
                        toggleSelection(image.id)
                    }
                    .onAppear {
                        // 最後から5番目で次ページ読み込み
                        if image.id == images.suffix(5).first?.id {
                            onLoadMore()
                        }
                    }
                }
            }
            .padding(2)
        }
    }

    private func toggleSelection(_ id: String) {
        if selectedIds.contains(id) {
            selectedIds.remove(id)
        } else {
            selectedIds.insert(id)
        }
    }
}

enum GridSize: String, CaseIterable {
    case small, medium, large
}
```

### 6.3 ImageCardView

```swift
import SwiftUI
import Kingfisher

struct ImageCardView: View {
    let image: ImageListItem
    let isSelected: Bool
    let isSelectionMode: Bool

    @Environment(\.colorScheme) private var colorScheme

    private var thumbnailURL: URL? {
        URL(string: "\(AppConfig.apiBaseURL)/storage/\(image.thumbnailPath)")
    }

    var body: some View {
        GeometryReader { geometry in
            ZStack(alignment: .topTrailing) {
                // サムネイル画像
                KFImage(thumbnailURL)
                    .resizable()
                    .aspectRatio(contentMode: .fill)
                    .frame(width: geometry.size.width, height: geometry.size.width)
                    .clipped()

                // オーバーレイ情報
                VStack {
                    Spacer()

                    HStack {
                        // 評価バッジ
                        if image.rating > 0 {
                            HStack(spacing: 2) {
                                Image(systemName: "star.fill")
                                    .font(.caption2)
                                Text("\(image.rating)")
                                    .font(.caption2)
                            }
                            .foregroundColor(.yellow)
                            .padding(4)
                            .background(.ultraThinMaterial)
                            .cornerRadius(4)
                        }

                        Spacer()

                        // お気に入りバッジ
                        if image.isFavorite {
                            Image(systemName: "heart.fill")
                                .font(.caption)
                                .foregroundColor(.red)
                                .padding(4)
                                .background(.ultraThinMaterial)
                                .cornerRadius(4)
                        }
                    }
                    .padding(4)
                }

                // 選択チェックマーク
                if isSelectionMode {
                    Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                        .font(.title2)
                        .foregroundColor(isSelected ? .blue : .white)
                        .shadow(radius: 2)
                        .padding(8)
                }
            }
        }
        .aspectRatio(1, contentMode: .fit)
        .overlay(
            RoundedRectangle(cornerRadius: 0)
                .stroke(isSelected ? Color.blue : Color.clear, lineWidth: 3)
        )
    }
}
```

### 6.4 ImageDetailView

```swift
import SwiftUI
import Kingfisher

struct ImageDetailView: View {
    let imageId: String
    let searchParams: ImageSearchParams

    @State private var viewModel: DetailViewModel
    @State private var showFullImage = false
    @State private var showDeleteConfirm = false

    init(imageId: String, searchParams: ImageSearchParams) {
        self.imageId = imageId
        self.searchParams = searchParams
        _viewModel = State(wrappedValue: DetailViewModel(imageId: imageId, searchParams: searchParams))
    }

    var body: some View {
        ScrollView {
            if let image = viewModel.image {
                VStack(spacing: 16) {
                    // メイン画像
                    imageSection(image)

                    // 評価・アクションセクション
                    ratingSection(image)

                    // タグセクション
                    TagEditorView(
                        tags: image.userTags,
                        onAddTag: { tag in
                            viewModel.addTag(tag)
                        },
                        onRemoveTag: { tag in
                            viewModel.removeTag(tag)
                        }
                    )

                    // メモセクション
                    MemoEditorView(
                        memo: image.userMemo ?? "",
                        onSave: { memo in
                            viewModel.updateMemo(memo)
                        }
                    )

                    // メタデータセクション
                    MetadataView(image: image)
                }
                .padding()
            } else if viewModel.isLoading {
                LoadingView()
            } else if let error = viewModel.error {
                ErrorView(error: error, onRetry: { viewModel.load() })
            }
        }
        .navigationTitle(viewModel.image?.originalFilename ?? "詳細")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Menu {
                    Button {
                        viewModel.downloadImage()
                    } label: {
                        Label("ダウンロード", systemImage: "arrow.down.circle")
                    }

                    Button(role: .destructive) {
                        showDeleteConfirm = true
                    } label: {
                        Label("削除", systemImage: "trash")
                    }
                } label: {
                    Image(systemName: "ellipsis.circle")
                }
            }
        }
        .fullScreenCover(isPresented: $showFullImage) {
            if let image = viewModel.image {
                FullImageView(imagePath: image.storagePath)
            }
        }
        .alert("画像を削除しますか？", isPresented: $showDeleteConfirm) {
            Button("キャンセル", role: .cancel) {}
            Button("削除", role: .destructive) {
                viewModel.deleteImage()
            }
        }
        .task {
            await viewModel.load()
        }
        // キーボードショートカット（iPadで有効）
        .keyboardShortcut("f", modifiers: []) {
            viewModel.toggleFavorite()
        }
    }

    @ViewBuilder
    private func imageSection(_ image: Image) -> some View {
        let imageURL = URL(string: "\(AppConfig.apiBaseURL)/storage/\(image.storagePath)")

        KFImage(imageURL)
            .resizable()
            .aspectRatio(contentMode: .fit)
            .frame(maxHeight: 400)
            .onTapGesture {
                showFullImage = true
            }
    }

    @ViewBuilder
    private func ratingSection(_ image: Image) -> some View {
        VStack(spacing: 12) {
            // 星評価
            StarRatingView(
                rating: image.rating,
                onRatingChange: { rating in
                    viewModel.setRating(rating)
                }
            )

            // アクションボタン
            HStack(spacing: 24) {
                // お気に入り
                Button {
                    viewModel.toggleFavorite()
                } label: {
                    VStack {
                        Image(systemName: image.isFavorite ? "heart.fill" : "heart")
                            .font(.title2)
                            .foregroundColor(image.isFavorite ? .red : .gray)
                        Text("お気に入り")
                            .font(.caption)
                    }
                }

                // 改善フラグ
                Button {
                    viewModel.toggleNeedsImprovement()
                } label: {
                    VStack {
                        Image(systemName: image.needsImprovement ? "exclamationmark.triangle.fill" : "exclamationmark.triangle")
                            .font(.title2)
                            .foregroundColor(image.needsImprovement ? .orange : .gray)
                        Text("改善")
                            .font(.caption)
                    }
                }
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(12)
    }
}
```

### 6.5 StarRatingView

```swift
import SwiftUI

struct StarRatingView: View {
    let rating: Int
    let onRatingChange: ((Int) -> Void)?

    private let labels = [
        0: "未評価",
        1: "削除候補",
        2: "とりあえず残す",
        3: "普通",
        4: "良い",
        5: "最高"
    ]

    init(rating: Int, onRatingChange: ((Int) -> Void)? = nil) {
        self.rating = rating
        self.onRatingChange = onRatingChange
    }

    var body: some View {
        VStack(spacing: 4) {
            HStack(spacing: 8) {
                ForEach(1...5, id: \.self) { index in
                    Image(systemName: index <= rating ? "star.fill" : "star")
                        .font(.title)
                        .foregroundColor(index <= rating ? .yellow : .gray)
                        .onTapGesture {
                            if let onRatingChange {
                                // 同じ評価をタップで0に
                                let newRating = index == rating ? 0 : index
                                onRatingChange(newRating)
                            }
                        }
                }
            }

            if let label = labels[rating] {
                Text(label)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
    }
}
```

### 6.6 StatsView

```swift
import SwiftUI
import Charts

struct StatsView: View {
    @State private var viewModel = StatsViewModel()

    var body: some View {
        NavigationStack {
            ScrollView {
                if viewModel.isLoading && viewModel.stats == nil {
                    LoadingView()
                } else if let stats = viewModel.stats {
                    VStack(spacing: 20) {
                        // 概要カード
                        OverviewCardsView(overview: stats.overview)

                        // 日別トレンド
                        DailyTrendChart(data: stats.dailyCounts)

                        // グラフセクション
                        LazyVGrid(columns: [
                            GridItem(.flexible()),
                            GridItem(.flexible())
                        ], spacing: 16) {
                            // モデルタイプ分布
                            PieChartView(
                                title: "Model Type",
                                data: stats.byModelType
                            )

                            // ソース分布
                            PieChartView(
                                title: "Source Tool",
                                data: stats.bySourceTool
                            )
                        }

                        // 評価分布
                        RatingDistributionChart(data: stats.byRating)

                        // トップモデル
                        HorizontalBarChart(
                            title: "Top Models",
                            data: stats.byModelName.prefix(10).map { $0 }
                        )

                        // トップサンプラー
                        HorizontalBarChart(
                            title: "Top Samplers",
                            data: stats.bySampler.prefix(10).map { $0 }
                        )

                        // トップLoRA
                        HorizontalBarChart(
                            title: "Top LoRAs",
                            data: stats.byLora.prefix(10).map { $0 }
                        )
                    }
                    .padding()
                }
            }
            .navigationTitle("統計")
            .refreshable {
                await viewModel.load()
            }
        }
        .task {
            await viewModel.load()
        }
    }
}

struct OverviewCardsView: View {
    let overview: StatsOverview

    var body: some View {
        LazyVGrid(columns: [
            GridItem(.flexible()),
            GridItem(.flexible())
        ], spacing: 12) {
            StatCard(
                title: "総画像数",
                value: "\(overview.totalImages)",
                icon: "photo.stack",
                color: .blue
            )

            StatCard(
                title: "お気に入り",
                value: "\(overview.totalFavorites)",
                icon: "heart.fill",
                color: .red
            )

            StatCard(
                title: "評価済み",
                value: "\(overview.totalRated)",
                icon: "star.fill",
                color: .yellow
            )

            StatCard(
                title: "平均評価",
                value: overview.avgRating.map { String(format: "★%.1f", $0) } ?? "-",
                icon: "chart.bar.fill",
                color: .green
            )
        }
    }
}

struct StatCard: View {
    let title: String
    let value: String
    let icon: String
    let color: Color

    var body: some View {
        HStack {
            Image(systemName: icon)
                .font(.title2)
                .foregroundColor(color)
                .frame(width: 40)

            VStack(alignment: .leading) {
                Text(value)
                    .font(.title2)
                    .fontWeight(.bold)
                Text(title)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Spacer()
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(12)
    }
}

struct DailyTrendChart: View {
    let data: [TimeSeriesItem]

    var body: some View {
        VStack(alignment: .leading) {
            Text("Daily Image Count")
                .font(.headline)

            Chart(data) { item in
                LineMark(
                    x: .value("Date", item.date),
                    y: .value("Count", item.count)
                )
                .foregroundStyle(.blue)
            }
            .frame(height: 200)
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(12)
    }
}
```

---

## 7. 画面遷移

### 7.1 ナビゲーション構造

```
┌─────────────────────────────────────────────────────────┐
│                       TabView                            │
├─────────────┬─────────────┬─────────────┬───────────────┤
│   Gallery   │    Stats    │   Folders   │   Settings    │
│    (Tab)    │    (Tab)    │    (Tab)    │    (Tab)      │
└─────────────┴─────────────┴─────────────┴───────────────┘
       │             │             │
       ▼             │             ▼
┌──────────────┐     │      ┌──────────────┐
│  ImageDetail │     │      │ FolderEditor │
└──────────────┘     │      └──────────────┘
       │             │
       ▼             │
┌──────────────┐     │
│  Slideshow   │     │
│(FullScreen)  │     │
└──────────────┘     │
                     │
                     ▼
              ┌──────────────┐
              │   Analysis   │
              │   (Sheet)    │
              └──────────────┘
```

### 7.2 メインタブビュー

```swift
import SwiftUI

struct MainTabView: View {
    @State private var selectedTab = 0
    @StateObject private var authService = AuthService.shared

    var body: some View {
        if authService.isAuthenticated {
            TabView(selection: $selectedTab) {
                GalleryView()
                    .tabItem {
                        Label("ギャラリー", systemImage: "photo.stack")
                    }
                    .tag(0)

                StatsView()
                    .tabItem {
                        Label("統計", systemImage: "chart.bar")
                    }
                    .tag(1)

                SmartFoldersView()
                    .tabItem {
                        Label("フォルダ", systemImage: "folder")
                    }
                    .tag(2)

                SettingsView()
                    .tabItem {
                        Label("設定", systemImage: "gear")
                    }
                    .tag(3)
            }
        } else {
            LoginView()
        }
    }
}
```

---

## 8. 設定・環境

### 8.1 アプリ設定

```swift
import Foundation

enum AppConfig {
    static var apiBaseURL: String {
        #if DEBUG
        return "http://localhost:8000/api"
        #else
        return UserDefaults.standard.string(forKey: "apiBaseURL") ?? "https://promptbox.example.com/api"
        #endif
    }

    static let defaultPerPage = 24
    static let maxBatchSize = 500

    static let supportedImageExtensions = ["png", "jpg", "jpeg", "webp"]
}
```

### 8.2 設定画面

```swift
import SwiftUI

struct SettingsView: View {
    @AppStorage("apiBaseURL") private var apiBaseURL = ""
    @AppStorage("gridSize") private var gridSize = GridSize.medium.rawValue
    @AppStorage("perPage") private var perPage = AppConfig.defaultPerPage

    @StateObject private var authService = AuthService.shared

    var body: some View {
        NavigationStack {
            Form {
                Section("サーバー設定") {
                    TextField("API URL", text: $apiBaseURL)
                        .autocapitalization(.none)
                        .keyboardType(.URL)
                }

                Section("表示設定") {
                    Picker("グリッドサイズ", selection: $gridSize) {
                        Text("小").tag(GridSize.small.rawValue)
                        Text("中").tag(GridSize.medium.rawValue)
                        Text("大").tag(GridSize.large.rawValue)
                    }

                    Picker("1ページの表示数", selection: $perPage) {
                        Text("24").tag(24)
                        Text("48").tag(48)
                        Text("72").tag(72)
                        Text("96").tag(96)
                    }
                }

                Section("アカウント") {
                    if let user = authService.currentUser {
                        HStack {
                            Text("ユーザー名")
                            Spacer()
                            Text(user.username)
                                .foregroundColor(.secondary)
                        }
                    }

                    Button("ログアウト", role: .destructive) {
                        Task {
                            try? await authService.logout()
                        }
                    }
                }

                Section("アプリ情報") {
                    HStack {
                        Text("バージョン")
                        Spacer()
                        Text(Bundle.main.appVersion)
                            .foregroundColor(.secondary)
                    }
                }
            }
            .navigationTitle("設定")
        }
    }
}

extension Bundle {
    var appVersion: String {
        "\(infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0") (\(infoDictionary?["CFBundleVersion"] as? String ?? "1"))"
    }
}
```

---

## 9. iPad対応

### 9.1 Split View対応

```swift
import SwiftUI

struct iPadGalleryView: View {
    @State private var viewModel = GalleryViewModel()
    @State private var selectedImage: ImageListItem?
    @State private var columnVisibility = NavigationSplitViewVisibility.all

    var body: some View {
        NavigationSplitView(columnVisibility: $columnVisibility) {
            // サイドバー: フォルダ・フィルター
            List {
                Section("スマートフォルダ") {
                    // フォルダリスト
                }

                Section("フィルター") {
                    // クイックフィルター
                }
            }
            .navigationTitle("PromptBox")
        } content: {
            // コンテンツ: 画像グリッド
            ImageGridView(
                images: viewModel.images,
                selectedIds: $viewModel.selectedIds,
                isSelectionMode: viewModel.isSelectionMode,
                gridSize: viewModel.gridSize,
                onImageTap: { image in
                    selectedImage = image
                },
                onLoadMore: {
                    viewModel.loadMore()
                }
            )
            .navigationTitle("ギャラリー")
        } detail: {
            // 詳細: 選択画像の詳細
            if let image = selectedImage {
                ImageDetailView(
                    imageId: image.id,
                    searchParams: viewModel.searchParams
                )
            } else {
                ContentUnavailableView(
                    "画像を選択",
                    systemImage: "photo",
                    description: Text("左のリストから画像を選択してください")
                )
            }
        }
    }
}
```

### 9.2 キーボードショートカット

```swift
extension View {
    func galleryKeyboardShortcuts(
        onSearch: @escaping () -> Void,
        onSelectAll: @escaping () -> Void,
        onDelete: @escaping () -> Void
    ) -> some View {
        self
            .keyboardShortcut("f", modifiers: .command) { onSearch() }
            .keyboardShortcut("a", modifiers: .command) { onSelectAll() }
            .keyboardShortcut(.delete, modifiers: .command) { onDelete() }
    }

    func detailKeyboardShortcuts(
        onRating: @escaping (Int) -> Void,
        onFavorite: @escaping () -> Void,
        onPrevious: @escaping () -> Void,
        onNext: @escaping () -> Void
    ) -> some View {
        self
            .keyboardShortcut("1", modifiers: []) { onRating(1) }
            .keyboardShortcut("2", modifiers: []) { onRating(2) }
            .keyboardShortcut("3", modifiers: []) { onRating(3) }
            .keyboardShortcut("4", modifiers: []) { onRating(4) }
            .keyboardShortcut("5", modifiers: []) { onRating(5) }
            .keyboardShortcut("0", modifiers: []) { onRating(0) }
            .keyboardShortcut("f", modifiers: []) { onFavorite() }
            .keyboardShortcut(.leftArrow, modifiers: []) { onPrevious() }
            .keyboardShortcut(.rightArrow, modifiers: []) { onNext() }
    }
}
```

---

## 10. オフライン対応（将来検討）

### 10.1 キャッシュ戦略

```swift
import SwiftData

@Model
class CachedImage {
    @Attribute(.unique) var id: String
    var thumbnailData: Data?
    var metadata: Data?  // JSON encoded
    var cachedAt: Date

    init(id: String, thumbnailData: Data?, metadata: Data?) {
        self.id = id
        self.thumbnailData = thumbnailData
        self.metadata = metadata
        self.cachedAt = Date()
    }
}

class ImageCacheService {
    private let modelContainer: ModelContainer

    init() throws {
        self.modelContainer = try ModelContainer(for: CachedImage.self)
    }

    @MainActor
    func cache(image: ImageListItem, thumbnailData: Data) async throws {
        let context = modelContainer.mainContext
        let cached = CachedImage(
            id: image.id,
            thumbnailData: thumbnailData,
            metadata: try JSONEncoder().encode(image)
        )
        context.insert(cached)
        try context.save()
    }

    @MainActor
    func getCached(id: String) async throws -> CachedImage? {
        let context = modelContainer.mainContext
        let descriptor = FetchDescriptor<CachedImage>(
            predicate: #Predicate { $0.id == id }
        )
        return try context.fetch(descriptor).first
    }
}
```

---

## 11. テスト計画

### 11.1 ユニットテスト

- **モデルテスト**: JSON デコード/エンコードの正確性
- **サービステスト**: API呼び出しのモック
- **ViewModelテスト**: 状態遷移の検証

### 11.2 UIテスト

- **画面遷移テスト**: 各画面間の遷移
- **操作テスト**: タップ、スワイプ、長押し
- **レスポンシブテスト**: iPhone/iPad各サイズ

### 11.3 統合テスト

- **認証フロー**: ログイン→操作→ログアウト
- **CRUD操作**: 画像の評価、タグ、削除
- **一括操作**: 複数選択→一括更新

---

## 12. リリース計画

### Phase 1: 基本機能（MVP）✅
- [x] 設計書作成
- [x] プロジェクトセットアップ
- [x] 認証機能
- [x] 画像一覧表示
- [x] 画像詳細表示
- [x] 基本的な評価・お気に入り機能

### Phase 2: 拡張機能 ✅
- [x] 検索・フィルタリング
- [x] タグ管理
- [x] 一括操作
- [x] スライドショー

### Phase 3: 高度な機能 ✅
- [x] 統計表示
- [x] スマートフォルダ
- [x] ゴミ箱機能
- [x] 画像ナビゲーション（前後移動）

### Phase 4: iPad対応 ✅
- [x] Split View（3カラム表示）
- [x] キーボードショートカット

### Phase 5: 最適化・リリース
- [ ] パフォーマンス最適化
- [ ] オフラインキャッシュ
- [ ] 画像ダウンロード（フォトライブラリ保存）
- [ ] App Store提出
- [ ] TestFlight配布

---

## 付録

### A. 参照ドキュメント

- [SwiftUI Documentation](https://developer.apple.com/documentation/swiftui)
- [Swift Charts](https://developer.apple.com/documentation/charts)
- [Kingfisher](https://github.com/onevcat/Kingfisher)
- [PromptBox API設計](./04_api_design.md)

### B. 用語集

| 用語 | 説明 |
|------|------|
| PromptBox | 画像管理Webアプリケーション |
| SwiftUI | AppleのUI構築フレームワーク |
| MVVM | Model-View-ViewModel アーキテクチャ |
| Kingfisher | Swift用画像読み込み・キャッシュライブラリ |
