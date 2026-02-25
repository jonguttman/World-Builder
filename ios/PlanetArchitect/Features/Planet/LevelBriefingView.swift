import SwiftUI

struct LevelBriefingView: View {
    let config: LevelConfig
    let onStart: () -> Void

    var body: some View {
        VStack(spacing: 20) {
            Text(config.name)
                .font(.largeTitle.bold())

            Text(config.description)
                .font(.body)
                .multilineTextAlignment(.center)
                .foregroundStyle(.secondary)
                .padding(.horizontal)

            Divider()

            VStack(alignment: .leading, spacing: 8) {
                Label("Objective", systemImage: "target")
                    .font(.headline)
                Text(objectiveDescription)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal)

            VStack(alignment: .leading, spacing: 8) {
                Label("Tools Available", systemImage: "wrench")
                    .font(.headline)
                Text(config.allowedInterventions.map { humanName($0) }.joined(separator: ", "))
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal)

            Label(String(format: "Energy Budget: %.0f", config.energyBudget), systemImage: "bolt.fill")
                .font(.subheadline)
                .foregroundStyle(.orange)

            Spacer()

            Button("Begin Simulation") {
                onStart()
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
        }
        .padding()
    }

    private var objectiveDescription: String {
        let obj = config.objective
        switch obj.type {
        case "MicrobialStability":
            let biomass = obj.minBiomass ?? 0
            let years = obj.requiredDurationSteps
            return "Establish and sustain microbial life (biomass > \(Int(biomass))) for \(formatSteps(years))."
        case "EcosystemStability":
            let levels = obj.minTrophicLevels ?? 3
            let years = obj.requiredDurationSteps
            return "Maintain a stable ecosystem with \(levels) trophic levels for \(formatSteps(years))."
        default:
            return "Complete the level objective."
        }
    }

    private func humanName(_ kind: String) -> String {
        switch kind {
        case "AdjustCO2": return "CO2 Adjustment"
        case "AdjustO2": return "O2 Adjustment"
        case "NutrientBloom": return "Nutrient Bloom"
        case "IceMeltPulse": return "Ice Melt Pulse"
        case "CloudSeeding": return "Cloud Seeding"
        case "AdjustCurrents": return "Current Adjustment"
        case "AdjustSalinity": return "Salinity Adjustment"
        default: return kind
        }
    }

    private func formatSteps(_ steps: UInt64) -> String {
        if steps >= 1_000_000 {
            return "\(steps / 1_000_000)M years"
        } else if steps >= 1_000 {
            return "\(steps / 1_000)K years"
        }
        return "\(steps) years"
    }
}
