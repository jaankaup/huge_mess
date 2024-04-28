use rand::Rng;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::mem::transmute;
use engine::misc::index_to_uvec3;
use engine::misc::uvec3_to_index;

use engine::gpu_debugger::primitive_processor::AABB;

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
pub enum WfcNode {
    Known(u32),
    Band(Vec<u32>),
    Far,
    // Invalid,
}

#[derive(Clone)]
pub struct WfcScene {
    dim_x: u32,
    dim_y: u32,
    dim_z: u32,
    scene_data: Vec<WfcNode>,   
    // band: BinaryHeap<Reverse<(u32, u32)>>,
    band: Vec<(u32, u32)>,
    all_block_cases: Vec<WfcBlock>,
    temp_aabbs: Vec<AABB>,
}

impl WfcScene {
    pub fn init(dim_x: u32, dim_y: u32, dim_z: u32) -> Self {
        Self {
            dim_x: dim_x,
            dim_y: dim_y,
            dim_z: dim_z,
            scene_data: vec![WfcNode::Far; (dim_x * dim_y * dim_x).try_into().unwrap()],   
            band: Vec::<(u32, u32)>::with_capacity(1000),
            // band: BinaryHeap::new(),
            all_block_cases: Vec::new(),
            temp_aabbs: Vec::new(),
        }
    }

    /// Insert a unique wfc block.
    pub fn insert_block_case(&mut self, mut block: WfcBlock) {
        block.id = self.all_block_cases.len() as u32;
        self.all_block_cases.push(block);
    }

    pub fn get_aabb_data(&mut self) -> Vec<AABB> {
        let aabbs = self.temp_aabbs.clone();
        self.temp_aabbs.clear();
        aabbs
    }

    /// Add a block to scene.
    pub fn add_seed_point(&mut self, block_id: u32, coord: [u32; 3]) {

        assert!(coord[0] < self.dim_x && coord[1] < self.dim_y && coord[2] < self.dim_z);
        assert!(self.band.len() == 0);
        assert!(block_id < self.all_block_cases.len().try_into().unwrap());

        self.scene_data[uvec3_to_index(coord[0], coord[1], coord[2], self.dim_x, self.dim_y) as usize] = WfcNode::Known(block_id); 
        let conn_data = self.all_block_cases[block_id as usize].get_connection_data();
        self.debug_aabb(block_id, coord, &conn_data.clone());
    }

    pub fn find_neighbor_indices_ind(&mut self, index: u32) -> [Option<u32>; 6] {
        // VALIDATION!
        self.find_neighbor_indices_coord(index_to_uvec3(index, self.dim_x, self.dim_y))
    }

