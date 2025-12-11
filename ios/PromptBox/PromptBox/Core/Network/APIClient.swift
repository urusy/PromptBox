import Foundation

enum APIError: Error, LocalizedError, Sendable {
    case invalidURL
    case invalidResponse
    case httpError(statusCode: Int, message: String?)
    case decodingError(String)
    case networkError(String)
    case unauthorized

    var errorDescription: String? {
        switch self {
        case .invalidURL:
            return "Invalid URL"
        case .invalidResponse:
            return "Invalid response from server"
        case .httpError(let statusCode, let message):
            return message ?? "HTTP Error: \(statusCode)"
        case .decodingError(let message):
            return "Decoding error: \(message)"
        case .networkError(let message):
            return "Network error: \(message)"
        case .unauthorized:
            return "Unauthorized"
        }
    }
}

final class APIClient: Sendable {
    static let shared = APIClient()

    private let session: URLSession
    private let decoder: JSONDecoder
    private let encoder: JSONEncoder

    private init() {
        let config = URLSessionConfiguration.default
        config.httpCookieAcceptPolicy = .always
        config.httpShouldSetCookies = true
        config.httpCookieStorage = .shared

        self.session = URLSession(configuration: config)

        let decoder = JSONDecoder()
        // FastAPIはマイクロ秒を含むISO8601形式を使用するため、カスタムフォーマッターを使用
        let dateFormatter = ISO8601DateFormatter()
        dateFormatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        decoder.dateDecodingStrategy = .custom { decoder in
            let container = try decoder.singleValueContainer()
            let dateString = try container.decode(String.self)

            // マイクロ秒付きISO8601を試す
            if let date = dateFormatter.date(from: dateString) {
                return date
            }

            // マイクロ秒なしを試す
            let basicFormatter = ISO8601DateFormatter()
            basicFormatter.formatOptions = [.withInternetDateTime]
            if let date = basicFormatter.date(from: dateString) {
                return date
            }

            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Cannot decode date: \(dateString)"
            )
        }
        self.decoder = decoder

        let encoder = JSONEncoder()
        self.encoder = encoder
    }

    func request<T: Decodable>(
        endpoint: APIEndpoint,
        body: (any Encodable)? = nil,
        queryItems: [URLQueryItem]? = nil
    ) async throws -> T {
        guard var url = endpoint.url(baseURL: AppConfig.apiBaseURL) else {
            throw APIError.invalidURL
        }

        if let queryItems, !queryItems.isEmpty {
            var components = URLComponents(url: url, resolvingAgainstBaseURL: false)
            components?.queryItems = queryItems
            if let newURL = components?.url {
                url = newURL
            }
        }

        var request = URLRequest(url: url)
        request.httpMethod = endpoint.method
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("application/json", forHTTPHeaderField: "Accept")

        if let body {
            request.httpBody = try encoder.encode(AnyEncodable(body))
        }

        print("[APIClient] Request: \(request.httpMethod ?? "GET") \(request.url?.absoluteString ?? "nil")")

        let (data, response) = try await session.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }

        print("[APIClient] Response: \(httpResponse.statusCode)")

        if httpResponse.statusCode == 401 {
            throw APIError.unauthorized
        }

        guard 200..<300 ~= httpResponse.statusCode else {
            let message = try? decoder.decode(ErrorResponse.self, from: data).detail
            print("[APIClient] Error response: \(String(data: data, encoding: .utf8) ?? "N/A")")
            throw APIError.httpError(statusCode: httpResponse.statusCode, message: message)
        }

        do {
            return try decoder.decode(T.self, from: data)
        } catch let decodingError as DecodingError {
            let detailedMessage: String
            switch decodingError {
            case .keyNotFound(let key, let context):
                detailedMessage = "Missing key '\(key.stringValue)' at \(context.codingPath.map { $0.stringValue }.joined(separator: "."))"
            case .typeMismatch(let type, let context):
                detailedMessage = "Type mismatch for \(type) at \(context.codingPath.map { $0.stringValue }.joined(separator: ".")): \(context.debugDescription)"
            case .valueNotFound(let type, let context):
                detailedMessage = "Value not found for \(type) at \(context.codingPath.map { $0.stringValue }.joined(separator: "."))"
            case .dataCorrupted(let context):
                detailedMessage = "Data corrupted at \(context.codingPath.map { $0.stringValue }.joined(separator: ".")): \(context.debugDescription)"
            @unknown default:
                detailedMessage = decodingError.localizedDescription
            }
            print("[APIClient] Decoding error: \(detailedMessage)")
            print("[APIClient] Raw response: \(String(data: data, encoding: .utf8) ?? "N/A")")
            throw APIError.decodingError(detailedMessage)
        } catch {
            throw APIError.decodingError(error.localizedDescription)
        }
    }

    func requestVoid(
        endpoint: APIEndpoint,
        body: (any Encodable)? = nil
    ) async throws {
        guard let url = endpoint.url(baseURL: AppConfig.apiBaseURL) else {
            throw APIError.invalidURL
        }

        var request = URLRequest(url: url)
        request.httpMethod = endpoint.method
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        if let body {
            request.httpBody = try encoder.encode(AnyEncodable(body))
        }

        let (data, response) = try await session.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }

        if httpResponse.statusCode == 401 {
            throw APIError.unauthorized
        }

        guard 200..<300 ~= httpResponse.statusCode else {
            let message = try? decoder.decode(ErrorResponse.self, from: data).detail
            throw APIError.httpError(statusCode: httpResponse.statusCode, message: message)
        }
    }
}

private struct ErrorResponse: Decodable {
    let detail: String?
}

private struct AnyEncodable: Encodable {
    private let encode: (Encoder) throws -> Void

    init(_ wrapped: any Encodable) {
        self.encode = wrapped.encode
    }

    func encode(to encoder: Encoder) throws {
        try encode(encoder)
    }
}
