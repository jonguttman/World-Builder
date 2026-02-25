import SwiftUI

struct LevelSelectView: View {
    let levels: [(id: Int, name: String, pack: Pack)] = [
        (1, "First Breath", .free),
        (2, "Shallow Seas", .free),
        (3, "Fragile Balance", .free),
        (4, "High Gravity Hell", .core),
        (5, "Toxic Skies", .core),
        (6, "Rogue Moon", .core),
        (7, "Crimson Star", .core),
        (8, "Desert Bloom", .core),
        (9, "Frozen Heart", .core),
        (10, "The Long Night", .core),
    ]

    var body: some View {
        NavigationStack {
            List {
                Section("Training") {
                    ForEach(levels.filter { $0.pack == .free }, id: \.id) { level in
                        NavigationLink {
                            PlanetView(levelId: level.id)
                        } label: {
                            LevelRow(number: level.id, name: level.name, locked: false)
                        }
                    }
                }
                Section("Core Challenge Pack") {
                    ForEach(levels.filter { $0.pack == .core }, id: \.id) { level in
                        LevelRow(number: level.id, name: level.name, locked: true)
                    }
                }
            }
            .navigationTitle("Planet Architect")
        }
    }
}

struct LevelRow: View {
    let number: Int
    let name: String
    let locked: Bool

    var body: some View {
        HStack {
            Text("\(number)")
                .font(.headline)
                .frame(width: 30)
            Text(name)
            Spacer()
            if locked {
                Image(systemName: "lock.fill")
                    .foregroundColor(.secondary)
            }
        }
    }
}