    pub fn find_neighbor_indices_coord(&mut self, coord: [u32 ; 3]) -> [Option<u32>; 6] {

        let ref_to_node = &self.scene_data[uvec3_to_index(coord[0], coord[1], coord[2], self.dim_x, self.dim_y) as usize];
        // assert!(ref_to_node);

        // Find all neighbor nodes;
        let mut neighbors: [Option<u32> ; 6] = [None ; 6];

        const directions: [[i32;3] ; 6] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]]; 

        // Neighbors.
        for (i,d) in directions.iter().enumerate() {
            let actual_index = [coord[0] as i32 + d[0], coord[1] as i32 + d[1], coord[2] as i32 + d[2]];   
            if actual_index[0] >= 0 && actual_index[0] < self.dim_x.try_into().unwrap() &&
                actual_index[1] >= 0 && actual_index[1] < self.dim_y.try_into().unwrap() &&
                    actual_index[2] >= 0 && actual_index[2] < self.dim_z.try_into().unwrap() {
                        neighbors[i] = Some(uvec3_to_index(
                                                actual_index[0].try_into().unwrap(),
                                                actual_index[1].try_into().unwrap(),
                                                actual_index[2].try_into().unwrap(),
                                                self.dim_x,
                                                self.dim_y));
            }
        }

        neighbors
    }

    pub fn expand_band_uvec3(&mut self, center_coord: [u32 ; 3]) {
        // VALIDATION!
        let center_index = uvec3_to_index(center_coord[0], center_coord[1], center_coord[2], self.dim_x, self.dim_y);   
        self.expand_band(center_index);
    }

    pub fn update_band_node(&mut self, ind: u32) {

        // VALIDATION!
        let neighbor_indices = self.find_neighbor_indices_ind(ind);
        // println!("neighbor_indices : {:?}", neighbor_indices);
        // let mut possible_cases = Vec::<u32>::new();

        // Store case number as key, and the key count as value.
        let mut possible_matches = HashMap::<u32, u32>::new();
        let mut known_neighbor_count = 0;

        for (direction_index, ni) in neighbor_indices.iter().enumerate() {
            // Kauppinen
            if ni.is_some() {
                let scene_data_ref = &mut self.scene_data[ni.unwrap() as usize];
                match *scene_data_ref {
                    WfcNode::Known(block_id) => {

                        known_neighbor_count += 1;
                        for x in self.all_block_cases.iter() {
                            // println!("all_block_cases : {:?}", self.all_block_cases.len());
                            // Check for all possible cases and this known match. 
                            if x.matches(&self.all_block_cases[block_id as usize], direction_index + 1) {
                                possible_matches.entry(x.get_id()).and_modify(|b_count| *b_count += 1).or_insert(1);
                            }
                        }
                        // self.all_block_cases[block_id].
                        //known_neighbor_refs.push(scene_data_ref);
                    }
                    _ => {}
                }
            }
        }
        // Get the intersection of the all possible cases.
        // println!("Possible matches:");
        // println!("{:?}", possible_matches);
        let the_vec_of_cases: Vec<u32> =
             possible_matches
             .iter()
             .filter(|(k,v)| **v == known_neighbor_count)
             .map(|(k,_)| *k)
             .collect::<Vec<_>>(); 
        
        // println!("Actual possible matches: {:?}", the_vec_of_cases);
        // Does this alredy exists of band..
        if let Some(pos) = self.band.iter().position(|(_, id)| *id == ind) {
            self.band[pos as usize] = (the_vec_of_cases.len().try_into().unwrap(), ind);    
        }
        // Add a new band node.
        else {
            self.band.push((the_vec_of_cases.len().try_into().unwrap(), ind));
        }
        self.scene_data[ind as usize] = WfcNode::Band(the_vec_of_cases); 
    }

    /// Expand band set from given center index.
    pub fn expand_band(&mut self, center_index: u32) {
        let coordinate = index_to_uvec3(center_index, self.dim_x, self.dim_y);   
        let neighbor_indices = self.find_neighbor_indices_coord(coordinate);
        for ind in neighbor_indices.iter() {
            if ind.is_some() {
            //println!("Find neigbors :: {:?}", ind.unwrap());
                let neighbor_ref = &mut self.scene_data[ind.unwrap() as usize];
                match *neighbor_ref {
                    WfcNode::Known(_) => {},
                    WfcNode::Band(_) => {
                        // println!("Found band :: {:?}", ind.unwrap());
                        // Update case count.
                        self.update_band_node(ind.unwrap());
                    },
                    WfcNode::Far => {

                        *neighbor_ref = WfcNode::Band(Vec::new());
                        // println!("Found far :: {:?}", ind.unwrap());
                        self.update_band_node(ind.unwrap());

                        // Update case count.

                        // Debugging.
                        //+0 let n_coordinate = index_to_uvec3(ind.unwrap(), self.dim_x, self.dim_y);
                        //+0 let base_position = [n_coordinate[0] as f32 * 5.0, n_coordinate[1] as f32 * 5.0, n_coordinate[2] as f32 * 5.0];
                        //+0 let color: f32 = unsafe {transmute::<u32, f32>(0x0000FFFF)};
                        //+0 self.temp_aabbs.push(
                        //+0     AABB {
                        //+0         min: [base_position[0],
                        //+0               base_position[1],
                        //+0               base_position[2],
                        //+0               color],
                        //+0         max: [base_position[0] + 1.0,
                        //+0               base_position[1] + 1.0,
                        //+0               base_position[2] + 1.0,
                        //+0               color],
                        //+0     });

                    },
                }
            }
        }
        // self.band.sort();
        // println!("Sorted band {:?}", self.band);
    }

    pub fn make_known(&mut self, index: u32) {
        // Validation
        // println!("BAND :: {:?}", self.band);
        let node_ref = &mut self.scene_data[index as usize];
        let mut r: Option<u32> = None;
        let new_node_block_id = match node_ref {
            WfcNode::Band(candidates) => {
                let mut rng = rand::thread_rng();
                r = Some(rng.gen_range(0..candidates.len()).try_into().unwrap());
                // println!("candidates.len() = {:?}", candidates.len());
                //let r: u32 = rng.gen_range(0..candidates.len()).try_into().unwrap();
                candidates[r.unwrap() as usize]
            },
            _ => { panic!("Not a band node.") }
        };
        // Add new known.
        self.scene_data[index as usize] = WfcNode::Known(new_node_block_id);
        // Kauppinen
        self.debug_aabb(new_node_block_id, index_to_uvec3(index, self.dim_x, self.dim_y), &self.all_block_cases[new_node_block_id as usize].get_connection_data().clone());
        // Delete from the band.
        
        // println!("self.band == {:?}", self.band);
        // println!("r == {:?}", r);
        // if r.is_some() {
            let delete_index = self.band.iter().position(|x| x.1 == index).unwrap(); 
            self.band.remove(delete_index.try_into().unwrap());
            // self.band.remove(index.try_into().unwrap());
        // }

    }

    pub fn find_next_known_candidates(&mut self) -> Option<Vec<u32>> {

        if self.band.len() == 0 { return None; }

        let mut result = Vec::new();
        self.band.sort();
        let mut smallest_case_count = 0;
        for v in self.band.iter() {
            if v.0 == 0 { continue; }
            // This is the smallest case number that matters.
            // Add to result and update the smalles_case_count.
            else if smallest_case_count == 0 && v.0 > 0 {
                smallest_case_count = v.0;
                result.push(v.1); 
            }
            // This is the smallest case number that matters.
            else if smallest_case_count == v.0 {
                result.push(v.1); 
            }
            // The smallest cases all already picked up.
            else if smallest_case_count < v.0 {
                break;
            }
        }
        Some(result)
    }
    
    pub fn debug_aabb(&mut self, block_id: u32, coord: [u32 ; 3], conn_data: &Vec<[f32; 3]>) {

        // For debugging reasons.
        let block_size = self.all_block_cases[block_id as usize].get_dimension();
        let base_position = [coord[0] as f32 * 0.8 * block_size as f32,
                             coord[1] as f32 * 0.8 * block_size as f32,
                             coord[2] as f32 * 0.8 * block_size as f32];
        let color: f32 = unsafe {transmute::<u32, f32>(0x0F00FFFF)};
        for x in conn_data.iter() {
            self.temp_aabbs.push(
                AABB {
                    min: [(base_position[0] + x[0] as f32) * 0.8,
                          (base_position[1] + x[1] as f32) * 0.8,
                          (base_position[2] + x[2] as f32) * 0.8,
                          color],
                    max: [(base_position[0] + x[0] as f32 + 1.0) * 0.8, // TODO: not 1.0, a parameter
                          (base_position[1] + x[1] as f32 + 1.0) * 0.8, // TODO: not 1.0, a parameter
                          (base_position[2] + x[2] as f32 + 1.0) * 0.8, // TODO: not 1.0, a parameter
                          color],
                });
        }
    }
}

