# The return type in JS is a string so you will need to convert it 
# to some kind of int before applying this logic. This is due to the lack
# of a "u64" or a "long long int" in js.

KEY_MAPPING = {
    "IN_ATTACK": 1 << 0,
    "IN_JUMP": 1 << 1,
    "IN_DUCK": 1 << 2,
    "IN_FORWARD": 1 << 3,
    "IN_BACK": 1 << 4,
    "IN_USE": 1 << 5, 
    "IN_CANCEL": 1 << 6,
    "IN_TURNLEFT": 1 << 7,
    "IN_TURNRIGHT": 1 << 8,
    "IN_MOVELEFT": 1 << 9,
    "IN_MOVERIGHT": 1 << 10,
    "IN_ATTACK2": 1 << 11,
    "IN_RELOAD": 1 << 13,
    "IN_ALT1": 1 << 14,
    "IN_ALT2": 1 << 15,
    "IN_SPEED": 1 << 16,
    "IN_WALK": 1 << 17,
    "IN_ZOOM": 1 << 18,
    "IN_WEAPON1": 1 << 19,
    "IN_WEAPON2": 1 << 20,
    "IN_BULLRUSH": 1 << 21,
    "IN_GRENADE1": 1 << 22,
    "IN_GRENADE2": 1 << 23,
    "IN_ATTACK3": 1 << 24,
    "UNKNOWN_25": 1 << 25,
    "UNKNOWN_26": 1 << 26,
    "UNKNOWN_27": 1 << 27,
    "UNKNOWN_28": 1 << 28,
    "UNKNOWN_29": 1 << 29,
    "UNKNOWN_30": 1 << 30,
    "UNKNOWN_31": 1 << 31,
    "IN_SCORE": 1 << 33,
    "IN_INSPECT": 1 << 35,
}


def extract_buttons(buttons: int):
    # The value you get out of the "buttons" field is a bitfield where
    # each bit in the value represents a boolean for each possible keypress
    # For example the value:

    # 5 = 0b0000000000000000000000000000101
    # where the first and the third bits (from the right) are set.

    # Now we check our map and see that first bit stands for the IN_ATTACK key and third bit is IN_DUCK
    # 0b0000000000000000000000000000101 = [IN_ATTACK, IN_DUCK]

    # This is a very efficient way to represent the state of all keys.
    # Can also be used to quickly check multiple inventory items with
    # more complicated bitwise operations.

    all_keys_as_str = []

    for (button_name, bit_slot) in KEY_MAPPING.items():
        if (buttons & bit_slot) != 0:    # Check if nth bit is set
            all_keys_as_str.append(button_name)

    return all_keys_as_str


keys = extract_buttons(5)
print(keys)