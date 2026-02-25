import SwiftUI

struct NewDiscoveriesSheet: View {
    let discoveryIds: [String]
    @Environment(CodexStore.self) private var store
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            VStack(spacing: 16) {
                Text("New Discoveries!")
                    .font(.title.bold())
                    .padding(.top)

                List {
                    ForEach(discoveryIds, id: \.self) { id in
                        if let entry = store.entry(for: id) {
                            HStack(spacing: 12) {
                                Image(systemName: entry.iconAssetId)
                                    .font(.title3)
                                    .frame(width: 30)

                                VStack(alignment: .leading, spacing: 2) {
                                    Text(entry.name)
                                        .font(.headline)
                                    Text(entry.flavorText)
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                        .lineLimit(2)
                                }
                            }
                        }
                    }
                }

                Button("View in Codex") {
                    store.markDiscoveriesViewed()
                    dismiss()
                }
                .buttonStyle(.borderedProminent)
                .padding(.bottom)
            }
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Dismiss") {
                        store.markDiscoveriesViewed()
                        dismiss()
                    }
                }
            }
        }
    }
}