pub fn test_data(color: u32) -> Vec<[f32 ; 4]> {

    let mut result = Vec::new();

    let the_color: f32 = unsafe {transmute::<u32, f32>(color)};

    for i in -2..2 {
        for k in -2..3 {
            result.push([i as f32, -2.0, k as f32, the_color]);
        }
    }
    for j in -2..3 {
        for k in -2..3 {
            result.push([2.0, j as f32, k as f32, the_color]);
        }
    }
    // result.push([0.0, 0.0, 0.0, the_color]);
    // result.push([1.0, 0.0, 0.0, the_color]);
    // result.push([2.0, 0.0, 0.0, the_color]);
    result 
}

pub fn test_data_v3() -> Vec<[f32 ; 3]> {

    let mut result = Vec::new();


    for i in -2..2 {
        for k in -2..3 {
            result.push([i as f32, -2.0, k as f32]);
        }
    }
    for j in -2..3 {
        for k in -2..3 {
            result.push([2.0, j as f32, k as f32]);
        }
    }
    // result.push([0.0, 0.0, 0.0]);
    // result.push([1.0, 0.0, 0.0]);
    // result.push([2.0, 0.0, 0.0]);
    result 
}

pub fn test_data_floor() -> Vec<[f32 ; 3]> {
    
    let mut result = Vec::new();

    for i in -2..3 {

        for k in -2..3 {
            result.push([i as f32, -2.0, k as f32]);
        }
    }
    result
}

pub fn test_data_floor_corner() -> Vec<[f32 ; 3]> {
    
    let mut result = Vec::new();

    for i in -2..3 {

        for k in -2..3 {
            result.push([i as f32, -2.0, k as f32]);
        }
    }

    for j in -1..3 {
        result.push([2.0, j as f32, -2.0]);
    }
    result
}

pub fn test_data_floor_corner_2() -> Vec<[f32 ; 3]> {
    
    let mut result = Vec::new();

    for i in -2..3 {

        for k in -2..3 {
            result.push([i as f32, -2.0, k as f32]);
        }
    }

    for j in -1..3 {
        result.push([2.0, j as f32, -2.0]);
    }
    for j in -1..3 {
        result.push([-2.0, j as f32, -2.0]);
    }
    result
}

pub fn test_data_floor_corner_3() -> Vec<[f32 ; 3]> {
    
    let mut result = Vec::new();

    for i in -2..3 {

        for k in -2..3 {
            result.push([i as f32, -2.0, k as f32]);
        }
    }

    for j in -1..3 {
        result.push([2.0, j as f32, -2.0]);
    }
    for j in -1..3 {
        result.push([-2.0, j as f32, -2.0]);
    }
    for j in -1..3 {
        result.push([-2.0, j as f32, 2.0]);
    }
    result
}

pub fn test_data_ceiling() -> Vec<[f32 ; 3]> {
    
    let mut result = Vec::new();

    for j in -2..3 {
        for k in -2..3 {
            result.push([2.0, j as f32, k as f32]);
        }
    }
    result
}

pub fn corner() -> Vec<[f32 ; 3]> {
    
    let mut result = Vec::new();

    for j in -2..3 {
        result.push([2.0, j as f32, -2.0]);
    }
    result
}

pub fn corner2() -> Vec<[f32 ; 3]> {
    
    let mut result = Vec::new();

    for j in -2..3 {
        result.push([2.0, j as f32, -2.0]);
    }
    for j in -2..3 {
        result.push([-2.0, j as f32, -2.0]);
    }
    result
}

pub fn test_data_ceiling_corner() -> Vec<[f32 ; 3]> {
    
    let mut result = Vec::new();

    for j in -2..3 {
        for k in -2..3 {
            result.push([2.0, j as f32, k as f32]);
        }
    }
    for j in -1..3 {
        result.push([2.0, j as f32, -2.0]);
    }
    result
}

pub fn test_data_ceiling_corner_2() -> Vec<[f32 ; 3]> {
    
    let mut result = Vec::new();

    for j in -2..3 {
        for k in -2..3 {
            result.push([2.0, j as f32, k as f32]);
        }
    }
    for j in -1..3 {
        result.push([2.0, j as f32, -2.0]);
    }
    for j in -1..3 {
        result.push([2.0, j as f32, 2.0]);
    }
    result
}

pub fn test_data_2x_ceiling_floor() -> Vec<[f32 ; 3]> {
    
    let mut result = Vec::new();

    for j in -2..2 {
        for k in -2..2 {
            result.push([2.0, j as f32, k as f32]);
        }
    }
    for i in -2..2 {
        for j in -2..2 {
            result.push([i as f32, j as f32, -2.0]);
        }
    }
    for j in -2..3 {
        for k in -2..3 {
            result.push([2.0, j as f32, k as f32]);
        }
    }
    result
}

pub fn test_data_empty() -> Vec<[f32 ; 3]> {
    vec![]
}

/// A basic wfc block.
#[derive(Clone, Debug)]
pub struct Voxel {
    pub id: u32,       // A unique voxel id.
    pub cases: u32,    // All legal rotations for this
    pub dimension: u32,// The dimension or the voxel.
    pub weight: f32,   // For future usage: possibility weight
    // pub rotation: u32, // The current rotation of the voxel
    pub connection_data: Vec<[f32 ; 4]>, // Used to match voxels together.
    pub visual_data: Vec<[f32 ; 4]>,     // The actual scene object.
    pub possible_neighbors: HashMap<u32, [u32 ; 6]>, // id, cases. Information about all possible neigbors and rotations.
}

impl Voxel {
    pub fn init(id: u32,
                cases: u32,
                dimension: u32,
                weight: f32,
                connection_data: &Vec<[f32; 4]>,
                visual_data: &Vec<[f32 ; 4]>) -> Self {

        assert!(cases <= 0b1111111111);
        assert!(weight <= 1.0 && weight >= 0.0);

        Self {
            id: id,
            cases: cases,
            dimension: dimension,
            weight: weight,
            connection_data: connection_data.clone(),
            visual_data: visual_data.clone(),
            possible_neighbors: HashMap::new(),
        }
    }

