use jni::JNIEnv;
use jni::objects::{JClass, JIntArray, JByteArray, JObject};
use jni::sys::{jint, jlong, jboolean};
use std::sync::{Arc, RwLock};

pub struct MapCache {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<TileData>,
}

#[derive(Clone, Default)]
pub struct TileData {
    pub is_water: bool,
    pub is_ocean: bool,
    pub is_mountain: bool,
    pub is_city_center: bool,
    pub road_status: u8,
    pub owner_id: i32,
    pub military_unit_owner_id: i32,
    pub civilian_unit_owner_id: i32,
    pub military_unit_id: i32,
    pub civilian_unit_id: i32,
    pub neighbors: [usize; 6],
}

impl MapCache {
    pub fn new(width: i32, height: i32, tile_count: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![TileData::default(); tile_count],
        }
    }
}

// Global state or returned pointer handle. We will return a jlong pointer to Kotlin.
#[no_mangle]
pub extern "system" fn Java_com_unciv_logic_NativeBridge_createMapCache(
    mut env: JNIEnv,
    _class: JClass,
    width: jint,
    height: jint,
    tile_count: jint,
) -> jlong {
    let result = std::panic::catch_unwind(|| {
        let cache = Box::new(MapCache::new(width, height, tile_count as usize));
        Box::into_raw(cache) as jlong
    });
    
    match result {
        Ok(ptr) => ptr,
        Err(_) => {
            let _ = env.throw_new("java/lang/RuntimeException", "Rust panic in createMapCache!");
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_unciv_logic_NativeBridge_destroyMapCache(
    mut _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    if ptr == 0 { return; }
    let _ = std::panic::catch_unwind(|| {
        unsafe {
            let _ = Box::from_raw(ptr as *mut MapCache);
        }
    });
}

// Update a single tile's static and dynamic state
#[no_mangle]
pub extern "system" fn Java_com_unciv_logic_NativeBridge_updateTile(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    index: jint,
    is_water: jboolean,
    is_ocean: jboolean,
    is_mountain: jboolean,
    is_city_center: jboolean,
    road_status: jint,
    owner_id: jint,
    military_unit_owner_id: jint,
    civilian_unit_owner_id: jint,
    military_unit_id: jint,
    civilian_unit_id: jint,
) {
    let result = std::panic::catch_unwind(|| {
        if ptr == 0 { return; }
        let cache = unsafe { &mut *(ptr as *mut MapCache) };
        if let Some(tile) = cache.tiles.get_mut(index as usize) {
            tile.is_water = is_water != 0;
            tile.is_ocean = is_ocean != 0;
            tile.is_mountain = is_mountain != 0;
            tile.is_city_center = is_city_center != 0;
            tile.road_status = road_status as u8;
            tile.owner_id = owner_id;
            tile.military_unit_owner_id = military_unit_owner_id;
            tile.civilian_unit_owner_id = civilian_unit_owner_id;
            tile.military_unit_id = military_unit_id;
            tile.civilian_unit_id = civilian_unit_id;
        }
    });

    if result.is_err() {
        let _ = env.throw_new("java/lang/RuntimeException", "Rust panic in updateTile!");
    }
}

#[no_mangle]
pub extern "system" fn Java_com_unciv_logic_NativeBridge_setTileNeighbors(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    index: jint,
    n0: jint,
    n1: jint,
    n2: jint,
    n3: jint,
    n4: jint,
    n5: jint,
) {
    let result = std::panic::catch_unwind(|| {
        if ptr == 0 { return; }
        let cache = unsafe { &mut *(ptr as *mut MapCache) };
        if let Some(tile) = cache.tiles.get_mut(index as usize) {
            tile.neighbors = [
                if n0 < 0 { usize::MAX } else { n0 as usize },
                if n1 < 0 { usize::MAX } else { n1 as usize },
                if n2 < 0 { usize::MAX } else { n2 as usize },
                if n3 < 0 { usize::MAX } else { n3 as usize },
                if n4 < 0 { usize::MAX } else { n4 as usize },
                if n5 < 0 { usize::MAX } else { n5 as usize },
            ];
        }
    });

    if result.is_err() {
        let _ = env.throw_new("java/lang/RuntimeException", "Rust panic in setTileNeighbors!");
    }
}


#[no_mangle]
pub extern "system" fn Java_com_unciv_logic_NativeBridge_getTilesInDistance(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    center_index: jint,
    max_distance: jint,
    output_array: JIntArray,
) -> jint {
    let result = std::panic::catch_unwind(|| {
        if ptr == 0 { return 0; }
        if max_distance < 0 { return 0; }
        
        let cache = unsafe { &*(ptr as *const MapCache) };
        let center = center_index as usize;
        
        if center >= cache.tiles.len() { return 0; }

        // BFS setup
        let mut visited = vec![false; cache.tiles.len()];
        let mut queue = std::collections::VecDeque::new();
        let mut result_indices = Vec::new();
        
        visited[center] = true;
        queue.push_back((center, 0));
        
        while let Some((curr, dist)) = queue.pop_front() {
            // Unciv getTilesInDistance usually includes the center if distance >= 0
            result_indices.push(curr as i32);
            
            if dist < max_distance {
                let tile = &cache.tiles[curr];
                for &n in &tile.neighbors {
                    if n < cache.tiles.len() && !visited[n] {
                        visited[n] = true;
                        queue.push_back((n, dist + 1));
                    }
                }
            }
        }
        
        // Write back to Java array
        let len_to_write = std::cmp::min(result_indices.len(), env.get_array_length(&output_array).unwrap_or(0) as usize);
        if len_to_write > 0 {
            let _ = env.set_int_array_region(&output_array, 0, &result_indices[0..len_to_write]);
        }
        
        len_to_write as jint
    });

    match result {
        Ok(count) => count,
        Err(_) => {
            let _ = env.throw_new("java/lang/RuntimeException", "Rust panic in getTilesInDistance!");
            0
        }
    }
}



#[no_mangle]
pub extern "system" fn Java_com_unciv_logic_NativeBridge_getPotentialAttackTargets(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    my_civ_id: jint,
    reachable_indices: JIntArray,
    attack_range: jint,
    output_array: JIntArray,
) -> jint {
    let result = std::panic::catch_unwind(|| {
        if ptr == 0 { return 0; }
        
        let cache = unsafe { &*(ptr as *const MapCache) };
        
        let reachable_len = env.get_array_length(&reachable_indices).unwrap_or(0);
        if reachable_len <= 0 { return 0; }
        
        let mut starting_nodes = vec![0i32; reachable_len as usize];
        if env.get_int_array_region(&reachable_indices, 0, &mut starting_nodes).is_err() {
            return 0;
        }

        // We use a generation-based visited array to avoid reallocation if we did it repeatedly,
        // but a simple boolean vec is extremely fast.
        let mut visited_dist = vec![i32::MAX; cache.tiles.len()];
        let mut queue = std::collections::VecDeque::new();
        
        for &idx in &starting_nodes {
            let uidx = idx as usize;
            if uidx < cache.tiles.len() {
                visited_dist[uidx] = 0;
                queue.push_back((uidx, 0));
            }
        }
        
        let mut targets = Vec::new();
        let mut targets_set = vec![false; cache.tiles.len()];

        while let Some((curr, dist)) = queue.pop_front() {
            let tile = &cache.tiles[curr];
            
            // If it has a foreign unit, it is a potential target
            let has_foreign_military = tile.military_unit_id != 0 && tile.military_unit_owner_id != my_civ_id;
            let has_foreign_civilian = tile.civilian_unit_id != 0 && tile.civilian_unit_owner_id != my_civ_id;
            let has_foreign_city = tile.is_city_center && tile.owner_id != -1 && tile.owner_id != my_civ_id;

            if has_foreign_military || has_foreign_civilian || has_foreign_city {
                if !targets_set[curr] {
                    targets_set[curr] = true;
                    targets.push(curr as i32);
                }
            }
            
            if dist < attack_range {
                for &n in &tile.neighbors {
                    if n < cache.tiles.len() {
                        if dist + 1 < visited_dist[n] {
                            visited_dist[n] = dist + 1;
                            queue.push_back((n, dist + 1));
                        }
                    }
                }
            }
        }
        
        let out_len = env.get_array_length(&output_array).unwrap_or(0) as usize;
        let len_to_write = std::cmp::min(targets.len(), out_len);
        if len_to_write > 0 {
            let _ = env.set_int_array_region(&output_array, 0, &targets[0..len_to_write]);
        }
        
        len_to_write as jint
    });

    match result {
        Ok(count) => count,
        Err(_) => {
            let _ = env.throw_new("java/lang/RuntimeException", "Rust panic in getPotentialAttackTargets!");
            0
        }
    }
}

