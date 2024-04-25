use std::collections::HashMap;
use std::mem::transmute;

pub fn test_data(color: u32) -> Vec<[f32 ; 4]> {

    let mut result = Vec::new();

    let color: f32 = unsafe {transmute::<u32, f32>(color)};

    for i in -2..2 {
        for k in -2..3 {
            result.push([i as f32, -2.0, k as f32, color]);
        }
    }
    for j in -2..3 {
        for k in -2..3 {
            result.push([2.0, j as f32, k as f32, color]);
        }
    }
    result 
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
    pub fn init(id: u32, cases: u32, dimension: u32, weight: f32, connection_data: &Vec<[f32; 4]>, visual_data: &Vec<[f32 ; 4]>) -> Self {

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
    // [round_half_up(vec[0]),
    //  round_half_up(-vec[2]),
    //  round_half_up(vec[1]),
    //
    // [round_half_up(vec[0]),
    //  round_half_up(-vec[1]),
    //  round_half_up(-vec[2]),
    //
    // [round_half_up(vec[0]),
    //  round_half_up(vec[2]),
    //  round_half_up(-vec[1]),
    //
    // [round_half_up(vec[2]),
    //  round_half_up(vec[1]),
    //  round_half_up(-vec[0]),
    //
    // [round_half_up(-vec[0]),
    //  round_half_up(vec[1]),
    //  round_half_up(-vec[2]),
    //  round_half_up(vec[3])]
    // [round_half_up(-vec[2]),
    //  round_half_up(vec[1]),
    //  round_half_up(vec[0]),
    // [round_half_up(-vec[1]),
    //  round_half_up(vec[0]),
    //  round_half_up(vec[2]),
    //[round_half_up(-vec[0]),
    // round_half_up(-vec[1]),
    // round_half_up(vec[2]),
    // [round_half_up(vec[1]),
    //  round_half_up(-vec[0]),
    //  round_half_up(vec[2]),
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
            // else { panic!("current_rotation not supported.");
        }
        // Check that neighbor rotations are legal.
        result
    }

    // All possible cases. Directions that has already a known neighbor is given as argumen.
    // Return hash map where key is voxel id and all possible rotations for that voxel.
    // pub fn get_all_possible_cases(&self,
    //                               x_plus:  Option<&Voxel>,
    //                               x_minus: Option<&Voxel>,
    //                               y_plus:  Option<&Voxel>,
    //                               y_minus: Option<&Voxel>,
    //                               z_plus:  Option<&Voxel>,
    //                               z_minus: Option<&Voxel>,
    //                               ) -> HashMap<u32, [u32 ; 6]> {

    //     // Check cases for each direction.
    //     if let Some(x_plus) {
    //           
    //     }

    // }
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

    let identity = rules & 1   != 0;  // 0
    let r90x = rules     & 2   != 0;  // 1
    let r180x = rules    & 4   != 0;  // 2
    let r270x = rules    & 8   != 0;  // 3
    let r90y = rules     & 16  != 0;  // 4
    let r180y = rules    & 32  != 0;  // 5
    let r270y = rules    & 64  != 0;  // 6
    let r90z = rules     & 128 != 0;  // 7
    let r180z = rules    & 256 != 0;  // 8
    let r270z = rules    & 512 != 0;  // 9

    // let r90x = rules  & 1   != 0;
    // let r180x = rules & 2   != 0;
    // let r270x = rules & 4   != 0;
    // let r90y = rules  & 8   != 0;
    // let r180y = rules & 16  != 0;
    // let r270y = rules & 32  != 0;
    // let r90z = rules  & 64  != 0;
    // let r180z = rules & 128 != 0;
    // let r270z = rules & 256 != 0;

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
    
    // TODO: implement refections too.
    result
}

pub fn check_connections(input: &Vec<[f32; 4]>, neighbor: &Vec<[f32;4]>, neighbor_rotations: u32) -> [u32 ; 6] {

    // TODO: Assert neighbor rotations.
    // Check all 6 direction for all rotations.
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

    println!("x_plus == {:?}", x_plus);
    println!("x_minus == {:?}", x_minus);
    println!("u_plus == {:?}", y_plus);
    println!("y_minus == {:?}", y_minus);
    println!("z_plus == {:?}", z_plus);
    println!("z_minus == {:?}", z_minus);

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