    pub fn add_rules(&mut self, voxel: &Voxel) {
        assert!(self.dimension == voxel.dimension);
        let connections = check_connections(&self.connection_data, &voxel.connection_data, voxel.cases); 
        self.possible_neighbors.insert(voxel.id, connections);
    }

    pub fn get_rotated_connection_data(&self, rotation: u32) -> Option<Vec<[f32 ; 4]>> {

        // Only one rotation allowed!!!!! No checking yet.
        if rotation == 0 { None }
        else { 
            Some(create_rotations(rotation, &self.connection_data)[0].clone())
        }
    }

    pub fn get_possible_neighbors(&self, current_rotation: u32) -> HashMap<u32, [u32 ; 6]> {
        let mut result = HashMap::new();
        for (k, v) in self.possible_neighbors.iter() {
            if current_rotation & 1        != 0 { result.insert(*k, *v);}
            else if current_rotation & 2   != 0 { result.insert(*k, [v[0], v[1], v[5], v[4], v[2], v[3]]); } //ro90x
            else if current_rotation & 4   != 0 { result.insert(*k, [v[0], v[1], v[3], v[2], v[5], v[4]]); } // ro180x
            else if current_rotation & 8   != 0 { result.insert(*k, [v[0], v[1], v[4], v[5], v[3], v[2]]); } // ro270x
            else if current_rotation & 16  != 0 { result.insert(*k, [v[4], v[5], v[2], v[3], v[1], v[0]]); } // ro90y
            else if current_rotation & 32  != 0 { result.insert(*k, [v[1], v[0], v[2], v[3], v[5], v[4]]); } // ro180y
            else if current_rotation & 64  != 0 { result.insert(*k, [v[5], v[4], v[2], v[3], v[0], v[1]]); } // ro270y
            else if current_rotation & 128 != 0 { result.insert(*k, [v[3], v[2], v[0], v[1], v[4], v[5]]); } // ro90z
            else if current_rotation & 256 != 0 { result.insert(*k, [v[1], v[0], v[3], v[2], v[4], v[5]]); } // ro180z
            else if current_rotation & 512 != 0 { result.insert(*k, [v[2], v[3], v[1], v[0], v[4], v[5]]); } // ro270z
            else { panic!("current_rotation not supported.") }
        }
        // Check that neighbor rotations are legal.
        result
    }

    // Get all possible rotations
    pub fn get_all_rotations(&self) -> Vec<Vec<[f32; 4]>> {
        let mut result: Vec<Vec<[f32; 4]>> = Vec::new(); 
        for i in 0..13 {
            result.push(create_rotations(1 << i, &self.connection_data)[0].clone());
        }
        // println!("All rotations {:?}", result);
        result
    }
}

/// A enum for each possible rotation and reflection.
pub enum Rotation {
    Identity,
    R90X,
    R180X,
    R270X,
    R90Y,
    R180Y,
    R270Y,
    R90Z,
    R180Z,
    R270Z,
    ReflectionX,
    ReflectionY,
    ReflectionZ,
}

/// Generate case number.
pub fn create_rotation_cases(cases: &Vec<Rotation>) -> u32 {
    let mut result: u32 = 0;
    for c in cases.iter() {
        match c {
            Rotation::Identity => { result |= 1; },
            Rotation::R90X     => { result |= 2; },
            Rotation::R180X    => { result |= 4; },
            Rotation::R270X    => { result |= 8; },
            Rotation::R90Y     => { result |= 16; },
            Rotation::R180Y    => { result |= 32; },
            Rotation::R270Y    => { result |= 64; },
            Rotation::R90Z     => { result |= 128; },
            Rotation::R180Z    => { result |= 256; },
            Rotation::R270Z    => { result |= 512; },
            Rotation::ReflectionX => { result |= 1024; },
            Rotation::ReflectionY => { result |= 2048; },
            Rotation::ReflectionZ => { result |= 4096; },
        }
    }
    result
}

//    0    1    2    3    4    5    6    7    8    9     10   11   12 
//  +----+-----+-----+----+-----+-----+----+-----+-----+----+----+----+
//  |r90x|r180x|r270x|r90y|r180y|r270y|r90z|r180z|r270z| rx | ry | rx |
//  +----+-----+-----+----+-----+-----+----+-----+-----+----+----+----+

#[inline]
fn round_half_up(value: f32) -> f32 {
   (((2.0 * value).floor())*0.5).ceil()
}

