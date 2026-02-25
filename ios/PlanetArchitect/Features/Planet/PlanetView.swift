import SwiftUI

struct PlanetView: View {
    let levelId: String
    @Environment(\.dismiss) private var dismiss
    @Environment(CodexStore.self) private var codexStore

    @State private var viewModel = SimulationViewModel()
    @State private var selectedTile: TileInfo?
    @State private var showTileInspector = false
    @State private var showBriefing = true
    @State private var showTutorial = true
    @State private var tutorialStep = 0

    private var tutorialSteps: [TutorialStep] {
        switch levelId {
        case "level_01_first_breath": return LevelTutorials.level1
        case "level_02_shallow_seas": return LevelTutorials.level2
        default: return LevelTutorials.level1
        }
    }

    var body: some View {
        ZStack {
            simulationView

            if showBriefing, let config = viewModel.levelConfig {
                LevelBriefingView(config: config) {
                    showBriefing = false
                }
                .background(.ultraThinMaterial)
            }

            if !showBriefing {
                TutorialOverlay(
                    steps: tutorialSteps,
                    currentStepIndex: $tutorialStep,
                    isVisible: $showTutorial
                )
            }

            if case .won = viewModel.levelStatus {
                LevelCompleteView(
                    won: true, failReason: nil,
                    steps: viewModel.currentStep,
                    biodiversity: viewModel.biodiversity,
                    newDiscoveryIds: viewModel.newCodexUnlocks,
                    onRestart: { restart() },
                    onExit: { dismiss() }
                )
                .background(.ultraThinMaterial)
                .onAppear { codexStore.syncUnlocked(from: viewModel.engine?.codexUnlockedJSON ?? "[]") }
            }

            if case .failed(let reason) = viewModel.levelStatus {
                LevelCompleteView(
                    won: false, failReason: reason,
                    steps: viewModel.currentStep,
                    biodiversity: viewModel.biodiversity,
                    newDiscoveryIds: viewModel.newCodexUnlocks,
                    onRestart: { restart() },
                    onExit: { dismiss() }
                )
                .background(.ultraThinMaterial)
                .onAppear { codexStore.syncUnlocked(from: viewModel.engine?.codexUnlockedJSON ?? "[]") }
            }
        }
        .navigationTitle(viewModel.levelConfig?.name ?? "Level")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            viewModel.startLevel(levelId: levelId)
            if codexStore.allEntries.isEmpty, let engine = viewModel.engine {
                codexStore.loadEntries(from: engine.codexAllEntriesJSON)
            }
        }
    }

    private var simulationView: some View {
        VStack(spacing: 0) {
            // Status bar
            HStack {
                Label("\(viewModel.currentStep) yr", systemImage: "clock")
                Spacer()
                Label(String(format: "%.0f biomass", viewModel.totalBiomass), systemImage: "microbe")
                Spacer()
                Label("\(viewModel.biodiversity) species", systemImage: "leaf")
            }
            .font(.caption)
            .padding(.horizontal)
            .padding(.vertical, 4)

            // Objective progress
            if viewModel.requiredSteps > 0 {
                VStack(spacing: 2) {
                    ProgressView(value: viewModel.objectiveProgress)
                        .tint(viewModel.objectiveProgress > 0 ? .green : .gray)
                    Text("Objective: \(Int(viewModel.objectiveProgress * 100))%")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
                .padding(.horizontal)
            }

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

            // Intervention tray
            if !viewModel.allowedInterventions.isEmpty {
                VStack(spacing: 4) {
                    HStack {
                        Label(String(format: "Energy: %.0f", viewModel.energyRemaining), systemImage: "bolt.fill")
                            .font(.caption)
                            .foregroundStyle(.orange)
                        Spacer()
                    }
                    .padding(.horizontal)

                    InterventionTray(
                        allowedInterventions: viewModel.allowedInterventions,
                        energyRemaining: viewModel.energyRemaining,
                        onApply: { kind, magnitude in
                            viewModel.applyIntervention(kind: kind, magnitude: magnitude)
                        }
                    )
                }
            }

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
        .sheet(isPresented: $showTileInspector) {
            if let tile = selectedTile {
                TileInspectorView(tile: tile)
                    .presentationDetents([.height(200)])
            }
        }
    }

    private func restart() {
        viewModel.startLevel(levelId: levelId)
        showBriefing = true
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
