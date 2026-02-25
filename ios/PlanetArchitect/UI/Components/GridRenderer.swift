import SwiftUI

struct GridRenderer: View {
    let data: [Float]
    let oceanMask: [UInt8]
    let width: Int
    let height: Int
    let overlayMode: OverlayMode
    var onTileTap: ((Int, Int) -> Void)?

    var body: some View {
        GeometryReader { geo in
            Canvas { context, size in
                guard !data.isEmpty, width > 0, height > 0 else { return }

                let tileW = size.width / CGFloat(width)
                let tileH = size.height / CGFloat(height)

                for y in 0..<height {
                    for x in 0..<width {
                        let index = y * width + x
                        guard index < data.count else { continue }

                        let isOcean = index < oceanMask.count && oceanMask[index] == 1
                        let color = tileColor(value: data[index], isOcean: isOcean)
                        let rect = CGRect(
                            x: CGFloat(x) * tileW,
                            y: CGFloat(y) * tileH,
                            width: tileW + 0.5,
                            height: tileH + 0.5
                        )
                        context.fill(Path(rect), with: .color(color))
                    }
                }
            }
            .contentShape(Rectangle())
            .onTapGesture { location in
                let tileW = geo.size.width / CGFloat(width)
                let tileH = geo.size.height / CGFloat(height)
                let x = Int(location.x / tileW)
                let y = Int(location.y / tileH)
                if x >= 0 && x < width && y >= 0 && y < height {
                    onTileTap?(x, y)
                }
            }
        }
    }

    private func tileColor(value: Float, isOcean: Bool) -> Color {
        switch overlayMode {
        case .temperature:
            return temperatureColor(value)
        case .nutrients:
            return scalarColor(value, low: Color(red: 0.05, green: 0.05, blue: 0.05),
                             high: Color(red: 0.2, green: 0.9, blue: 0.1))
        case .moisture:
            return scalarColor(value, low: Color(red: 0.3, green: 0.2, blue: 0.1),
                             high: Color(red: 0.1, green: 0.3, blue: 0.9))
        case .population:
            if value <= 0 {
                return isOcean ? Color(white: 0.12) : Color(white: 0.2)
            }
            let intensity = min(Double(value) / 5000.0, 1.0)
            return isOcean
                ? Color(red: 0, green: intensity * 0.8, blue: intensity)
                : Color(red: intensity * 0.2, green: intensity, blue: 0)
        }
    }

    private func temperatureColor(_ temp: Float) -> Color {
        let t = Double((temp + 80.0) / 160.0).clamped(to: 0...1)
        if t < 0.25 {
            let f = t / 0.25
            return Color(red: 0, green: 0, blue: 0.3 + f * 0.7)
        } else if t < 0.5 {
            let f = (t - 0.25) / 0.25
            return Color(red: 0, green: f, blue: 1.0 - f * 0.5)
        } else if t < 0.75 {
            let f = (t - 0.5) / 0.25
            return Color(red: f, green: 1.0 - f * 0.3, blue: 0)
        } else {
            let f = (t - 0.75) / 0.25
            return Color(red: 1.0, green: 0.7 - f * 0.7, blue: 0)
        }
    }

    private func scalarColor(_ val: Float, low: Color, high: Color) -> Color {
        let t = Double(val.clamped(to: 0...1))
        return blend(from: low, to: high, fraction: t)
    }

    private func blend(from: Color, to: Color, fraction: Double) -> Color {
        let f = fraction.clamped(to: 0...1)
        let r1 = from.components.red, g1 = from.components.green, b1 = from.components.blue
        let r2 = to.components.red, g2 = to.components.green, b2 = to.components.blue
        return Color(
            red: r1 + (r2 - r1) * f,
            green: g1 + (g2 - g1) * f,
            blue: b1 + (b2 - b1) * f
        )
    }
}

// MARK: - Helpers

private extension Comparable {
    func clamped(to range: ClosedRange<Self>) -> Self {
        min(max(self, range.lowerBound), range.upperBound)
    }
}

private extension Color {
    struct Components {
        let red: Double, green: Double, blue: Double
    }
    var components: Components {
        var r: CGFloat = 0, g: CGFloat = 0, b: CGFloat = 0, a: CGFloat = 0
        UIColor(self).getRed(&r, green: &g, blue: &b, alpha: &a)
        return Components(red: Double(r), green: Double(g), blue: Double(b))
    }
}
