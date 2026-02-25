import Foundation

// MARK: - Planet Parameters

struct PlanetParams: Codable {
    var gravity: Float = 9.8
    var rotationRate: Float = 1.0
    var axialTilt: Float = 23.4
    var coreHeat: Float = 0.4
    var magneticField: Float = 0.6
    var atmosphere: AtmosphereState = .init()
    var hydrology: HydroState = .init()
}

struct AtmosphereState: Codable {
    var pressure: Float = 1.0
    var o2: Float = 0.21
    var co2: Float = 0.0004
    var toxicity: Float = 0.0
}

struct HydroState: Codable {
    var oceanCoverage: Float = 0.7
    var salinity: Float = 0.035
    var currentStrength: Float = 0.5
    var iceFraction: Float = 0.1
}

// MARK: - Time

enum TimeSpeed: String, Codable {
    case observe
    case adapt
    case epoch
    case eon

    var stepsPerBatch: UInt64 {
        switch self {
        case .observe: return 1
        case .adapt: return 100
        case .epoch: return 10_000
        case .eon: return 1_000_000
        }
    }
}

// MARK: - Species

enum TrophicLevel: String, Codable {
    case producer
    case consumer
    case predator
}

struct SpeciesTraits: Codable {
    var tempOptimal: Float
    var tempRange: Float
    var o2Need: Float
    var toxinResistance: Float
    var trophicLevel: TrophicLevel
    var reproductionRate: Float
    var dispersal: Float
    var mutationRate: Float
}

struct Species: Codable, Identifiable {
    let id: UInt32
    var name: String
    var traits: SpeciesTraits
}

// MARK: - Interventions

enum InterventionKind: Codable {
    case adjustCO2(delta: Float)
    case adjustO2(delta: Float)
    case cloudSeeding(magnitude: Float)
    case nutrientBloom(magnitude: Float)
    case iceMeltPulse(magnitude: Float)
}

struct RegionTarget: Codable {
    let x: Int
    let y: Int
    let radius: Int
}

struct Intervention: Codable {
    let kind: InterventionKind
    let targetRegion: RegionTarget?
    let step: UInt64
}

// MARK: - Level Spec

enum Pack: String, Codable {
    case free = "FREE"
    case core = "PACK_CORE"
    case advanced = "PACK_ADV"
}

struct LevelSpec: Codable, Identifiable {
    let id: String
    let name: String
    let pack: Pack
    let description: String
    let startingSeed: UInt64
}

// MARK: - Codex

enum CodexCategory: String, Codable {
    case species
    case bodyPlan
    case biome
    case planetarySystem
    case evolutionaryEvent
    case failureMode
    case rarePhenomenon
    case historicWorld
}

struct CodexEntry: Codable, Identifiable {
    let id: String
    let category: CodexCategory
    let name: String
    let requirementsText: String
    let factsText: String
    let flavorText: String
    let relatedEntryIds: [String]
    let iconAssetId: String
}

// MARK: - Snapshot

struct SimSnapshot {
    let currentStep: UInt64
    let biodiversityCount: UInt32
    let species: [Species]
}
