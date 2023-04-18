### Welcome to the source code




The outer loop is very simple and can be found in parsing/parser.rs. This is enough to get trough the file:

1. Read what command next bytes will be.
2. Read what tick the next command is.
3. Read how many bytes the next command is.

```Rust
    pub fn start(&mut self){
        loop {
            let cmd = self.read_varint()?;
            let tick = self.read_varint()?;
            let size = self.read_varint()?;

            let bytes = self.bytes[cur_pos..cur_pos + size]
            cur_pos += size

            match demo_cmd_type_from_int(msg_type as i32) {
                DEM_Packet => self.parse_packet(&bytes),
                DEM_FileHeader => self.parse_header(&bytes),
                DEM_FileInfo => self.parse_file_info(&bytes),
                DEM_SendTables => self.parse_classes(&bytes),
                DEM_ClassInfo => self.parse_class_info(&bytes),
                DEM_SignonPacket => self.parse_packet(&bytes),
                DEM_FullPacket => self.parse_full_packet(&bytes),
                DEM_UserCmd => self.parse_user_command_cmd(&bytes),
                DEM_StringTables => self.parse_stringtable_cmd(&bytes),
                DEM_Stop => break,
                _ => {},
            };
        }
    }
```
(slightly simplified from source code)

Depending on which command it is we decode the message accordingly:


### Dem_packet
DEM_Packet is the most common CMD. The DEM_Packet is decoded similarly to how the outer loop works:

1. Read what msg type the next bytes are.
2. Read how many bytes the next message is.

```Rust
let mut bitreader = Bitreader::new(&bytes);
// Inner loop
while bitreader.reader.bits_remaining().unwrap() > 8 {

    let msg_type = bitreader.read_u_bit_var().unwrap();
    let size = bitreader.read_varint().unwrap();
    let msg_bytes = bitreader.read_n_bytes(size as usize).unwrap();

    match netmessage_type_from_int(msg_type as i32) {
        svc_PacketEntities => self.parse_packet_ents(&msg_bytes),
        svc_CreateStringTable => self.parse_create_stringtable(&msg_bytes),
        svc_UpdateStringTable => self.update_string_table(&msg_bytes),
        GE_Source1LegacyGameEventList => self.parse_game_event_map(&msg_bytes),
        GE_Source1LegacyGameEvent => self.parse_event(&msg_bytes),
        // ... some extra left out
        _ => {},
    };
```
(slightly simplified from source code)

The main message types are:

- svc_CreateStringTable
- svc_UpdateStringTable
- GE_Source1LegacyGameEventList
- GE_Source1LegacyGameEvent
- svc_PacketEntities


#### Stringtables (both create and update)
Stringtables are messages that contains suprise surprise string data. Most of this data is not interesting for demo parsing, but baselines for entities pass trough here. These are the "default" values for entities. For example a players default health is 100?. 
"Userinfo" Also comes trough here, but with the source2 format it seems to be included in packet-entities and doesn't seem interesting anymore.

#### GE_Source1LegacyGameEventList
Includes needed data for parsing game events. This message should come before the first game event. It tells you how the game events should be decoded. If this message is lost then I think decoding game events is not possible?

#### GE_Source1LegacyGameEvent
Typical game events. Triggered when interesting things happen in the game, like when a weapon is fired or a round is over. Events are just key value pairs. These events can be parsed seperately from packet_entities (assuming you dont want to add extra custom values into the events).

#### svc_PacketEntities
The majority of data in the demo. All the data relating to entities. For example every players every coordinate, health, viewangles and you name it. If you can see a value in a replay then it probably comes from here.  

Something to note is that packet entities only send changes in values. So if a player is standing still then that value is not updated during that tick. This means that a value at tick 5000 may have been set at tick 3542 (when the player last moved) so you can't just parse the ticks that you are interested in because that packet only includes values that changed that tick.

This part combined with command "DEM_SendTables" are by far the most comlicated parts of the demo. Getting these right is many more times harder than the entire rest of demo parsing. If you want to try parsing the demo I would recommend just starting of with Game events and then move to these.

The DEM_SendTables creates blueprints for how to decode values in packet entities. The blueprints come in form of huge nested structs andso you have to traverse down these structs that is quite tricky. Mainly debugging becomes horrible here without proper tools.
