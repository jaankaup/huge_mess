use crate::misc::index_to_uvec3;
use std::collections::HashMap;
use crate::misc::udiv_up_safe32;
use crate::misc::uvec3_to_index;

#[derive(Clone, Debug)]
pub struct SceneNode {
    pub tag: WfcTag,
    pub index: u32,
    pub wfc_data: WfcData,
    pub alternatives: Vec<WfcData>,
}


#[derive(Copy, Clone)]
pub enum Direction {
    Top,
    Left,
    Bottom,
    Right,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WfcTag {
    Known,
    Band,
}

#[derive(Clone, Debug)]
pub struct WfcData {
    pub dimensionX: u32,
    pub dimensionY: u32,
    pub dimensionZ: u32,
    pub data: Vec<u8>,
}

impl WfcData {
    pub fn init(dimX: u32, dimY: u32, dimZ: u32) -> Self {
        let total_size = dimX * dimY * dimZ; 
        let data_array: Vec<u8> = vec![0; (dimX * dimY * dimZ).try_into().unwrap()]; 
        
        Self {
            dimensionX: dimX,
            dimensionY: dimY,
            dimensionZ: dimZ,
            data: data_array,
        }
    }
    pub fn write(&mut self, x: u32, y: u32, z: u32, value: u8) {
        let index = uvec3_to_index(x, y, z, self.dimensionX, self.dimensionY);

        self.data[index as usize] = value;
    }

    pub fn rotate90(&self) -> WfcData {

        let mut temp: Vec<u8> = Vec::with_capacity((self.dimensionX * self.dimensionY).try_into().unwrap());
        for i in 0..self.dimensionX {
            for j in 0..self.dimensionY {
                // Assume symmetric matrix.
                temp.push(self.data[(i + (4 - j) * self.dimensionX) as usize]);
                // println!("{:?}", i + (4 - j) * 5);
            }
        }
        WfcData {
            dimensionX: self.dimensionX,
            dimensionY: self.dimensionY,
            dimensionZ: self.dimensionZ,
            data: temp,
        }
    }

    fn get_row(&self, row_index: u32) -> Vec<u8> {

        assert!(row_index < self.dimensionY); 

        let mut result = Vec::new();
        for i in 0..self.dimensionX {
            result.push(self.data[(row_index * self.dimensionY + i) as usize]);
        }
        
        result
    }

    fn get_column(&self, column_index: u32) -> Vec<u8> {

        assert!(column_index < self.dimensionX); 

        let mut result = Vec::new();
        for i in 0..self.dimensionY {
            result.push(self.data[(column_index + self.dimensionY * i) as usize]);
        }
        
        result
    }

