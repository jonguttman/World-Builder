import SwiftUI

enum OverlayMode: String, CaseIterable, Identifiable {
    case temperature = "Temp"
    case nutrients = "Nutrients"
    case moisture = "Moisture"
    case population = "Population"

    var id: String { rawValue }
}

enum LevelStatus: Equatable {
    case playing
    case won
    case failed(reason: String)
}

@MainActor
@Observable
final class SimulationViewModel {
    // Engine
    private(set) var engine: SimulationEngine?

    // Level config
    private(set) var levelConfig: LevelConfig?
    private var objectiveJSON: String?

    // Objective tracking
    private(set) var levelStatus: LevelStatus = .playing
    private(set) var sustainedSteps: UInt64 = 0
    private(set) var requiredSteps: UInt64 = 0
    private(set) var totalBiomass: Double = 0

    // Energy budget
    private(set) var energyRemaining: Float = 0
    var allowedInterventions: [String] { levelConfig?.allowedInterventions ?? [] }

    // Codex
    private(set) var newCodexUnlocks: [String] = []

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

    var objectiveProgress: Double {
        guard requiredSteps > 0 else { return 0 }
        return Double(sustainedSteps) / Double(requiredSteps)
    }

    // MARK: - Lifecycle

    func startLevel(levelId: String) {
        guard let config = LevelLoader.load(levelId: levelId) else { return }
        self.levelConfig = config
        self.objectiveJSON = LevelLoader.objectiveJSON(from: config)
        self.energyRemaining = config.energyBudget
        self.requiredSteps = config.objective.requiredDurationSteps
        self.levelStatus = .playing
        self.sustainedSteps = 0

        let paramsJSON = LevelLoader.paramsJSON(from: config)
        engine = SimulationEngine(seed: config.startingSeed, paramsJSON: paramsJSON)

        // Warm up nutrients
        engine?.step(500)

        // Seed species based on level
        seedSpecies(for: config.id)
        refreshSnapshot()
    }

    func startLevel(seed: UInt64, paramsJSON: String? = nil) {
        engine = SimulationEngine(seed: seed, paramsJSON: paramsJSON)
        engine?.step(500)
        let microbeJSON = """
        {"id":0,"name":"Thermophile","traits":{"temp_optimal":5.0,"temp_range":60.0,"o2_need":0.0,"toxin_resistance":0.3,"trophic_level":"Producer","reproduction_rate":0.04,"dispersal":0.3,"mutation_rate":0.005}}
        """
        engine?.addSpecies(json: microbeJSON, initialPopulation: 100.0)
        refreshSnapshot()
    }

    private func seedSpecies(for levelId: String) {
        switch levelId {
        case "level_01":
            let microbeJSON = """
            {"id":0,"name":"Thermophile","traits":{"temp_optimal":5.0,"temp_range":60.0,"o2_need":0.0,"toxin_resistance":0.3,"trophic_level":"Producer","reproduction_rate":0.04,"dispersal":0.3,"mutation_rate":0.005}}
            """
            engine?.addSpecies(json: microbeJSON, initialPopulation: 100.0)

        case "level_02":
            let producer = """
            {"id":0,"name":"Planktonic Algae","traits":{"temp_optimal":18.0,"temp_range":40.0,"o2_need":0.0,"toxin_resistance":0.1,"trophic_level":"Producer","reproduction_rate":0.06,"dispersal":0.5,"mutation_rate":0.01}}
            """
            engine?.addSpecies(json: producer, initialPopulation: 200.0)

            let consumer = """
            {"id":1,"name":"Grazer","traits":{"temp_optimal":16.0,"temp_range":30.0,"o2_need":0.05,"toxin_resistance":0.2,"trophic_level":"Consumer","reproduction_rate":0.03,"dispersal":0.3,"mutation_rate":0.008}}
            """
            engine?.addSpecies(json: consumer, initialPopulation: 50.0)

            let predator = """
            {"id":2,"name":"Apex Filter","traits":{"temp_optimal":17.0,"temp_range":25.0,"o2_need":0.08,"toxin_resistance":0.15,"trophic_level":"Predator","reproduction_rate":0.015,"dispersal":0.2,"mutation_rate":0.005}}
            """
            engine?.addSpecies(json: predator, initialPopulation: 15.0)

        default:
            let microbeJSON = """
            {"id":0,"name":"Microbe","traits":{"temp_optimal":15.0,"temp_range":50.0,"o2_need":0.0,"toxin_resistance":0.1,"trophic_level":"Producer","reproduction_rate":0.05,"dispersal":0.3,"mutation_rate":0.01}}
            """
            engine?.addSpecies(json: microbeJSON, initialPopulation: 100.0)
        }
    }

