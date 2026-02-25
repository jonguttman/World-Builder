# Entitlements & Purchases (StoreKit2)

## Products
- planet_architect_pack_core (non-consumable)
- planet_architect_pack_adv (non-consumable)

## Entitlements
- has_core_pack boolean
- has_adv_pack boolean

## Rules
- PACK_ADV does NOT imply PACK_CORE unless bundled
- Recommended: PACK_ADV access 11–20 only, store UI encourages buying Core first

## Implementation
- StoreKit2 Transaction.currentEntitlements
- Cache locally with last_verified timestamp
- Restore purchases button
- Offline grace: cached entitlements for 7 days

## UX
- Paywall only after Level 3 completion or explicit tap on locked level
- No interrupt popups mid-game
