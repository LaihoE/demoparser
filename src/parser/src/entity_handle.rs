// Demo/network handles use 14 index bits and 10 serial bits. Native
// CEntityHandle values use 15 index bits and must not use this decoder.
const ENTITY_INDEX_MASK: u32 = 0x3FFF;
const INVALID_NETWORK_HANDLE: u32 = 0x00FF_FFFF;
pub(crate) const INVALID_ENTITY_ID: i32 = -1;

#[inline]
pub(crate) fn entity_handle_index(handle: u32) -> i32 {
    // Game events can also supply the signed -1 sentinel.
    if handle == INVALID_NETWORK_HANDLE || handle == u32::MAX {
        INVALID_ENTITY_ID
    } else {
        (handle & ENTITY_INDEX_MASK) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::first_pass::parser_settings::{FirstPassParser, ParserInputs};
    use crate::first_pass::prop_controller::PropInfo;
    use crate::second_pass::collect_data::PropType;
    use crate::second_pass::entities::{Entity, EntityType};
    use crate::second_pass::game_events::GameEventInfo;
    use crate::second_pass::parser_settings::SecondPassParser;
    use crate::second_pass::variants::Variant;
    use std::sync::Arc;

    #[test]
    fn decodes_handles_reported_in_issue_362() {
        assert_eq!(entity_handle_index(4_361_079), 2935);
        assert_eq!(entity_handle_index(6_275_616), 544);
    }

    #[test]
    fn keeps_index_boundaries_separate_from_invalid_handles() {
        for index in [0, 2047, 2048, 0x3FFE, 0x3FFF] {
            assert_eq!(entity_handle_index((1 << 14) | index), index as i32);
        }
        assert_eq!(entity_handle_index(INVALID_NETWORK_HANDLE), INVALID_ENTITY_ID);
        assert_eq!(entity_handle_index(u32::MAX), INVALID_ENTITY_ID);
    }

    #[test]
    fn resolves_player_weapon_and_event_props_from_network_handles() {
        let huffman = vec![];
        let settings = ParserInputs {
            real_name_to_og_name: Default::default(),
            wanted_players: vec![],
            wanted_player_props: vec![],
            wanted_other_props: vec![],
            wanted_prop_states: Default::default(),
            wanted_ticks: vec![],
            wanted_events: vec![],
            parse_ents: true,
            parse_projectiles: false,
            parse_grenades: false,
            only_header: false,
            only_convars: false,
            huffman_lookup_table: &huffman,
            order_by_steamid: false,
            list_props: false,
            fallback_bytes: None,
        };
        let mut first_pass = FirstPassParser::new(&settings);
        first_pass.cls_by_id = Some(Arc::new(vec![]));
        first_pass.prop_controller.special_ids.player_pawn = Some(1);
        first_pass.prop_controller.special_ids.steamid = Some(2);
        first_pass.prop_controller.special_ids.player_name = Some(3);
        first_pass.prop_controller.special_ids.active_weapon = Some(4);
        first_pass.prop_controller.special_ids.grenade_owner_id = Some(5);
        first_pass.prop_controller.prop_infos = vec![PropInfo {
            id: 6,
            prop_type: PropType::Player,
            prop_name: "CCSPlayerPawn.m_iHealth".to_string(),
            prop_friendly_name: "health".to_string(),
            is_player_prop: true,
        }];
        let mut parser = SecondPassParser::new(first_pass.create_first_pass_output().unwrap(), 0, false, None).unwrap();
        parser.entities.resize(0x4000, None);
        // A controller index above 2047 also exercises PlayerConnect's plain-index path.
        let controller_id = 4097;
        let weapon_id = 5000;
        parser.entities[weapon_id] = Some(Entity {
            cls_id: 0,
            entity_id: weapon_id as i32,
            props: [(7, Variant::U32(30))].into_iter().collect(),
            entity_type: EntityType::Normal,
        });
        for (handle, pawn_id) in [(4_361_079, 2935), (6_275_616, 544), (0x47FF, 2047), (0x7FFF, 16383)] {
            parser.players.clear();
            parser.entities[controller_id] = Some(Entity {
                cls_id: 0,
                entity_id: controller_id as i32,
                props: [
                    (1, Variant::U32(handle)),
                    (2, Variant::U64(76561198000000001)),
                    (3, Variant::String("player".to_string())),
                ]
                .into_iter()
                .collect(),
                entity_type: EntityType::PlayerController,
            });
            parser.entities[pawn_id as usize] = Some(Entity {
                cls_id: 0,
                entity_id: pawn_id,
                props: [
                    (4, Variant::U32((1 << 14) | weapon_id as u32)),
                    (5, Variant::U32(handle)),
                    (6, Variant::U32(73)),
                ]
                .into_iter()
                .collect(),
                entity_type: EntityType::Normal,
            });

            parser.gather_extra_info(&(controller_id as i32), false).unwrap();
            assert_eq!(parser.players.len(), 1);
            assert_eq!(parser.players[&pawn_id].controller_entid, Some(controller_id as i32));
            assert_eq!(parser.entity_id_from_user_pawn(handle as i32), Some(pawn_id));
            assert_eq!(parser.find_weapon_prop(&7, &pawn_id), Ok(Variant::U32(30)));
            assert_eq!(parser.grenade_owner_entid_from_grenade(&Some(Variant::I32(pawn_id))), Some(pawn_id));
            assert_eq!(
                parser.create_player_name_field(pawn_id, "user").data,
                Some(Variant::String("player".to_string()))
            );
            let fields = parser.find_extra_props_events(pawn_id, "user");
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].name, "user_health");
            assert_eq!(fields[0].data, Some(Variant::U32(73)));

            parser.players.clear();
            parser.emit_events(vec![GameEventInfo::PlayerConnect(controller_id as i32)]).unwrap();
            assert!(parser.players.contains_key(&pawn_id));
        }

        // Keep the valid player at index 16383: invalid handles must not alias it.
        for handle in [INVALID_NETWORK_HANDLE, u32::MAX] {
            let id = parser.entity_id_from_user_pawn(handle as i32).unwrap();
            assert_eq!(parser.create_player_name_field(id, "user").data, None);
            assert_eq!(parser.find_extra_props_events(id, "user")[0].data, None);
            parser.players.clear();
            parser.entities[controller_id].as_mut().unwrap().props.insert(1, Variant::U32(handle));
            parser.gather_extra_info(&(controller_id as i32), false).unwrap();
            assert!(parser.players.is_empty());
            parser.emit_events(vec![GameEventInfo::PlayerConnect(controller_id as i32)]).unwrap();
            assert!(parser.players.is_empty());
        }
    }
}
