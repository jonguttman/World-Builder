# Data Models

## LevelAuthoring
LevelSpec: id, name, pack (FREE|PACK_CORE|PACK_ADV), description, starting_seed, locked_planet_params, player_editable_params, intervention_rules (type, max_uses, cooldown_steps, energy_cost), objective (type + thresholds + required_duration), fail_conditions

## Codex
CodexEntry: id, category, name, unlock_trigger (enum + params), requirements_text, facts_text, flavor_text, related_entry_ids, icon_asset_id