    pub fn resolve_cases(neighbors: &Vec<(Option<SceneNode>, Direction)>,
                         cases: &HashMap<String, WfcData>,
                         dim_x: u32,
                         dim_y: u32) -> Vec<WfcData> {
        let mut result = Vec::new();
        // let wfc_neighbors = Vec::new();
        // Check all cases for each direction.
        for case in cases.iter() {
            let mut ok = true;
            for n in neighbors.iter() {
                if n.0.is_some() {
                    let n_ref = n.0.as_ref().unwrap();
                    if n_ref.tag == WfcTag::Band { continue; }
                    match n.1 {
                       Direction::Top => {
                            let top_row = n_ref.wfc_data.get_row(0);
                            let bottom_row_neighbor = case.1.get_row(dim_y - 1);
                            if top_row != bottom_row_neighbor {
                                ok = false;
                                break;
                            }
                       },
                       Direction::Right => {
                            let rigth_row = n_ref.wfc_data.get_column(dim_x - 1);
                            let left_row_neighbor = case.1.get_column(0);
                            if rigth_row != left_row_neighbor {
                                ok = false;
                                break;
                            }
                       },
                       Direction::Bottom => {
                            let bottom_row = n_ref.wfc_data.get_row(dim_y - 1);
                            let up_row_neighbor = case.1.get_row(0);
                            if bottom_row != up_row_neighbor {
                                ok = false;
                                break;
                            }
                       },
                       Direction::Left => {
                            let left_row = n_ref.wfc_data.get_column(0);
                            let right_row_neighbor = case.1.get_column(dim_x - 1);
                            if left_row != right_row_neighbor {
                                ok = false;
                                break;
                            }

                       },
                    }
                }
            } // for
            if ok {
                result.push(case.1.clone());
            }
        }
        
        // Check with cases satifies.
        // for case in cases.iter() { 
        //     let wcf = case.1;
        //     if top.is_some() {
        //         let top_ref = top.unwrap();
        //         let top_row = case.1.get_row(0);
        //         let bottom_row_neighbor = top_ref.get_row(dim_y - 1);
        //         if top_row != bottom_row_neighbor {
        //             continue;
        //         }
        //     }
        //     if right.is_some() {
        //         let right_ref = right.unwrap();
        //         let rigth_row = case.1.get_column(dim_x - 1);
        //         let left_row_neighbor = right_ref.get_column(0);
        //         if rigth_row != left_row_neighbor {
        //             continue;
        //         }
        //     }
        //     if bottom.is_some() {
        //         let bottom_ref = bottom.unwrap();
        //         let bottom_row = case.1.get_row(dim_y - 1);
        //         let up_row_neighbor = bottom_ref.get_row(0);
        //         if bottom_row != up_row_neighbor {
        //             continue;
        //         }
        //     }
        //     if left.is_some() {
        //         let left_ref = bottom.unwrap();
        //         let left_row = case.1.get_column(0);
        //         let right_row_neighbor = left_ref.get_column(dim_x - 1);
        //         if left_row != right_row_neighbor {
        //             continue;
        //         }
        //     }
        //     result.push(case.1.clone());
        // }
        result
    }

    pub fn test(&self, direction: Direction, cases: &HashMap<String, WfcData>) -> Vec<WfcData> {

        let mut result = Vec::new();

        // Do not check the up-axis.
        // Assume nxn-matrix.

        for (_,w) in cases.iter() {
            match direction {
                Direction::Top => {
                    // Check the top row agains neighbor bottom row.
                    let top_row = self.get_row(0);
                    let bottom_row_neighbor = w.get_row(w.dimensionY - 1);
                    if top_row == bottom_row_neighbor {
                        result.push(w.clone());
                    }

                },
                Direction::Right => {
                    let rigth_row = self.get_column(self.dimensionX - 1);
                    let left_row_neighbor = w.get_column(0);
                    if rigth_row == left_row_neighbor {
                        result.push(w.clone());
                    }
                },
                Direction::Bottom => {
                    let bottom_row = self.get_row(self.dimensionY - 1);
                    let up_row_neighbor = w.get_row(0);
                    if bottom_row == up_row_neighbor {
                        result.push(w.clone());
                    }
                },
                Direction::Left => {
                    let left_row = self.get_column(0);
                    let right_row_neighbor = w.get_column(w.dimensionX - 1);
                    if left_row == right_row_neighbor {
                        result.push(w.clone());
                    }
                },
            }
        }
        result
    }

    pub fn print(&self) {
        println!("The length = {:?}", self.data.len());
        println!("***********************");
        for (i, v) in self.data.iter().enumerate() {
            if i as u32 % self.dimensionX == 0 {
                print!("\n{:?} ", v);
            }
            else {
                print!("{:?} ", v);
            }
        }
        println!("***********************");
    }

    pub fn toString(&self) -> String {
        String::from_utf8(self.data.clone()).unwrap()
    }

