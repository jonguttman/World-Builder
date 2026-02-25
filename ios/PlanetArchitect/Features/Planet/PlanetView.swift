import SwiftUI

struct PlanetView: View {
    let levelId: Int
    @State private var timeSpeed: TimeSpeed = .observe
    @State private var currentStep: UInt64 = 0
    @State private var isPaused: Bool = true

    var body: some View {
        VStack {
            ZStack {
                Circle()
                    .fill(
                        RadialGradient(
                            colors: [.blue, .green, .brown],
                            center: .center,
                            startRadius: 50,
                            endRadius: 150
                        )
                    )
                    .frame(width: 300, height: 300)

                Text("Level \(levelId)")
                    .font(.title2)
                    .foregroundColor(.white)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)

            HStack(spacing: 20) {
                Button(isPaused ? "Play" : "Pause") {
                    isPaused.toggle()
                }

                Picker("Speed", selection: $timeSpeed) {
                    Text("1x").tag(TimeSpeed.observe)
                    Text("100x").tag(TimeSpeed.adapt)
                    Text("10K").tag(TimeSpeed.epoch)
                    Text("1M").tag(TimeSpeed.eon)
                }
                .pickerStyle(.segmented)
            }
            .padding()

            Text("Step: \(currentStep)")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .navigationTitle("Planet View")
        .navigationBarTitleDisplayMode(.inline)
    }
}
