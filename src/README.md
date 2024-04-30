# stop! please read me first :)


If you are planning to read the rust source I would recommend starting with these 2 files: 

parser/src/entities.rs  
parser/src/collect_data.rs  

Most of the interesting stuff happens here. For example ALL new decoded values pass through this function: "decode_entity_update"

After that I would look at the 2 passes through the file.

The parser does 2 passes through the file:

1st pass: (parser/src/parser.rs): Fast pass through the file, checks where packets start and end in prep for 2nd pass. Also parses 1-off things like descriptors for entity data and game events.

2nd pass (parser/src/parser_thread.rs): Multi-threaded parsing of the majority of data.

And finally you could take a peek at the game events (parser/src/game_events.rs)

The rest of the files are in some way support to the above mentioned files and are likely not too interesting to look at without deeper understanding of the demo format.