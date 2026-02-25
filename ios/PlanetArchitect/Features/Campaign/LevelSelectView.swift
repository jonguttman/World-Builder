import SwiftUI

struct LevelSelectView: View {
    let levels: [(id: String, number: Int, name: String, pack: Pack)] = [
        ("level_01_first_breath", 1, "First Breath", .free),
        ("level_02_shallow_seas", 2, "Shallow Seas", .free),
        ("level_03_fragile_balance", 3, "Fragile Balance", .free),
        ("level_04_high_gravity_hell", 4, "High Gravity Hell", .core),
        ("level_05_toxic_skies", 5, "Toxic Skies", .core),
        ("level_06_rogue_moon", 6, "Rogue Moon", .core),
        ("level_07_crimson_star", 7, "Crimson Star", .core),
        ("level_08_desert_bloom", 8, "Desert Bloom", .core),
        ("level_09_frozen_heart", 9, "Frozen Heart", .core),
        ("level_10_the_long_night", 10, "The Long Night", .core),
    ]

    var body: some View {
        NavigationStack {
            List {
                Section("Training") {
                    ForEach(levels.filter { $0.pack == .free }, id: \.number) { level in
                        NavigationLink {
                            PlanetView(levelId: level.id)
                        } label: {
                            LevelRow(number: level.number, name: level.name, locked: false)
                        }
                    }
                }
                Section("Core Challenge Pack") {
                    ForEach(levels.filter { $0.pack == .core }, id: \.number) { level in
                        LevelRow(number: level.number, name: level.name, locked: true)
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