pub fn create_rotations(rules: u32, data: &Vec<[f32 ; 4]>) -> Vec<Vec<[f32 ; 4]>> {

    let mut result = Vec::new();

    let identity = rules & 1    != 0;  // 0
    let r90x = rules     & 2    != 0;  // 1
    let r180x = rules    & 4    != 0;  // 2
    let r270x = rules    & 8    != 0;  // 3
    let r90y = rules     & 16   != 0;  // 4
    let r180y = rules    & 32   != 0;  // 5
    let r270y = rules    & 64   != 0;  // 6
    let r90z = rules     & 128  != 0;  // 7
    let r180z = rules    & 256  != 0;  // 8
    let r270z = rules    & 512  != 0;  // 9
    let mir_x = rules & 1024 != 0; // 10
    let mir_y = rules & 2048 != 0; // 11
    let mir_z = rules & 4096 != 0; // 12

    // Add surrounding ifs.

    if identity {
        let mut temp = Vec::new();
        for d in data.iter() { temp.push(*d); }
        result.push(temp);
    }

    if r90x {
        let mut temp = Vec::new();
        // for d in data.iter() { if r90x { temp.push(ro90x(d));   } }
        for d in data.iter() { temp.push(ro90x(d)); }
        result.push(temp);
    }

    if r180x {
        let mut temp = Vec::new();
        for d in data.iter() { temp.push(ro180x(d)); }
        // for d in data.iter() { if r180x { temp.push(ro180x(d)); } }
        result.push(temp);
    }

    if r270x {
        let mut temp = Vec::new();
        for d in data.iter() { temp.push(ro270x(d)); }
        result.push(temp);
    }

    if r90y {
        let mut temp = Vec::new();
        for d in data.iter() { temp.push(ro90y(d)); }
        result.push(temp);
    }

    if r180y {
        let mut temp = Vec::new();
        for d in data.iter() { temp.push(ro180y(d)); }
        result.push(temp);
    }

    if r270y {
        let mut temp = Vec::new();
        for d in data.iter() { temp.push(ro270y(d)); }
        result.push(temp);
    }

    if r90z {
        let mut temp = Vec::new();
        for d in data.iter() { temp.push(ro90z(d)); }
        result.push(temp);
    }

    if r180z {
        let mut temp = Vec::new();
        for d in data.iter() { temp.push(ro180z(d)); }
        result.push(temp);
    }

    if r270z {
        let mut temp = Vec::new();
        for d in data.iter() { temp.push(ro270z(d)); }
        result.push(temp);
    }
    if mir_x {
        let mut temp = Vec::new();
        for d in data.iter() { temp.push(mirror_x(d)); }
        result.push(temp);
    }
    if mir_y {
        let mut temp = Vec::new();
        for d in data.iter() { temp.push(mirror_y(d)); }
        result.push(temp);
    }
    if mir_z {
        let mut temp = Vec::new();
        for d in data.iter() { temp.push(mirror_z(d)); }
        result.push(temp);
    }
    
    // TODO: implement refections too.
    result
}