    pub fn get_inner_locations(&self) -> Vec<[i32; 3]> {

        let mut result: Vec<[i32; 3]>  = Vec::new();
        for x in 0..self.data.len() {
            if self.data[x as usize] != 0 {
                let index = index_to_uvec3(x.try_into().unwrap(), self.dimensionX, self.dimensionY);
                result.push([index[0] as i32, index[1] as i32, index[2] as i32]);
            }
        }
        result
    }
}

pub struct Block {
    data: Vec<u32>,
    neigborId: u32,
}

pub fn rotate90(matrix: Vec<u32>) -> Vec<u32> {
    let mut result: Vec<u32> = Vec::with_capacity(25);
    for i in 0..5 {
        for j in 0..5 {
            result.push(matrix[i + (4 - j) * 5]);
            println!("{:?}", i + (4 - j) * 5);
        }
    }
    result
}

pub fn check(center: &Vec<u32>, neigbor: &Vec<u32>, direction: Direction) -> bool {
    match direction {
        Direction::Top => {
            center[0] == neigbor[20] &&
                center[1] == neigbor[21] &&
                center[2] == neigbor[22] &&
                center[3] == neigbor[23] &&
                center[4] == neigbor[24]
        },
        Direction::Left => {
            center[4] == neigbor[0] &&
                center[9] == neigbor[5] &&
                center[14] == neigbor[10] &&
                center[19] == neigbor[15] &&
                center[24] == neigbor[20]

        },
        Direction::Bottom => {
            center[20] == neigbor[0] &&
                center[21] == neigbor[1] &&
                center[22] == neigbor[2] &&
                center[23] == neigbor[3] &&
                center[24] == neigbor[4]

        },
        Direction::Right => {
            center[0] == neigbor[4] &&
                center[5] == neigbor[9] &&
                center[10] == neigbor[14] &&
                center[15] == neigbor[19] &&
                center[20] == neigbor[24]

        },
    }
}

pub fn create_shapes() -> HashMap<u32, Vec<(i32, i32)>> {

    let mut result: HashMap<u32, Vec<(i32, i32)>> = HashMap::new();

    for i in 0..512 {

        let mut offsets: Vec<(i32, i32)> = Vec::new();

        if i & 1 == 1 {
            offsets.push((-1, 0)); 
            offsets.push(( 0, -1)); 
        }
        if i & 2 == 1 {
            offsets.push((0, -1)); 
        }
        if i & 3 == 1 {
            offsets.push((1, 0)); 
            offsets.push((0, -1)); 
        }
        if i & 4 == 1 {
            offsets.push((-1, 0)); 
        }
        if i & 5 == 1 {

        }
        if i & 6 == 1 {
            offsets.push((1, 0)); 
        }
        if i & 7 == 1 {
            offsets.push((-1, 0)); 
            offsets.push(( 0, 1)); 
        }
        if i & 8 == 1 {
            offsets.push((0, 1)); 
        }
        if i & 9 == 1 {
            offsets.push((1, 0)); 
            offsets.push((0, 1)); 
        }

        result.insert(i, offsets);
    }
    result
}

fn swap_bit_positions(a: u32, b: u32, value: u32) -> u32 {

    let mut result = value;

    if ((value & 1 << a) >> a) ^ ((value & (1 << b)) >> b) == 1
    {
        result ^= 1 << a;
        result ^= 1 << b;
    }
    result
}

pub fn create_x_mirror(case: u32) -> u32 {
    let mut result = case;
    result = swap_bit_positions(0,2,result);
    result = swap_bit_positions(3,5,result);
    result = swap_bit_positions(6,8,result);
    result
}

pub fn create_y_mirror(case: u32) -> u32 {
    let mut result = case;
    result = swap_bit_positions(0,6,result);
    result = swap_bit_positions(1,7,result);
    result = swap_bit_positions(2,8,result);
    result
}

// pub fn rotate90(case: u32) -> u32 {
//     let mut result = case;
// 
// }


// pub fn create_possible_combinations(case: u32, neighbors: [Option<u32> ; 4]) -> [Vec<u32> ; 4] {
//     for n : neigbors.iter() {
//          
//     }
// }
