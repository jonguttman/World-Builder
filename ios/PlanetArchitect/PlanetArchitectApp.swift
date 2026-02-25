import SwiftUI

@main
struct PlanetArchitectApp: App {
    @State private var codexStore = CodexStore()

    var body: some Scene {
        WindowGroup {
            TabView {
                NavigationStack {
                    LevelSelectView()
                }
                .tabItem {
                    Label("Campaign", systemImage: "map")
                }

                NavigationStack {
                    CodexView()
                }
                .tabItem {
                    Label("Codex", systemImage: "book.closed")
                }
                .badge(codexStore.unviewedCount)
            }
            .environment(codexStore)
        }
    }
}