    // MARK: - Simulation Control

    func togglePause() {
        guard levelStatus == .playing else { return }
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
        guard let engine, levelStatus == .playing else { return }
        engine.step(timeSpeed.stepsPerBatch)
        refreshSnapshot()
        evaluateObjective()
    }

    func refreshSnapshot() {
        guard let engine else { return }
        engine.updateSnapshot()
        currentStep = engine.currentStep
        biodiversity = engine.biodiversityCount
        totalBiomass = engine.totalBiomass
        temperatures = engine.temperatures
        nutrients = engine.nutrients
        moisture = engine.moisture
        populationDensity = engine.populationDensity
        oceanMask = engine.oceanMask

        // Sync codex unlocks
        if let data = engine.codexUnlockedJSON.data(using: .utf8),
           let ids = try? JSONDecoder().decode([String].self, from: data) {
            newCodexUnlocks = ids
        }
    }

    // MARK: - Objective Evaluation

    private func evaluateObjective() {
        guard let engine, let objJSON = objectiveJSON else { return }

        if let result = engine.evaluateObjective(json: objJSON) {
            if result.extinct {
                levelStatus = .failed(reason: "All life has gone extinct.")
                isPaused = true
                simulationTask?.cancel()
                return
            }

            if result.conditionMet {
                sustainedSteps += timeSpeed.stepsPerBatch
            } else {
                sustainedSteps = 0
            }

            if sustainedSteps >= requiredSteps {
                levelStatus = .won
                isPaused = true
                simulationTask?.cancel()
            }
        }
    }

    // MARK: - Interventions

    func applyIntervention(kind: String, magnitude: Float) {
        guard let engine, levelStatus == .playing else { return }

        let energyCost: Float = magnitude * 5.0
        guard energyRemaining >= energyCost else { return }

        let json: String
        switch kind {
        case "AdjustCO2":
            json = #"{"kind":{"AdjustCO2":{"delta":\#(magnitude)}},"target_region":null,"step":\#(currentStep)}"#
        case "AdjustO2":
            json = #"{"kind":{"AdjustO2":{"delta":\#(magnitude)}},"target_region":null,"step":\#(currentStep)}"#
        case "NutrientBloom":
            json = #"{"kind":{"NutrientBloom":{"magnitude":\#(magnitude)}},"target_region":{"x":32,"y":16,"radius":10},"step":\#(currentStep)}"#
        case "IceMeltPulse":
            json = #"{"kind":{"IceMeltPulse":{"magnitude":\#(magnitude)}},"target_region":null,"step":\#(currentStep)}"#
        case "AdjustCurrents":
            json = #"{"kind":{"AdjustCurrents":{"delta":\#(magnitude)}},"target_region":null,"step":\#(currentStep)}"#
        case "AdjustSalinity":
            json = #"{"kind":{"AdjustSalinity":{"delta":\#(magnitude)}},"target_region":null,"step":\#(currentStep)}"#
        default:
            return
        }

        if engine.applyIntervention(json: json) {
            energyRemaining -= energyCost
        }
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
