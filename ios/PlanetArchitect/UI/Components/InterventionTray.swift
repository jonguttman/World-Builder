import SwiftUI

struct InterventionTray: View {
    let allowedInterventions: [String]
    let energyRemaining: Float
    let onApply: (String, Float) -> Void

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 12) {
                ForEach(allowedInterventions, id: \.self) { kind in
                    InterventionButton(
                        kind: kind,
                        energyRemaining: energyRemaining,
                        onApply: onApply
                    )
                }
            }
            .padding(.horizontal)
        }
    }
}

private struct InterventionButton: View {
    let kind: String
    let energyRemaining: Float
    let onApply: (String, Float) -> Void

    private var label: String {
        switch kind {
        case "AdjustCO2": return "CO2"
        case "AdjustO2": return "O2"
        case "NutrientBloom": return "Nutrients"
        case "IceMeltPulse": return "Melt Ice"
        case "CloudSeeding": return "Clouds"
        case "AdjustCurrents": return "Currents"
        case "AdjustSalinity": return "Salinity"
        default: return kind
        }
    }

    private var icon: String {
        switch kind {
        case "AdjustCO2": return "carbon.dioxide.cloud"
        case "AdjustO2": return "wind"
        case "NutrientBloom": return "leaf.arrow.circlepath"
        case "IceMeltPulse": return "snowflake"
        case "CloudSeeding": return "cloud.rain"
        case "AdjustCurrents": return "water.waves"
        case "AdjustSalinity": return "drop.triangle"
        default: return "wand.and.stars"
        }
    }

    private var magnitude: Float {
        switch kind {
        case "AdjustCO2": return 0.02
        case "AdjustO2": return 0.03
        case "NutrientBloom": return 0.3
        case "IceMeltPulse": return 0.5
        case "CloudSeeding": return 0.4
        case "AdjustCurrents": return 0.15
        case "AdjustSalinity": return 0.05
        default: return 0.1
        }
    }

    private var cost: Float { magnitude * 5.0 }
    private var canAfford: Bool { energyRemaining >= cost }

    var body: some View {
        Button {
            onApply(kind, magnitude)
        } label: {
            VStack(spacing: 4) {
                Image(systemName: icon)
                    .font(.title3)
                Text(label)
                    .font(.caption2)
                Text(String(format: "-%.0f", cost))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            .frame(width: 64, height: 64)
            .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 10))
        }
        .disabled(!canAfford)
        .opacity(canAfford ? 1.0 : 0.4)
    }
}
