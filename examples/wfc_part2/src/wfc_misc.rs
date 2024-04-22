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

    let r90x = rules  & 1   != 0;
    let r180x = rules & 2   != 0;
    let r270x = rules & 4   != 0;
    let r90y = rules  & 8   != 0;
    let r180y = rules & 16  != 0;
    let r270y = rules & 32  != 0;
    let r90z = rules  & 64  != 0;
    let r180z = rules & 128 != 0;
    let r270z = rules & 256 != 0;

    // Add surrounding ifs.
    if r90x {
        let mut temp = Vec::new();
        for d in data.iter() { if r90x { temp.push(ro90x(d));   } }
        result.push(temp);
    }

    if r180x {
        let mut temp = Vec::new();
        for d in data.iter() { if r180x { temp.push(ro180x(d)); } }
        result.push(temp);
    }

    if r270x {
        let mut temp = Vec::new();
        for d in data.iter() { if r270x { temp.push(ro270x(d)); } }
        result.push(temp);
    }

    if r90y {
        let mut temp = Vec::new();
        for d in data.iter() { if r90y { temp.push(ro90y(d));   } }
        result.push(temp);
    }

    if r180y {
        let mut temp = Vec::new();
        for d in data.iter() { if r180y { temp.push(ro180y(d)); } }
        result.push(temp);
    }

    if r270y {
        let mut temp = Vec::new();
        for d in data.iter() { if r270y { temp.push(ro270y(d)); } }
        result.push(temp);
    }

    if r90z {
        let mut temp = Vec::new();
        for d in data.iter() { if r90z { temp.push(ro90z(d));   } }
        result.push(temp);
    }

    if r180z {
        let mut temp = Vec::new();
        for d in data.iter() { if r180z { temp.push(ro180z(d)); } }
        result.push(temp);
    }

    if r270z {
        let mut temp = Vec::new();
        for d in data.iter() { if r270z { temp.push(ro270z(d)); } }
        result.push(temp);
    }
    
    // TODO: implement refections too.
    result
}

pub fn check_connections_5x5x5(input: &Vec<[f32; 4]>, neighbor: &Vec<[f32;4]>) -> [u32 ; 6] {

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

    let all_neighbor_rotations = create_rotations(0b111111111, neighbor);

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

        // z+ matches! Add rotation index  
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
