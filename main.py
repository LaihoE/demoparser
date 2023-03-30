import re

#name = "CCSPlayerController_InventoryServices*"
name = "CNetworkUtlVectorBase< CHandle< CBaseModelEntity > >"
#name = "char[161]"

itemCounts = {"MAX_ITEM_STOCKS":             8,
                      "MAX_ABILITY_DRAFT_ABILITIES": 48
                      }

p = re.compile('([^\<\[\*]+)(\<\s(.*)\s\>)?(\*)?(\[(.*)\])?')
searches = p.search(name)

ss = searches.groups()
print("groups",ss)

print(ss[1])

base_type = ss[0]
pointer = ss[3] == "*"
generic_type = None
count = 0

if ss[2] != None:
    print("Generic",ss[2])
    #generic_type = FieldType(name=ss[2])


if ss[5] in itemCounts:
    count = itemCounts[ss[5]]
elif ss[5] != None:
    if int(ss[5]) > 0:
        count = int(ss[5])
    else:
        count = 1024



print(base_type)
print(pointer)
print(count)