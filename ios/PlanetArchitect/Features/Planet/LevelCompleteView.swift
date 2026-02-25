import SwiftUI

struct LevelCompleteView: View {
    let won: Bool
    let failReason: String?
    let steps: UInt64
    let biodiversity: UInt32
    let onRestart: () -> Void
    let onExit: () -> Void

    var body: some View {
        VStack(spacing: 24) {
            Image(systemName: won ? "checkmark.circle.fill" : "xmark.circle.fill")
                .font(.system(size: 64))
                .foregroundStyle(won ? .green : .red)

            Text(won ? "Level Complete!" : "Level Failed")
                .font(.largeTitle.bold())

            if let reason = failReason {
                Text(reason)
                    .font(.body)
                    .foregroundStyle(.secondary)
            }

            VStack(spacing: 8) {
                HStack {
                    Text("Time Elapsed")
                    Spacer()
                    Text(formatSteps(steps))
                }
                HStack {
                    Text("Species")
                    Spacer()
                    Text("\(biodiversity)")
                }
            }
            .font(.body)
            .padding()
            .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 12))

            Spacer()

            HStack(spacing: 16) {
                Button("Try Again") {
                    onRestart()
                }
                .buttonStyle(.bordered)

                Button(won ? "Continue" : "Back") {
                    onExit()
                }
                .buttonStyle(.borderedProminent)
            }
        }
        .padding()
    }

    private func formatSteps(_ steps: UInt64) -> String {
        if steps >= 1_000_000 {
            return String(format: "%.1fM years", Double(steps) / 1_000_000)
        } else if steps >= 1_000 {
            return String(format: "%.0fK years", Double(steps) / 1_000)
        }
        return "\(steps) years"
    }
}
