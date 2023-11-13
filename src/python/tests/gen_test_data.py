import pickle
from demoparser2 import DemoParser



def gen_prop_test(prop):
    parser = DemoParser("tests/data/test.dem")
    df = parser.parse_ticks([prop], ticks=[x for x in range(100000) if x % 100 == 0])
    df.to_parquet(f"tests/data/per_prop/{prop}.parquet")


    print(f'def test_{prop}(self):')
    print('    parser = DemoParser("tests/data/test.dem")')
    print(f'    df = parser.parse_ticks(["{prop}"], ticks=[x for x in range(100000) if x % 100 == 0])')
    print(f'    df_correct = pd.read_parquet("tests/data/per_prop/{prop}.parquet")')
    print('    assert_frame_equal(df, df_correct)')
    print()

def gen_event_test(event_name):
    parser = DemoParser("tests/data/test.dem")
    df = parser.parse_event(event_name)
    df.to_parquet(f"tests/data/per_event/{event_name}.parquet")

    print(f'def test_{event_name}(self):')
    print('    parser = DemoParser("tests/data/test.dem")')
    print(f'    df = parser.parse_event("{event_name}")')
    print(f'    df_correct = pd.read_parquet("tests/data/per_event/{event_name}.parquet")')
    print('    assert_frame_equal(df, df_correct)')
    print()

def gen_event_with_props():
    parser = DemoParser("tests/data/test.dem")
    df = parser.parse_event("player_death", player=["X", "Y"], other=["game_time", "total_rounds_played"])
    df.to_parquet(f"tests/data/event_with_props.parquet")

def gen_events_with_props():
    parser = DemoParser("tests/data/test.dem")
    events_list = parser.parse_events(["all"], player=["X", "Y"], other=["game_time", "total_rounds_played"])
    with open('tests/data/events_with_props.pickle', 'wb') as fp:
        pickle.dump(events_list, fp)

def gen_event_with_prop_tests():
    gen_event_with_props()
    gen_events_with_props()


# If event changes regenerate test with this
"""wanted_events = ["item_pickup", "item_equip" ,"player_spawn"]
for event in wanted_events:
    gen_event_test(event)

# If prop changes regenerate test with this
wanted_prop = "is_alive"
gen_prop_test(wanted_prop)"""

wanted_props = ["velocity", "velocity_X", "velocity_Y", "velocity_Z"]
for prop in wanted_props:
    gen_prop_test(prop)

# gen_event_with_prop_tests()