pub fn check_connections(input: &Vec<[f32; 4]>, neighbor: &Vec<[f32;4]>, neighbor_rotations: u32) -> [u32 ; 6] {

    // TODO: Assert neighbor rotations.
    // Check all 6 directions for all rotations.
    let mut x_plus =  input.iter().filter(|x| x[0] ==  2.0).map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
    let mut x_minus = input.iter().filter(|x| x[0] == -2.0).map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
    let mut y_plus =  input.iter().filter(|x| x[1] ==  2.0).map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
    let mut y_minus = input.iter().filter(|x| x[1] == -2.0).map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
    let mut z_plus =  input.iter().filter(|x| x[2] ==  2.0).map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
    let mut z_minus = input.iter().filter(|x| x[2] == -2.0).map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();

    x_plus.sort();
    x_minus.sort();
    y_plus.sort();
    y_minus.sort();
    z_plus.sort();
    z_minus.sort();

    // println!("x_plus == {:?}", x_plus);
    // println!("x_minus == {:?}", x_minus);
    // println!("u_plus == {:?}", y_plus);
    // println!("y_minus == {:?}", y_minus);
    // println!("z_plus == {:?}", z_plus);
    // println!("z_minus == {:?}", z_minus);

    let all_neighbor_rotations = create_rotations(neighbor_rotations, neighbor);

    // 0 :: x+ direction
    // 1 :: x- direction
    // 2 :: y+ direction
    // 3 :: y- direction
    // 4 :: z+ direction
    // 5 :: z- direction
    let mut result = [0, 0, 0, 0, 0, 0];
    
    // input x+ amd left neighbor x- matches.
    for (index, n) in all_neighbor_rotations.iter().enumerate() {

        // Check the x+ side of the cube.  
        let mut all_x_minus_neighbor = n.iter().filter(|x| x[0] == -2.0).map(|x| [-1 * x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>(); 
        all_x_minus_neighbor.sort();
        // println!("all_x_minus_neighbor {:?}", all_x_minus_neighbor);

        // x- matches! Add rotation index  
        if all_x_minus_neighbor == x_plus {
            result[0] |= 1 << index;
        }
    }
    // input x- and right neighbor x+ matches.
    for (index, n) in all_neighbor_rotations.iter().enumerate() {

        // Check the x- side of the cube.  
        let mut all_x_minus_neighbor = n.iter().filter(|x| x[0] == 2.0).map(|x| [-1 * x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>(); 
        all_x_minus_neighbor.sort();
        // println!("all_x_minus_neighbor {:?}", all_x_minus_neighbor);

        // x+ matches! Add rotation index  
        if all_x_minus_neighbor == x_minus {
            result[1] |= 1 << index;
        }
    }
    // input y+ and y- neighbor.
    for (index, n) in all_neighbor_rotations.iter().enumerate() {

        // Check the x- side of the cube.  
        let mut all_y_minus_neighbor = n.iter().filter(|x| x[1] == -2.0).map(|x| [x[0] as i32, -1 * x[1] as i32, x[2] as i32]).collect::<Vec<_>>(); 
        all_y_minus_neighbor.sort();
        // println!("all_x_minus_neighbor {:?}", all_x_minus_neighbor);

        // x+ matches! Add rotation index  
        if all_y_minus_neighbor == y_plus {
            result[2] |= 1 << index;
        }
    }

    // input y- and y+ neighbor.
    for (index, n) in all_neighbor_rotations.iter().enumerate() {

        // Check the x- side of the cube.  
        let mut all_y_plus_neighbor = n.iter().filter(|x| x[1] == 2.0).map(|x| [x[0] as i32, -1 * x[1] as i32, x[2] as i32]).collect::<Vec<_>>(); 
        all_y_plus_neighbor.sort();
        // println!("all_x_minus_neighbor {:?}", all_x_minus_neighbor);

        // x+ matches! Add rotation index  
        if all_y_plus_neighbor == y_minus {
            result[3] |= 1 << index;
        }
    }

    // input z+ and z- neighbor.
    for (index, n) in all_neighbor_rotations.iter().enumerate() {

        // Check the z+ side of the cube.  
        let mut all_z_minus_neighbor = n.iter().filter(|x| x[2] == -2.0).map(|x| [x[0] as i32, x[1] as i32, -1 * x[2] as i32]).collect::<Vec<_>>(); 
        all_z_minus_neighbor.sort();

        // z+ matches! Add rotation index  
        if all_z_minus_neighbor == z_plus {
            result[4] |= 1 << index;
        }
    }

    // input z- and z+ neighbor.
    for (index, n) in all_neighbor_rotations.iter().enumerate() {

        // Check the z- side of the cube.  
        let mut all_z_plus_neighbor = n.iter().filter(|x| x[2] == 2.0).map(|x| [x[0] as i32, x[1] as i32, -1 * x[2] as i32]).collect::<Vec<_>>(); 
        all_z_plus_neighbor.sort();

        // z- matches! Add rotation index  
        if all_z_plus_neighbor == z_minus {
            result[5] |= 1 << index;
        }
    }

    result
}

// TODO: asserts that check that the input is actually big enought.
#[inline]
pub fn ro90x(vec: &[f32; 4]) -> [f32 ; 4] {
    [round_half_up(vec[0]),
     round_half_up(-vec[2]),
     round_half_up(vec[1]),
     round_half_up(vec[3])]
}

#[inline]
pub fn ro180x(vec: &[f32; 4]) -> [f32 ; 4] {
    [round_half_up(vec[0]),
     round_half_up(-vec[1]),
     round_half_up(-vec[2]),
     round_half_up(vec[3])]
}

#[inline]
pub fn ro270x(vec: &[f32; 4]) -> [f32 ; 4] {
    [round_half_up(vec[0]),
     round_half_up(vec[2]),
     round_half_up(-vec[1]),
     round_half_up(vec[3])]
}

#[inline]
pub fn ro90y(vec: &[f32; 4]) -> [f32 ; 4] {
    [round_half_up(vec[2]),
     round_half_up(vec[1]),
     round_half_up(-vec[0]),
     round_half_up(vec[3])]
}
 
#[inline]
pub fn ro180y(vec: &[f32; 4]) -> [f32 ; 4] {
    [round_half_up(-vec[0]),
     round_half_up(vec[1]),
     round_half_up(-vec[2]),
     round_half_up(vec[3])]
}

#[inline]
pub fn ro270y(vec: &[f32]) -> [f32 ; 4] {
    [round_half_up(-vec[2]),
     round_half_up(vec[1]),
     round_half_up(vec[0]),
     round_half_up(vec[3])]
}

#[inline]
pub fn ro90z(vec: &[f32]) -> [f32 ; 4] {
    [round_half_up(-vec[1]),
     round_half_up(vec[0]),
     round_half_up(vec[2]),
     round_half_up(vec[3])]
}

#[inline]
pub fn ro180z(vec: &[f32]) -> [f32 ; 4] {
    [round_half_up(-vec[0]),
     round_half_up(-vec[1]),
     round_half_up(vec[2]),
     round_half_up(vec[3])]
}

#[inline]
pub fn ro270z(vec: &[f32]) -> [f32 ; 4] {
    [round_half_up(vec[1]),
     round_half_up(-vec[0]),
     round_half_up(vec[2]),
     round_half_up(vec[3])]
}

#[inline]
pub fn mirror_x(vec: &[f32]) -> [f32 ; 4] {
    [round_half_up(-vec[0]),
     round_half_up(vec[1]),
     round_half_up(vec[2]),
     round_half_up(vec[3])]
}
#[inline]
pub fn mirror_y(vec: &[f32]) -> [f32 ; 4] {
    [round_half_up(vec[0]),
     round_half_up(-vec[1]),
     round_half_up(vec[2]),
     round_half_up(vec[3])]
}
#[inline]
pub fn mirror_z(vec: &[f32]) -> [f32 ; 4] {
    [round_half_up(vec[0]),
     round_half_up(vec[1]),
     round_half_up(-vec[2]),
     round_half_up(vec[3])]
}

#[inline]
pub fn ro90xv3(vec: &[f32; 3]) -> [f32 ; 3] {
    [round_half_up(vec[0]),
     round_half_up(-vec[2]),
     round_half_up(vec[1])]
}

#[inline]
pub fn ro180xv3(vec: &[f32; 3]) -> [f32 ; 3] {
    [round_half_up(vec[0]),
     round_half_up(-vec[1]),
     round_half_up(-vec[2])]
}

#[inline]
pub fn ro270xv3(vec: &[f32; 3]) -> [f32 ; 3] {
    [round_half_up(vec[0]),
     round_half_up(vec[2]),
     round_half_up(-vec[1])]
}

#[inline]
pub fn ro90yv3(vec: &[f32; 3]) -> [f32 ; 3] {
    [round_half_up(vec[2]),
     round_half_up(vec[1]),
     round_half_up(-vec[0])]
}
 
#[inline]
pub fn ro180yv3(vec: &[f32; 3]) -> [f32 ; 3] {
    [round_half_up(-vec[0]),
     round_half_up(vec[1]),
     round_half_up(-vec[2])]
}

#[inline]
pub fn ro270yv3(vec: &[f32; 3]) -> [f32 ; 3] {
    [round_half_up(-vec[2]),
     round_half_up(vec[1]),
     round_half_up(vec[0])]
}

#[inline]
pub fn ro90zv3(vec: &[f32; 3]) -> [f32 ; 3] {
    [round_half_up(-vec[1]),
     round_half_up(vec[0]),
     round_half_up(vec[2])]
}

#[inline]
pub fn ro180zv3(vec: &[f32; 3]) -> [f32 ; 3] {
    [round_half_up(-vec[0]),
     round_half_up(-vec[1]),
     round_half_up(vec[2])]
}

#[inline]
pub fn ro270zv3(vec: &[f32; 3]) -> [f32 ; 3] {
    [round_half_up(vec[1]),
     round_half_up(-vec[0]),
     round_half_up(vec[2])]
}

#[inline]
pub fn mirror_xv3(vec: &[f32; 3]) -> [f32 ; 3] {
    [round_half_up(-vec[0]),
     round_half_up(vec[1]),
     round_half_up(vec[2])]
}
#[inline]
pub fn mirror_yv3(vec: &[f32]) -> [f32 ; 3] {
    [round_half_up(vec[0]),
     round_half_up(-vec[1]),
     round_half_up(vec[2])]
}
#[inline]
pub fn mirror_zv3(vec: &[f32]) -> [f32 ; 3] {
    [round_half_up(vec[0]),
     round_half_up(vec[1]),
     round_half_up(-vec[2])]
}

#[inline]
pub fn rex(vec: &[f32]) -> [f32 ; 3] {
    [vec[0], -vec[1], -vec[2]]
}

#[inline]
pub fn rey(vec: &[f32]) -> [f32 ; 3] {
    [-vec[0], vec[1], -vec[2]]
}

#[inline]
pub fn rez(vec: &[f32]) -> [f32 ; 3] {
    [-vec[0], -vec[1], vec[2]]
}

//////////////////////////


#[inline]
pub fn rotate_90z(vec: &[f32 ; 3]) -> [f32 ; 3] {

    [-vec[1], vec[0], vec[2]]
}

#[inline]
pub fn rotate_180z(vec: &[f32 ; 3]) -> [f32 ; 3] {
    [-vec[0], -vec[1], vec[2]]
}

#[inline]
pub fn rotate_270z(vec: &[f32 ; 3]) -> [f32 ; 3] {
    [vec[1], -vec[0], vec[2]]
}

#[inline]
pub fn rotate_90x(vec: &[f32 ; 3]) -> [f32 ; 3] {
    [vec[0], -vec[2], vec[1]]
}

#[inline]
pub fn rotate_180x(vec: &[f32 ; 3]) -> [f32 ; 3] {
    [vec[0], -vec[1], -vec[2]]
}

#[inline]
pub fn rotate_270x(vec: &[f32 ; 3]) -> [f32 ; 3] {
    [vec[0], vec[2], -vec[1]]
}

#[inline]
pub fn rotate_90y(vec: &[f32 ; 3]) -> [f32 ; 3] {
    [vec[2], vec[1], -vec[0]]
}
 
#[inline]
pub fn rotate_180y(vec: &[f32 ; 3]) -> [f32 ; 3] {
    [-vec[0], vec[1], -vec[2]]
}

#[inline]
pub fn rotate_270y(vec: &[f32 ; 3]) -> [f32 ; 3] {
    [-vec[2], vec[1], vec[0]]
}

#[inline]
pub fn reflect_x(vec: &[f32 ; 3]) -> [f32 ; 3] {
    [vec[0], -vec[1], -vec[2]]
}

#[inline]
pub fn reflect_y(vec: &[f32 ; 3]) -> [f32 ; 3] {
    [-vec[0], vec[1], -vec[2]]
}

#[inline]
pub fn reflect_z(vec: &[f32 ; 3]) -> [f32 ; 3] {
    [-vec[0], -vec[1], vec[2]]
}


/***********************************************************************************/

#[derive(Clone)]
pub struct WfcBlock {

    pub id: u32,
    dimension: u32, // Symmetric matrix
    connection_data: Vec<[f32 ; 3]>,
    render_data: Vec<[f32 ; 4]>,
    neighbors: Vec<(u32, [u32 ; 6])>,
    match_data: Vec<Vec<[i32 ; 3]>>,
    match_inverted: Vec<Vec<[i32 ; 3]>>
}

impl WfcBlock {

    pub fn init(id: u32, dimension: u32, connection_data: Vec<[f32 ; 3]>, render_data: Vec<[f32 ; 4]>) -> Self {

        // Precalculate match data for each direction.
        let mut identity =  connection_data.iter().map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
        let mut x_plus =  connection_data.iter().filter(|x| x[0] ==  2.0).map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
        let mut x_minus = connection_data.iter().filter(|x| x[0] == -2.0).map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
        let mut y_plus =  connection_data.iter().filter(|x| x[1] ==  2.0).map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
        let mut y_minus = connection_data.iter().filter(|x| x[1] == -2.0).map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
        let mut z_plus =  connection_data.iter().filter(|x| x[2] ==  2.0).map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();
        let mut z_minus = connection_data.iter().filter(|x| x[2] == -2.0).map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>();

        x_plus.sort();
        x_minus.sort();
        y_plus.sort();
        y_minus.sort();
        z_plus.sort();
        z_minus.sort();
        identity.sort();

        // Precalculate inverted match data for each direction.
        let mut x_minus_inverted = connection_data.iter().filter(|x| x[0] == -2.0).map(|x| [-1 * x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>(); 
        let mut x_plus_inverted  = connection_data.iter().filter(|x| x[0] == 2.0).map(|x| [-1 * x[0] as i32, x[1] as i32, x[2] as i32]).collect::<Vec<_>>(); 
        let mut y_minus_inverted = connection_data.iter().filter(|x| x[1] == -2.0).map(|x| [x[0] as i32, -1 * x[1] as i32, x[2] as i32]).collect::<Vec<_>>(); 
        let mut y_plus_inverted  = connection_data.iter().filter(|x| x[1] == 2.0).map(|x| [x[0] as i32, -1 * x[1] as i32, x[2] as i32]).collect::<Vec<_>>(); 
        let mut z_minus_inverted = connection_data.iter().filter(|x| x[2] == -2.0).map(|x| [x[0] as i32, x[1] as i32, -1 * x[2] as i32]).collect::<Vec<_>>(); 
        let mut z_plus_inverted  = connection_data.iter().filter(|x| x[2] == 2.0).map(|x| [x[0] as i32, x[1] as i32, -1 * x[2] as i32]).collect::<Vec<_>>(); 

        x_minus_inverted.sort();
        x_plus_inverted.sort();
        y_minus_inverted.sort();
        y_plus_inverted.sort();
        z_minus_inverted.sort();
        z_plus_inverted.sort();

        Self {
            id: id,
            dimension: dimension,
            connection_data: connection_data,
            render_data: render_data,
            neighbors: Vec::new(),
            match_data: vec![identity.clone(), x_plus, x_minus, y_plus, y_minus, z_plus, z_minus],
            match_inverted: vec![identity, // Inverted identity is not used. It's just a placeholder.
                                 x_minus_inverted,
                                 x_plus_inverted,
                                 y_minus_inverted,
                                 y_plus_inverted,
                                 z_minus_inverted,
                                 z_plus_inverted],
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_dimension(&self) -> u32 {
        self.dimension
    }

    pub fn get_match_data(&self, direction: usize) -> &Vec<[i32; 3]> {
        assert!(direction < 7);
        &self.match_data[direction]
    }

    // Use 1 => x+
    // Use 2 => x-
    // Use 3 => y+
    // Use 4 => y-
    // Use 5 => z+
    // Use 6 => z-
    pub fn get_inverted_match(&self, direction: usize) -> &Vec<[i32; 3]> {
        assert!(direction < 7);
        &self.match_inverted[direction]
    }

    pub fn get_connection_data(&self) -> &Vec<[f32; 3]> {
        &self.connection_data
    }

    // Check if this block matches with an other block.
    // Direction: 0 :: identity
    // Direction: 1 :: x+
    // Direction: 2 :: x-
    // Direction: 3 :: y+
    // Direction: 4 :: y-
    // Direction: 5 :: z+
    // Direction: 6 :: z-
    pub fn matches(&self, other: &WfcBlock, direction: usize) -> bool {
        assert!(direction < 7);
        
        // VALIDATION!!!!!!!!!!!!!!
        *other.get_inverted_match(direction) == self.match_data[direction]
    }

    pub fn create_rotation(&self, rule: u32, id: u32) -> WfcBlock {
    
        let mut connection_data = Vec::new();
    
        let identity = rule & 1    != 0;  // 0
        let r90x = rule     & 2    != 0;  // 1
        let r180x = rule    & 4    != 0;  // 2
        let r270x = rule    & 8    != 0;  // 3
        let r90y = rule     & 16   != 0;  // 4
        let r180y = rule    & 32   != 0;  // 5
        let r270y = rule    & 64   != 0;  // 6
        let r90z = rule     & 128  != 0;  // 7
        let r180z = rule    & 256  != 0;  // 8
        let r270z = rule    & 512  != 0;  // 9
        let mir_x = rule & 1024 != 0; // 10
        let mir_y = rule & 2048 != 0; // 11
        let mir_z = rule & 4096 != 0; // 12
    
        if identity {
            let mut temp = Vec::new();
            for d in self.connection_data.iter() { temp.push(*d); }
            connection_data.push(temp);
        }
    
        if r90x {
            let mut temp = Vec::new();
            // for d in data.iter() { if r90x { temp.push(ro90x(d));   } }
            for d in self.connection_data.iter() { temp.push(ro90xv3(d)); }
            connection_data.push(temp);
        }
    
        if r180x {
            let mut temp = Vec::new();
            for d in self.connection_data.iter() { temp.push(ro180xv3(d)); }
            // for d in data.iter() { if r180x { temp.push(ro180x(d)); } }
            connection_data.push(temp);
        }
    
        if r270x {
            let mut temp = Vec::new();
            for d in self.connection_data.iter() { temp.push(ro270xv3(d)); }
            connection_data.push(temp);
        }
    
        if r90y {
            let mut temp = Vec::new();
            for d in self.connection_data.iter() { temp.push(ro90yv3(d)); }
            connection_data.push(temp);
        }
    
        if r180y {
            let mut temp = Vec::new();
            for d in self.connection_data.iter() { temp.push(ro180yv3(d)); }
            connection_data.push(temp);
        }
    
        if r270y {
            let mut temp = Vec::new();
            for d in self.connection_data.iter() { temp.push(ro270yv3(d)); }
            connection_data.push(temp);
        }
    
        if r90z {
            let mut temp = Vec::new();
            for d in self.connection_data.iter() { temp.push(ro90zv3(d)); }
            connection_data.push(temp);
        }
    
        if r180z {
            let mut temp = Vec::new();
            for d in self.connection_data.iter() { temp.push(ro180zv3(d)); }
            connection_data.push(temp);
        }
    
        if r270z {
            let mut temp = Vec::new();
            for d in self.connection_data.iter() { temp.push(ro270zv3(d)); }
            connection_data.push(temp);
        }
        // Not checked yet.
        if mir_x {
            let mut temp = Vec::new();
            for d in self.connection_data.iter() { temp.push(mirror_xv3(d)); }
            connection_data.push(temp);
        }
        // Not checked yet.
        if mir_y {
            let mut temp = Vec::new();
            for d in self.connection_data.iter() { temp.push(mirror_yv3(d)); }
            connection_data.push(temp);
        }
        // Not checked yet.
        if mir_z {
            let mut temp = Vec::new();
            for d in self.connection_data.iter() { temp.push(mirror_zv3(d)); }
            connection_data.push(temp);
        }
        assert!(connection_data.len() > 0);
        
        // TODO: Rotate render data!!!!!
        WfcBlock::init(id, self.dimension, connection_data[0].clone(), self.render_data.clone())
    }
}
