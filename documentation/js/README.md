## Function signatures
```TypeScript
function parseChatMessages(path: string): any
function listGameEvents(path: string): any
function parseGrenades(path: string): any
function parseHeader(path: string): any
function parsePlayerInfo(path: string): any

function parseEvent(path: string, eventName: string, extraPlayer?: Array<string> | undefined | null, extraOther?: Array<string> | undefined | null): any
function parseEvents(path: string, eventNames?: Array<string> | undefined | null, extraPlayer?: Array<string> | undefined | null, extraOther?: Array<string> | undefined | null): any
function parseTicks(path: string, wantedProps: Array<string>, wantedTicks?: Array<number> | undefined | null): any
```


