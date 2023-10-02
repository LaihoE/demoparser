from demoparser2 import DemoParser


parser = DemoParser("path_to_demo.dem")

# If you just want the names of all events then you can use this:
event_names = parser.list_game_events()

# Currently the event "all" gives you all events. Cursed solution for now
df = parser.parse_events(["all"])
