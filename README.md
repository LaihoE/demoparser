# Demo parser for Counter-Strike 2


Work in progress so expect some bugs here and there!

## Install
```pip install demoparser2```

### Usage:
Game events
```python
from demoparser2 import DemoParser

parser = DemoParser("path_to_demo.dem")
df = parser.parse_events("player_death")
```

Tick data (entities)
```python
from demoparser2 import DemoParser

wanted_props = ["m_vecX", "m_vecY"]

parser = DemoParser("path_to_demo.dem")
df = parser.parse_ticks(wanted_props)
```

Both functions return a Pandas Dataframe.

Example output:
```
             m_vecX       m_vecY   tick            steamid        name
0        649.795044   633.648438      0  76512345678912345      person1
1        526.207642   794.186157      0  76512345678912345      person2
2        997.494324   583.692871      0  76512345678912345      person3
3        958.421570   498.485657      0  76512345678912345      person4
4        624.525696   556.522217      0  76512345678912345      person5
...             ...          ...    ...                ...         ...
913215   924.593140   308.131622  30452  76512345678912345      person6
913216   598.564514   794.186157  30452  76512345678912345      person7
913217   329.393677   323.219360  30452  76512345678912345      person8
913218   526.207642    81.611084  30452  76512345678912345      person9
913219    36.691650   308.887451  30452  76512345678912345      person10
```
(steamids and names are made up in this example)

You can also filter ticks:
```python
df = parser.parse_ticks(wanted_props, ticks=[x for x in range(29000, 30000)])
```



### Progress:

- [x] Game events
- [x] Server info
- [x] Sendtables
- [x] Serverclasses
- [x] Header
- [x] Packet entites
- [ ] String tables


### Values to try for tick data (more on the way):

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

m_hPlayerPing
m_nSurvivalTeamNumber
m_unCurrentEquipmentValue
m_flFlashDuration
m_nWhichBombZone
m_iMoveState
m_bInNoDefuseArea
m_bHasMovedSinceSpawn
m_bIsDefusing
m_bIsScoped
m_bSpottedByMask
m_iMaxHealth
m_angEyeAngles
m_bombsiteCenterA
m_bombsiteCenterB
m_iMVPs


m_iPlayerState
m_pActionTrackingServices
CPlayer_MovementServices
CPlayer_ItemServices
CCSPlayer_ItemServices
m_iAmmo
m_hActiveWeapon
m_iScore


CCSPlayerController_InGameMoneyServices
m_iTotalCashSpent
m_iTotalCashSpent
m_iStartAccount
m_iAccount


m_steamID
m_iszPlayerName

m_iNumConsecutiveTerroristLoses
m_iMatchStats_RoundResults
m_iMatchStats_PlayersAlive_T
m_bBombPlanted
m_bBombDropped
m_szMatchStatTxt
m_bIsQueuedMatchmaking
m_nOvertimePlaying
m_totalRoundsPlayed
m_gamePhase
m_flGameStartTime
m_fRoundStartTime
m_flRestartRoundTime
m_fMatchStartTime
m_iRoundTime
m_bTechnicalTimeOut
m_bWarmupPeriod

m_iCompetitiveRankType
m_iCompetitiveWins
m_iCompetitiveRanking
m_iCompTeammateColor
m_szCrosshairCodes
m_bHasCommunicationAbuseMute
m_iPing

m_nPersonaDataPublicCommendsFriendly
m_nPersonaDataPublicCommendsTeacher
m_nPersonaDataPublicCommendsLeader
m_unMusicID

CSPerRoundStats_t

var_name: "m_vecKills",
var_type: "CNetworkUtlVectorBase< EKillTypes_t >",

var_name: "m_DamageList",
var_type: "CUtlVectorEmbeddedNetworkVar< CDamageRecord >",

var_name: "m_perRoundStats",
var_type: "CUtlVectorEmbeddedNetworkVar< CSPerRoundStats_t >",






## Acknowledgements
Without Dotabuff's dota 2 parser manta this would not have been possible. Check it out: https://github.com/dotabuff/manta

The dota 2 demo format is very similar to CS2 demo format with only a few minor changes.