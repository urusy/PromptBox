import Foundation

struct User: Codable, Identifiable, Sendable {
    let id: String
    let username: String
    let createdAt: Date?

    enum CodingKeys: String, CodingKey {
        case id
        case username
        case createdAt = "created_at"
    }
}

struct LoginRequest: Encodable {
    let username: String
    let password: String
}

struct LoginResponse: Decodable {
    let message: String
    let username: String
}

struct MeResponse: Decodable {
    let username: String
}
