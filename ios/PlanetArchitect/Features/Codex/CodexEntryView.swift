import SwiftUI

struct CodexEntryView: View {
    let entry: CodexEntryData
    @Environment(CodexStore.self) private var store

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                // Header
                HStack {
                    Image(systemName: entry.iconAssetId)
                        .font(.largeTitle)
                    VStack(alignment: .leading) {
                        Text(entry.name)
                            .font(.title.bold())
                        Text(displayCategory(entry.category))
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }
                }

                Divider()

                // Facts
                VStack(alignment: .leading, spacing: 8) {
                    Text("Facts")
                        .font(.headline)
                    Text(entry.factsText)
                        .font(.body)
                }

                // Flavor
                Text(entry.flavorText)
                    .font(.body.italic())
                    .foregroundStyle(.secondary)
                    .padding()
                    .frame(maxWidth: .infinity, alignment: .center)
                    .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 12))

                // Requirements
                VStack(alignment: .leading, spacing: 8) {
                    Text("How to Unlock")
                        .font(.headline)
                    Text(entry.requirementsText)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                // Related entries
                if !entry.relatedEntryIds.isEmpty {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Related")
                            .font(.headline)
                        ForEach(entry.relatedEntryIds, id: \.self) { relatedId in
                            if let related = store.entry(for: relatedId), store.isUnlocked(relatedId) {
                                NavigationLink {
                                    CodexEntryView(entry: related)
                                } label: {
                                    Label(related.name, systemImage: related.iconAssetId)
                                        .font(.subheadline)
                                }
                            } else {
                                Label("???", systemImage: "lock.fill")
                                    .font(.subheadline)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
            }
            .padding()
        }
        .navigationTitle(entry.name)
        .navigationBarTitleDisplayMode(.inline)
    }

    private func displayCategory(_ cat: String) -> String {
        switch cat {
        case "Species": return "Species"
        case "BodyPlan": return "Body Plan"
        case "Biome": return "Biome"
        case "PlanetarySystem": return "Planetary System"
        case "EvolutionaryEvent": return "Evolutionary Event"
        case "FailureMode": return "Failure Mode"
        case "RarePhenomenon": return "Rare Phenomenon"
        case "HistoricWorld": return "Historic World"
        default: return cat
        }
    }
}
