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

pub fn create_rotations(rules: u32, data: &Vec<[f32 ; 4]>) -> Vec<Vec<[f32 ; 4]>> {

    let mut result = Vec::new();

    let r90x = rules  & 1  != 0;
    let r180x = rules & 2  != 0;
    let r270x = rules & 4  != 0;
    let r90y = rules  & 8  != 0;
    let r180y = rules & 16 != 0;
    let r270y = rules & 32 != 0;
    let r90z = rules  & 64 != 0;
    let r180z = rules & 128 != 0;
    let r270z = rules & 256 != 0;

    let mut temp = Vec::new();
    for d in data.iter() { if r90x { temp.push(ro90x(d));   } }
    result.push(temp);

    let mut temp = Vec::new();
    for d in data.iter() { if r180x { temp.push(ro180x(d)); } }
    result.push(temp);

    let mut temp = Vec::new();
    for d in data.iter() { if r270x { temp.push(ro270x(d)); } }
    result.push(temp);

    let mut temp = Vec::new();
    for d in data.iter() { if r90y { temp.push(ro90y(d));   } }
    result.push(temp);

    let mut temp = Vec::new();
    for d in data.iter() { if r180y { temp.push(ro180y(d)); } }
    result.push(temp);

    let mut temp = Vec::new();
    for d in data.iter() { if r270y { temp.push(ro270y(d)); } }
    result.push(temp);

    let mut temp = Vec::new();
    for d in data.iter() { if r90z { temp.push(ro90z(d));   } }
    result.push(temp);

    let mut temp = Vec::new();
    for d in data.iter() { if r180z { temp.push(ro180z(d)); } }
    result.push(temp);

    let mut temp = Vec::new();
    for d in data.iter() { if r270z { temp.push(ro270z(d)); } }
    result.push(temp);
    
    // TODO: implement refections too.
    result
}

// TODO: asserts that check that the input is actually big enought.
#[inline]
pub fn ro90x(vec: &[f32; 4]) -> [f32 ; 4] {
    [vec[0], -vec[2], vec[1], vec[3]]
}

#[inline]
pub fn ro180x(vec: &[f32; 4]) -> [f32 ; 4] {
    [vec[0], -vec[1], -vec[2], vec[3]]
}

#[inline]
pub fn ro270x(vec: &[f32; 4]) -> [f32 ; 4] {
    [vec[0], vec[2], -vec[1], vec[3]]
}

#[inline]
pub fn ro90y(vec: &[f32; 4]) -> [f32 ; 4] {
    [vec[2], vec[1], -vec[0], vec[3]]
}
 
#[inline]
pub fn ro180y(vec: &[f32; 4]) -> [f32 ; 4] {
    [-vec[0], vec[1], -vec[2], vec[3]]
}

#[inline]
pub fn ro270y(vec: &[f32]) -> [f32 ; 4] {
    [-vec[2], vec[1], vec[0], vec[3]]
}

#[inline]
pub fn ro90z(vec: &[f32]) -> [f32 ; 4] {
    [-vec[1], vec[0], vec[2], vec[3]]
}

#[inline]
pub fn ro180z(vec: &[f32]) -> [f32 ; 4] {
    [-vec[0], -vec[1], vec[2], vec[3]]
}

#[inline]
pub fn ro270z(vec: &[f32]) -> [f32 ; 4] {
    [vec[1], -vec[0], vec[2], vec[3]]
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
