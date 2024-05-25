struct PrefixParams {
    data_start_index: u32,
    data_end_index: u32,
    data_size: u32,
};

struct FmmBlock {
    index: u32,
    band_points_count: u32,
};

var<workgroup> predicate_array: array<u32, 1024>;
var<workgroup> count_array: array<u32, 1024>;

@group(0)
@binding(0)
var<uniform> fmm_prefix_params: PrefixParams;

@group(0)
@binding(1)
var<storage, read_write> input_array: array<FmmBlock>;

// fn lane_id(id: u32, subgroup_size: u32) -> i32 {
//     return id % subgroup_size;
// }

@compute
@workgroup_size(64,1,1)
fn main(@builtin(local_invocation_id)    local_id: vec3<u32>,
        @builtin(local_invocation_index) local_index: u32,
        @builtin(workgroup_id) work_group_id: vec3<u32>,
        @builtin(global_invocation_id)   global_id: vec3<u32>, 
        @builtin(subgroup_size) sg_size : u32,
        @builtin(subgroup_invocation_id) sg_id : u32) {

        if (global_id.x >= fmm_prefix_params.data_size >> 5u) {
	    return;
	}

	let warp_id = global_id.x >> 5u;

	for (var i=0u ; i < 32u ; i++) {
	    
	    // Test the predicate. Calculate the number of 1 bits.
	    let mask = subgroupBallot(input_array[warp_id << 10u + i << 5u + sg_id].band_points_count > 0); // Predicate for this.
	    let one_bits = countOneBits(mask);
        }


        // // Create one bit masks subgroup-level sum of bit masks.
        // // Store results to predicate and count array. 
	// for(var i = 0u; i < 32u ; i++) {
	//     mask = __ballot(input[(warp_id<<10)+(i<<5)+lnid] <= percent);
	//     
	//     if (lnid == 0)
	//     pred[(warp_id<<5)+i] = mask;
	//     
	//     if (lnid == i)
	//     cnt = __popc(mask);
	// }

	// Calculate the sum of all one bits.
	if (local_id.x < 10u) {
	    var val = input_array[local_id.x].band_points_count;
	    for (var offset = 16u; offset > 0u; offset >>= 1u) {
                val += subgroupShuffleDown(val, offset);
	    }
        }

	// subgroupBallot(local_id.x < 10);
	// subgroupBarrier();

        // var jep = subgroupAll(true);
        // var jep2 = subgroupAny(true);

        // subgroupAdd(1);
	// subgroupMul(3);
        // subgroupMax(15);
        // subgroupMin(10);
        // subgroupAnd(0xfu);
        // subgroupOr(0x0f0u);
        // subgroupXor(0xf00u);
        // subgroupExclusiveAdd(5);
        // subgroupExclusiveMul(7);
        // subgroupInclusiveAdd(3);
        // subgroupInclusiveMul(10);

        // subgroupBroadcastFirst(5);
	// subgroupBroadcast(123, 10u);
        // subgroupShuffle(5, 5u);
        // subgroupShuffleDown(5, 5u);
        // subgroupShuffleUp(1, 3u);
        // subgroupShuffleXor(1, 3u);
	
}
