import SwiftUI
import Kingfisher

struct SlideshowView: View {
    let images: [ImageItem]

    @Environment(\.dismiss) private var dismiss
    @State private var currentIndex = 0
    @State private var isPlaying = true
    @State private var showControls = true
    @State private var timer: Timer?

    private let interval: TimeInterval = 5.0

    var body: some View {
        GeometryReader { geometry in
            ZStack {
                Color.black.ignoresSafeArea()

                if !images.isEmpty {
                    TabView(selection: $currentIndex) {
                        ForEach(Array(images.enumerated()), id: \.element.id) { index, image in
                            KFImage(image.imageURL)
                                .resizable()
                                .aspectRatio(contentMode: .fit)
                                .frame(width: geometry.size.width, height: geometry.size.height)
                                .tag(index)
                        }
                    }
                    .tabViewStyle(.page(indexDisplayMode: .never))
                }

                // Controls overlay
                if showControls {
                    VStack {
                        HStack {
                            Spacer()
                            Button {
                                dismiss()
                            } label: {
                                Image(systemName: "xmark.circle.fill")
                                    .font(.title)
                                    .foregroundStyle(.white)
                            }
                            .padding()
                        }

                        Spacer()

                        // Bottom controls
                        HStack(spacing: 40) {
                            Button {
                                previousImage()
                            } label: {
                                Image(systemName: "backward.fill")
                                    .font(.title2)
                            }
                            .disabled(currentIndex == 0)

                            Button {
                                togglePlayback()
                            } label: {
                                Image(systemName: isPlaying ? "pause.fill" : "play.fill")
                                    .font(.title)
                            }

                            Button {
                                nextImage()
                            } label: {
                                Image(systemName: "forward.fill")
                                    .font(.title2)
                            }
                            .disabled(currentIndex >= images.count - 1)
                        }
                        .foregroundStyle(.white)
                        .padding()
                        .background(.ultraThinMaterial, in: Capsule())
                        .padding(.bottom, 40)
                    }
                    .transition(.opacity)
                }

                // Page indicator
                if showControls && images.count > 1 {
                    VStack {
                        Spacer()
                        Text("\(currentIndex + 1) / \(images.count)")
                            .font(.caption)
                            .foregroundStyle(.white)
                            .padding(.horizontal, 12)
                            .padding(.vertical, 6)
                            .background(.black.opacity(0.5), in: Capsule())
                            .padding(.bottom, 100)
                    }
                }
            }
            .onTapGesture {
                withAnimation {
                    showControls.toggle()
                }
            }
        }
        .onAppear {
            startTimer()
        }
        .onDisappear {
            stopTimer()
        }
        .onChange(of: isPlaying) { _, playing in
            if playing {
                startTimer()
            } else {
                stopTimer()
            }
        }
        .statusBarHidden()
    }

    private func startTimer() {
        guard isPlaying else { return }
        stopTimer()
        timer = Timer.scheduledTimer(withTimeInterval: interval, repeats: true) { _ in
            withAnimation {
                if currentIndex < images.count - 1 {
                    currentIndex += 1
                } else {
                    currentIndex = 0
                }
            }
        }
    }

    private func stopTimer() {
        timer?.invalidate()
        timer = nil
    }

    private func togglePlayback() {
        isPlaying.toggle()
    }

    private func previousImage() {
        if currentIndex > 0 {
            withAnimation {
                currentIndex -= 1
            }
        }
    }

    private func nextImage() {
        if currentIndex < images.count - 1 {
            withAnimation {
                currentIndex += 1
            }
        }
    }
}

#Preview {
    SlideshowView(images: [])
}
