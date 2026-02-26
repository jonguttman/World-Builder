# Bug Log

<!-- Document bugs here with: symptoms, root cause, fix, prevention -->

## BUG-001: Levels fail to load — no briefing, no interventions
- **Symptoms:** Selecting a level shows an empty PlanetView with "Level" title, 0 yr/0 biomass/0 species, no intervention tray, no briefing overlay
- **Root cause:** `ObjectiveConfig` had custom `CodingKeys` with snake_case raw values (`"min_biomass"`, `"required_duration_steps"`) while the JSON decoder also used `.convertFromSnakeCase`. This double-conversion caused key mismatches: the decoder converted `"required_duration_steps"` → `"requiredDurationSteps"`, then failed to match it against the CodingKey raw value `"required_duration_steps"`. Since `requiredDurationSteps` is non-optional, the entire `LevelConfig` decode failed silently (`try?` returned nil).
- **Fix:** Removed redundant custom `CodingKeys` from `ObjectiveConfig` — `.convertFromSnakeCase` already handles the conversion.
- **Prevention:** Never combine `.convertFromSnakeCase` with custom CodingKeys that specify snake_case raw values. Use one or the other, not both.
