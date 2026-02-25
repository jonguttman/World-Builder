import SwiftUI

struct TutorialOverlay: View {
    let steps: [TutorialStep]
    @Binding var currentStepIndex: Int
    @Binding var isVisible: Bool

    var body: some View {
        if isVisible, currentStepIndex < steps.count {
            let step = steps[currentStepIndex]

            VStack(spacing: 16) {
                Spacer()

                VStack(spacing: 12) {
                    Text(step.title)
                        .font(.headline)

                    Text(step.message)
                        .font(.body)
                        .multilineTextAlignment(.center)
                        .foregroundStyle(.secondary)

                    Button(currentStepIndex < steps.count - 1 ? "Next" : "Got it") {
                        if currentStepIndex < steps.count - 1 {
                            currentStepIndex += 1
                        } else {
                            isVisible = false
                        }
                    }
                    .buttonStyle(.borderedProminent)
                }
                .padding(24)
                .background(.ultraThickMaterial, in: RoundedRectangle(cornerRadius: 16))
                .padding(.horizontal, 32)

                Spacer()
                    .frame(height: 80)
            }
        }
    }
}

struct TutorialStep {
    let title: String
    let message: String
}

enum LevelTutorials {
    static let level1: [TutorialStep] = [
        TutorialStep(
            title: "Welcome, Architect",
            message: "This barren world has a thin CO2 atmosphere and frozen oceans. Your goal: create conditions where microbial life can survive for 10 million years."
        ),
        TutorialStep(
            title: "Use Interventions",
            message: "Tap the tools at the bottom to adjust your planet's atmosphere and surface. Each action costs energy from your limited budget."
        ),
        TutorialStep(
            title: "Watch the Overlays",
            message: "Switch between Temperature, Nutrients, Moisture, and Population views to understand what's happening on your world."
        ),
        TutorialStep(
            title: "Control Time",
            message: "Use the speed controls to fast-forward through geological time. Watch for changes in your biosphere — life is fragile."
        ),
    ]
}
