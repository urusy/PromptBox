import SwiftUI

struct MainTabView: View {
    @State private var selectedTab = 0

    var body: some View {
        TabView(selection: $selectedTab) {
            GalleryView()
                .tabItem {
                    Label("ギャラリー", systemImage: "photo.on.rectangle")
                }
                .tag(0)

            SmartFoldersView()
                .tabItem {
                    Label("スマートフォルダ", systemImage: "folder.badge.gearshape")
                }
                .tag(1)

            StatsView()
                .tabItem {
                    Label("統計", systemImage: "chart.bar")
                }
                .tag(2)

            TrashView()
                .tabItem {
                    Label("ゴミ箱", systemImage: "trash")
                }
                .tag(3)

            SettingsView()
                .tabItem {
                    Label("設定", systemImage: "gearshape")
                }
                .tag(4)
        }
    }
}

#Preview {
    MainTabView()
}
