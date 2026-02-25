import SwiftUI

@MainActor
@Observable
final class CodexStore {
    private(set) var allEntries: [CodexEntryData] = []
    private(set) var unlockedIds: Set<String> = []
    private(set) var newDiscoveryIds: [String] = []
    var unviewedCount: Int { newDiscoveryIds.count }

    private let unlockedKey = "codex_unlocked_ids"

    init() {
        if let saved = UserDefaults.standard.array(forKey: unlockedKey) as? [String] {
            unlockedIds = Set(saved)
        }
    }

    func loadEntries(from json: String) {
        guard let data = json.data(using: .utf8),
              let entries = try? JSONDecoder().decode([CodexEntryData].self, from: data) else {
            return
        }
        allEntries = entries
    }

    func syncUnlocked(from json: String) {
        guard let data = json.data(using: .utf8),
              let ids = try? JSONDecoder().decode([String].self, from: data) else {
            return
        }
        for id in ids {
            if !unlockedIds.contains(id) {
                unlockedIds.insert(id)
                newDiscoveryIds.append(id)
            }
        }
        save()
    }

    func markDiscoveriesViewed() {
        newDiscoveryIds.removeAll()
    }

    func isUnlocked(_ id: String) -> Bool {
        unlockedIds.contains(id)
    }

    func entry(for id: String) -> CodexEntryData? {
        allEntries.first { $0.id == id }
    }

    func entries(in category: String) -> [CodexEntryData] {
        allEntries.filter { $0.category == category }
    }

    private func save() {
        UserDefaults.standard.set(Array(unlockedIds), forKey: unlockedKey)
    }
}

struct CodexEntryData: Codable, Identifiable {
    let id: String
    let category: String
    let name: String
    let requirementsText: String
    let factsText: String
    let flavorText: String
    let relatedEntryIds: [String]
    let iconAssetId: String

    enum CodingKeys: String, CodingKey {
        case id, category, name
        case requirementsText = "requirements_text"
        case factsText = "facts_text"
        case flavorText = "flavor_text"
        case relatedEntryIds = "related_entry_ids"
        case iconAssetId = "icon_asset_id"
    }
}
