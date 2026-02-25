import SwiftUI

struct PlanetView: View {
    let levelId: Int
    let seed: UInt64
    let paramsJSON: String?

    @State private var viewModel = SimulationViewModel()
    @State private var selectedTile: TileInfo?
    @State private var showTileInspector = false

    init(levelId: Int, seed: UInt64 = 7749, paramsJSON: String? = nil) {
        self.levelId = levelId
        self.seed = seed
        self.paramsJSON = paramsJSON
    }

    var body: some View {
        VStack(spacing: 0) {
            // Status bar
            HStack {
                Label("\(viewModel.currentStep) yr", systemImage: "clock")
                Spacer()
                Label("\(viewModel.biodiversity) species", systemImage: "leaf")
            }
            .font(.caption)
            .padding(.horizontal)
            .padding(.vertical, 4)

            // Grid
            GridRenderer(
                data: viewModel.currentOverlayData,
                oceanMask: viewModel.oceanMask,
                width: viewModel.gridWidth,
                height: viewModel.gridHeight,
                overlayMode: viewModel.overlayMode,
                onTileTap: { x, y in
                    selectedTile = viewModel.tileInfo(x: x, y: y)
                    showTileInspector = true
                }
            )
            .aspectRatio(2.0, contentMode: .fit)
            .clipShape(RoundedRectangle(cornerRadius: 8))
            .padding(.horizontal)

            Spacer()

            // Overlay picker
            Picker("Overlay", selection: $viewModel.overlayMode) {
                ForEach(OverlayMode.allCases) { mode in
                    Text(mode.rawValue).tag(mode)
                }
            }
            .pickerStyle(.segmented)
            .padding(.horizontal)

            // Time controls
            HStack(spacing: 16) {
                Button {
                    viewModel.togglePause()
                } label: {
                    Image(systemName: viewModel.isPaused ? "play.fill" : "pause.fill")
                        .font(.title2)
                        .frame(width: 44, height: 44)
                }

                Picker("Speed", selection: $viewModel.timeSpeed) {
                    Text("1x").tag(TimeSpeed.observe)
                    Text("100x").tag(TimeSpeed.adapt)
                    Text("10K").tag(TimeSpeed.epoch)
                    Text("1M").tag(TimeSpeed.eon)
                }
                .pickerStyle(.segmented)
            }
            .padding()
        }
        .navigationTitle("Level \(levelId)")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            viewModel.startLevel(seed: seed, paramsJSON: paramsJSON)
        }
        .sheet(isPresented: $showTileInspector) {
            if let tile = selectedTile {
                TileInspectorView(tile: tile)
                    .presentationDetents([.height(200)])
            }
        }
    }
}

struct TileInspectorView: View {
    let tile: TileInfo

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Tile (\(tile.x), \(tile.y))")
                .font(.headline)

            HStack {
                Text(tile.isOcean ? "Ocean" : "Land")
                    .foregroundStyle(tile.isOcean ? .blue : .brown)
                Spacer()
            }

            Grid(alignment: .leading) {
                GridRow {
                    Text("Temperature")
                    Text(String(format: "%.1f C", tile.temperature))
                        .foregroundStyle(.secondary)
                }
                GridRow {
                    Text("Nutrients")
                    Text(String(format: "%.3f", tile.nutrients))
                        .foregroundStyle(.secondary)
                }
                GridRow {
                    Text("Moisture")
                    Text(String(format: "%.3f", tile.moisture))
                        .foregroundStyle(.secondary)
                }
                GridRow {
                    Text("Population")
                    Text(String(format: "%.0f", tile.population))
                        .foregroundStyle(.secondary)
                }
            }
            .font(.body)
        }
        .padding()
    }
}
