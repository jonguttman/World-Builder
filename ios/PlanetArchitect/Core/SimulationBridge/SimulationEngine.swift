import Foundation

/// Type-safe Swift wrapper around the Rust sim-core FFI.
/// Manages the opaque simulation handle lifecycle and provides
/// Swift-native access to simulation data.
///
/// `PASimHandle` (C: `typedef void* PASimHandle`) is imported by Swift as
/// `OpaquePointer?`. All FFI calls go through this opaque pointer.
///
/// Thread safety: marked `@unchecked Sendable` — callers are responsible
/// for serialising access (e.g. via an actor or serial DispatchQueue).
final class SimulationEngine: @unchecked Sendable {

    // MARK: - Stored Properties

    /// Non-optional handle; we guarantee it's valid from init to deinit.
    private let handle: OpaquePointer

    /// Cached grid dimensions (immutable for the lifetime of a simulation).
    let gridWidth: Int
    let gridHeight: Int
    private let tileCount: Int

    // MARK: - Initialisers

    /// Create a brand-new simulation.
    /// - Parameters:
    ///   - seed: Deterministic RNG seed.
    ///   - paramsJSON: Optional JSON string encoding `PlanetParams`.
    /// - Returns: `nil` if the Rust side failed to allocate.
    init?(seed: UInt64, paramsJSON: String? = nil) {
        let raw: OpaquePointer?
        if let json = paramsJSON {
            raw = json.withCString { pa_sim_create(seed, $0) }
        } else {
            raw = pa_sim_create(seed, nil)
        }
        guard let h = raw else { return nil }
        self.handle = h
        self.gridWidth  = Int(pa_sim_snapshot_width(h))
        self.gridHeight = Int(pa_sim_snapshot_height(h))
        self.tileCount  = gridWidth * gridHeight
    }

    /// Internal initialiser used by `loadState(from:)`.
    private init(handle: OpaquePointer, width: Int, height: Int) {
        self.handle    = handle
        self.gridWidth  = width
        self.gridHeight = height
        self.tileCount  = width * height
    }

    deinit {
        pa_sim_destroy(handle)
    }

    // MARK: - Simulation Control

    /// Advance the simulation by the given number of steps.
    func step(_ steps: UInt64) {
        pa_sim_step(handle, steps)
    }

    /// The current simulation tick.
    var currentStep: UInt64 {
        pa_sim_current_step(handle)
    }

    // MARK: - Snapshot

    /// Refresh the cached snapshot arrays on the Rust side.
    /// Must be called before reading any tile-level data.
    func updateSnapshot() {
        pa_sim_snapshot_update(handle)
    }

    /// Per-tile temperature values (length == `tileCount`).
    /// Returns an empty array if the snapshot pointer is NULL.
    var temperatures: [Float] {
        guard let ptr = pa_sim_snapshot_temperatures(handle) else { return [] }
        return Array(UnsafeBufferPointer(start: ptr, count: tileCount))
    }

    /// Per-tile nutrient values.
    var nutrients: [Float] {
        guard let ptr = pa_sim_snapshot_nutrients(handle) else { return [] }
        return Array(UnsafeBufferPointer(start: ptr, count: tileCount))
    }

    /// Per-tile moisture values.
    var moisture: [Float] {
        guard let ptr = pa_sim_snapshot_moisture(handle) else { return [] }
        return Array(UnsafeBufferPointer(start: ptr, count: tileCount))
    }

    /// Per-tile population density.
    var populationDensity: [Float] {
        guard let ptr = pa_sim_snapshot_population(handle) else { return [] }
        return Array(UnsafeBufferPointer(start: ptr, count: tileCount))
    }

    /// Per-tile ocean mask (1 = ocean, 0 = land).
    var oceanMask: [UInt8] {
        guard let ptr = pa_sim_snapshot_ocean_mask(handle) else { return [] }
        return Array(UnsafeBufferPointer(start: ptr, count: tileCount))
    }

    /// Total number of extant species.
    var biodiversityCount: UInt32 {
        pa_sim_snapshot_biodiversity(handle)
    }

    /// JSON array describing all current species.
    var speciesJSON: String {
        guard let ptr = pa_sim_snapshot_species_json(handle) else { return "[]" }
        return String(cString: ptr)
    }

    // MARK: - Species

    /// Introduce a new species described by a JSON string.
    /// - Parameters:
    ///   - json: JSON-encoded species traits.
    ///   - initialPopulation: Starting population count.
    func addSpecies(json: String, initialPopulation: Double) {
        json.withCString { pa_sim_add_species_json(handle, $0, initialPopulation) }
    }

    // MARK: - Interventions

    /// Apply a planet-level intervention.
    /// - Parameter json: JSON-encoded `Intervention`.
    /// - Returns: `true` if the Rust side accepted the intervention.
    @discardableResult
    func applyIntervention(json: String) -> Bool {
        json.withCString { pa_sim_apply_intervention_json(handle, $0) } == 0
    }

    // MARK: - Save / Load

    /// Serialise the full simulation state to `Data`.
    /// Returns `nil` if serialisation fails on the Rust side.
    func saveState() -> Data? {
        var len: Int = 0
        guard let ptr = pa_sim_save_state(handle, &len), len > 0 else { return nil }
        let data = Data(bytes: ptr, count: len)
        pa_free_bytes(ptr, len)
        return data
    }

    /// Restore a simulation from previously saved state bytes.
    /// - Parameter data: Bytes returned by a prior `saveState()` call.
    /// - Returns: A fully hydrated `SimulationEngine`, or `nil` on failure.
    static func loadState(from data: Data) -> SimulationEngine? {
        data.withUnsafeBytes { buffer -> SimulationEngine? in
            guard let baseAddress = buffer.baseAddress else { return nil }
            let ptr = baseAddress.assumingMemoryBound(to: UInt8.self)
            guard let h = pa_sim_load_state(ptr, buffer.count) else { return nil }
            let width  = Int(pa_sim_snapshot_width(h))
            let height = Int(pa_sim_snapshot_height(h))
            return SimulationEngine(handle: h, width: width, height: height)
        }
    }
}
