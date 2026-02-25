import SwiftUI

enum OverlayMode: String, CaseIterable, Identifiable {
    case temperature = "Temp"
    case nutrients = "Nutrients"
    case moisture = "Moisture"
    case population = "Population"

    var id: String { rawValue }
}

@MainActor
@Observable
final class SimulationViewModel {
    // Engine
    private(set) var engine: SimulationEngine?

    // State
    private(set) var currentStep: UInt64 = 0
    private(set) var biodiversity: UInt32 = 0
    var isPaused: Bool = true
    var timeSpeed: TimeSpeed = .adapt
    var overlayMode: OverlayMode = .temperature

    // Grid data (updated from snapshot)
    private(set) var temperatures: [Float] = []
    private(set) var nutrients: [Float] = []
    private(set) var moisture: [Float] = []
    private(set) var populationDensity: [Float] = []
    private(set) var oceanMask: [UInt8] = []

    var gridWidth: Int { engine?.gridWidth ?? 64 }
    var gridHeight: Int { engine?.gridHeight ?? 32 }

    // Simulation loop
    private var simulationTask: Task<Void, Never>?

    var currentOverlayData: [Float] {
        switch overlayMode {
        case .temperature: return temperatures
        case .nutrients: return nutrients
        case .moisture: return moisture
        case .population: return populationDensity
        }
    }

    // MARK: - Lifecycle

    func startLevel(seed: UInt64, paramsJSON: String? = nil) {
        engine = SimulationEngine(seed: seed, paramsJSON: paramsJSON)

        // Warm up nutrients
        engine?.step(500)

        // Seed initial microbe for Level 1
        let microbeJSON = """
        {"id":0,"name":"Thermophile","traits":{"temp_optimal":5.0,"temp_range":60.0,"o2_need":0.0,"toxin_resistance":0.3,"trophic_level":"Producer","reproduction_rate":0.04,"dispersal":0.3,"mutation_rate":0.005}}
        """
        engine?.addSpecies(json: microbeJSON, initialPopulation: 100.0)
        refreshSnapshot()
    }

    // MARK: - Simulation Control

    func togglePause() {
        isPaused.toggle()
        if isPaused {
            simulationTask?.cancel()
            simulationTask = nil
        } else {
            startSimulationLoop()
        }
    }

    private func startSimulationLoop() {
        simulationTask = Task { [weak self] in
            while !Task.isCancelled {
                guard let self else { break }
                self.tick()
                try? await Task.sleep(for: .milliseconds(33))
            }
        }
    }

    private func tick() {
        guard let engine else { return }
        engine.step(timeSpeed.stepsPerBatch)
        refreshSnapshot()
    }

    func refreshSnapshot() {
        guard let engine else { return }
        engine.updateSnapshot()
        currentStep = engine.currentStep
        biodiversity = engine.biodiversityCount
        temperatures = engine.temperatures
        nutrients = engine.nutrients
        moisture = engine.moisture
        populationDensity = engine.populationDensity
        oceanMask = engine.oceanMask
    }

    // MARK: - Tile Inspector

    func tileInfo(x: Int, y: Int) -> TileInfo? {
        let index = y * gridWidth + x
        guard index < temperatures.count else { return nil }
        return TileInfo(
            x: x, y: y,
            temperature: temperatures[index],
            nutrients: nutrients[index],
            moisture: moisture[index],
            population: populationDensity[index],
            isOcean: oceanMask[index] == 1
        )
    }
}

struct TileInfo {
    let x: Int
    let y: Int
    let temperature: Float
    let nutrients: Float
    let moisture: Float
    let population: Float
    let isOcean: Bool
}
