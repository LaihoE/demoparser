Work in progress!

Development of source 2 demos are moved here. Not sure yet where code for source 1 demos will go.


Project moves fast so no guarantees anything will stay the same.

Following mostly this: https://github.com/dotabuff/manta formats seems similar.



maybe you can get the project to run if you edit main.rs path to your own demo and run this: 
```rust
cargo run --release
```
requires at least cargo but not too sure about others.


Progress:

- [x] Game events (somewhat done, can list most events (what are type 8 and 9?))
- [x] Server info
- [x] Sendtables
- [x] Serverclasses
- [x] Header
- [x] Packet entites
- [ ] String tables


Values to try:

m_vecX
m_vecY
m_vecZ
m_iHealth
m_iTeamNum
m_bInBuyZone
m_szLastPlaceName
m_iWeaponPurchasesThisRound
m_szRagdollDamageWeaponName
m_unTotalRoundDamageDealt