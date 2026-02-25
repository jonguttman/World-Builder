import Foundation

struct LevelConfig: Codable {
    let id: String
    let name: String
    let pack: String
    let description: String
    let startingSeed: UInt64
    let startingParams: StartingParams?
    let allowedInterventions: [String]
    let energyBudget: Float
    let objective: ObjectiveConfig
    let failConditions: [String]

    struct StartingParams: Codable {
        let gravity: Float
        let rotationRate: Float
        let axialTilt: Float
        let coreHeat: Float
        let magneticField: Float
        let atmosphere: AtmoConfig
        let hydrology: HydroConfig

        struct AtmoConfig: Codable {
            let pressure: Float
            let o2: Float
            let co2: Float
            let toxicity: Float
        }

        struct HydroConfig: Codable {
            let oceanCoverage: Float
            let salinity: Float
            let currentStrength: Float
            let iceFraction: Float
        }
    }

    struct ObjectiveConfig: Codable {
        let type: String
        let minBiomass: Double?
        let requiredDurationSteps: UInt64

        enum CodingKeys: String, CodingKey {
            case type
            case minBiomass = "min_biomass"
            case requiredDurationSteps = "required_duration_steps"
        }
    }
}

enum LevelLoader {
    static func load(levelId: String) -> LevelConfig? {
        guard let url = Bundle.main.url(forResource: levelId, withExtension: "json") else {
            return nil
        }
        return decode(url: url)
    }

    private static func decode(url: URL) -> LevelConfig? {
        guard let data = try? Data(contentsOf: url) else { return nil }
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return try? decoder.decode(LevelConfig.self, from: data)
    }

    static func paramsJSON(from config: LevelConfig) -> String? {
        guard let params = config.startingParams else { return nil }
        let dict: [String: Any] = [
            "gravity": params.gravity,
            "rotation_rate": params.rotationRate,
            "axial_tilt": params.axialTilt,
            "core_heat": params.coreHeat,
            "magnetic_field": params.magneticField,
            "atmosphere": [
                "pressure": params.atmosphere.pressure,
                "o2": params.atmosphere.o2,
                "co2": params.atmosphere.co2,
                "toxicity": params.atmosphere.toxicity,
            ],
            "hydrology": [
                "ocean_coverage": params.hydrology.oceanCoverage,
                "salinity": params.hydrology.salinity,
                "current_strength": params.hydrology.currentStrength,
                "ice_fraction": params.hydrology.iceFraction,
            ],
        ]
        guard let data = try? JSONSerialization.data(withJSONObject: dict) else { return nil }
        return String(data: data, encoding: .utf8)
    }

    static func objectiveJSON(from config: LevelConfig) -> String? {
        let obj = config.objective
        var dict: [String: Any] = [
            "type": obj.type,
            "required_duration_steps": obj.requiredDurationSteps,
        ]
        if let minBiomass = obj.minBiomass {
            dict["min_biomass"] = minBiomass
        }
        guard let data = try? JSONSerialization.data(withJSONObject: dict) else { return nil }
        return String(data: data, encoding: .utf8)
    }
}
