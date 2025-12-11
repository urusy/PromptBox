import SwiftUI

struct RatingView: View {
    let rating: Int
    let maxRating: Int
    let onRatingChange: ((Int) -> Void)?

    init(rating: Int, maxRating: Int = 5, onRatingChange: ((Int) -> Void)? = nil) {
        self.rating = rating
        self.maxRating = maxRating
        self.onRatingChange = onRatingChange
    }

    var body: some View {
        HStack(spacing: 4) {
            ForEach(1...maxRating, id: \.self) { star in
                Image(systemName: rating >= star ? "star.fill" : "star")
                    .foregroundStyle(.yellow)
                    .onTapGesture {
                        if let onRatingChange {
                            if rating == star {
                                onRatingChange(0)
                            } else {
                                onRatingChange(star)
                            }
                        }
                    }
            }
        }
    }
}

#Preview {
    VStack(spacing: 20) {
        RatingView(rating: 0)
        RatingView(rating: 3)
        RatingView(rating: 5)
    }
}
