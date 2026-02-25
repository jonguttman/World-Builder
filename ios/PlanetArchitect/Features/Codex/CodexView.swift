import SwiftUI

struct CodexView: View {
    @Environment(CodexStore.self) private var store
    @State private var searchText = ""
    @State private var selectedCategory: String?

    private let categories = [
        ("Species", "hare"),
        ("BodyPlan", "figure.stand"),
        ("Biome", "mountain.2"),
        ("PlanetarySystem", "globe"),
        ("EvolutionaryEvent", "sparkles"),
        ("FailureMode", "exclamationmark.triangle"),
        ("RarePhenomenon", "star"),
        ("HistoricWorld", "clock"),
    ]

    var body: some View {
        List {
            // Category filter chips
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    categoryChip(nil, label: "All")
                    ForEach(categories, id: \.0) { cat, icon in
                        categoryChip(cat, label: displayName(cat), icon: icon)
                    }
                }
                .padding(.vertical, 4)
            }
            .listRowInsets(EdgeInsets())
            .listRowBackground(Color.clear)

            // Entries grouped by category
            ForEach(filteredCategories, id: \.0) { category, entries in
                Section(displayName(category)) {
                    ForEach(entries) { entry in
                        if store.isUnlocked(entry.id) {
                            NavigationLink {
                                CodexEntryView(entry: entry)
                            } label: {
                                CodexRow(entry: entry, unlocked: true)
                            }
                        } else {
                            CodexRow(entry: entry, unlocked: false)
                        }
                    }
                }
            }
        }
        .searchable(text: $searchText, prompt: "Search entries")
        .navigationTitle("Codex")
    }

    private var filteredCategories: [(String, [CodexEntryData])] {
        let allEntries = store.allEntries

        let filtered: [CodexEntryData]
        if searchText.isEmpty {
            if let cat = selectedCategory {
                filtered = allEntries.filter { $0.category == cat }
            } else {
                filtered = allEntries
            }
        } else {
            let search = searchText.lowercased()
            filtered = allEntries.filter { entry in
                let matchesSearch = entry.name.lowercased().contains(search)
                let matchesCat = selectedCategory.map { $0 == entry.category } ?? true
                return matchesSearch && matchesCat
            }
        }

        // Group by category
        var grouped: [String: [CodexEntryData]] = [:]
        for entry in filtered {
            grouped[entry.category, default: []].append(entry)
        }

        let order = categories.map(\.0)
        return order.compactMap { cat in
            guard let entries = grouped[cat], !entries.isEmpty else { return nil }
            return (cat, entries)
        }
    }

    private func categoryChip(_ category: String?, label: String, icon: String = "square.grid.2x2") -> some View {
        Button {
            selectedCategory = category
        } label: {
            Label(label, systemImage: icon)
                .font(.caption)
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(
                    selectedCategory == category ? Color.accentColor.opacity(0.2) : Color.secondary.opacity(0.1),
                    in: Capsule()
                )
        }
        .buttonStyle(.plain)
    }

    private func displayName(_ category: String) -> String {
        switch category {
        case "Species": return "Species"
        case "BodyPlan": return "Body Plans"
        case "Biome": return "Biomes"
        case "PlanetarySystem": return "Planetary Systems"
        case "EvolutionaryEvent": return "Events"
        case "FailureMode": return "Failures"
        case "RarePhenomenon": return "Rare"
        case "HistoricWorld": return "Historic"
        default: return category
        }
    }
}

private struct CodexRow: View {
    let entry: CodexEntryData
    let unlocked: Bool

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: unlocked ? entry.iconAssetId : "lock.fill")
                .font(.title3)
                .frame(width: 30)
                .foregroundStyle(unlocked ? .primary : .secondary)

            VStack(alignment: .leading, spacing: 2) {
                Text(unlocked ? entry.name : "???")
                    .font(.body)
                Text(unlocked ? String(entry.factsText.prefix(60)) + "..." : "Undiscovered")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
        .opacity(unlocked ? 1.0 : 0.5)
    }
}
