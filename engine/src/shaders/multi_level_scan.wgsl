struct ScanParams {
    data_start_index: u32,
    data_end_index: u32,
    data_size: u32,
};

struct InputData {
    test_value: f32,
};

var<workgroup> predicate_array: array<vec4<u32>, 1024u>;
var<workgroup> count_array: array<u32, 1024>;

@group(0)
@binding(0)
var<uniform> scan_params: ScanParams;

@group(0)
@binding(1)
var<storage, read_write> input_array: array<InputData>;

// Process stream compaction. The amount of data processed is related to the subgroup_size.
// If subgroup_size is 32, then 2 * 32 * 32 = 2048 items is processed.
// If subgroup_size is 64, then     64 * 64 = 4096 items is processed.
// Low workgroup_size.x can cause problems when the data size grows.

@compute
@workgroup_size(128,1,1)
fn main(@builtin(local_invocation_id)    local_id: vec3<u32>,
        @builtin(local_invocation_index) local_index: u32,
        @builtin(workgroup_id) work_group_id: vec3<u32>,
        @builtin(global_invocation_id)   global_id: vec3<u32>, 
        @builtin(subgroup_size) sg_size : u32,
        @builtin(subgroup_invocation_id) sg_id : u32) {

	// The unit number of warps.
	// The workgroup_size must be multiple of sg_size.
	let UNIT_SIZE = 128u / sg_size;

	// Warps per dispatch. This could be read from uniform variable.
	let NUM_OF_WARPS = 32u;

        // Pre calculated number of for loop iteratios that are needed.	
	// We want to process certain number of subgroups.
	let FOR_LOOP_ITERATIONS = NUM_OF_WARPS / UNIT_SIZE + 1u;

	// Total number number of items for this dispatch.
	let DISPATCH_OFFSET: u32 = NUM_OF_WARPS * sg_size;

        // Avoid index out ot bound. TODO: check this later. This is not correct way to do this.
        if (global_id.x >= scan_params.data_size) {
	    return;
	}

	// The index for block.
	let warp_id: u32 = global_id.x / sg_size; // TODO: bit operations.

	// The sum of 1-bits per subgroup.
        var one_bits: u32;

	// The total sum of all bits in the whole group.
	var group_sum = 0u;
	var mask: vec4<u32>;

	// TODO: when testing predicate, test indices. If indices is out of bounds => false.
	for (var i=0u ; i < FOR_LOOP_ITERATIONS ; i++) {
	    
	    let input_data_index = DISPATCH_OFFSET * work_group_id.x + 128u * i + sg_id;  

	    let temp_mask: vec4<u32> = subgroupBallot(input_array[input_data_index].test_value < 0.5); // Predicate for this.

            // Store the sum of subgroup bitmast one bits.
	    if (sg_id == 1u) {
	    	one_bits = countOneBits(temp_mask.x);
	    	one_bits += countOneBits(temp_mask.y);
                mask = temp_mask;
            }
        }

	// Store all predicates at the same time.
        predicate_array[warp_id * sg_size + sg_id] = mask; 

	// Reduction for subgroup sum.
	// Calculate the sum of all one bits.

	for (var offset = sg_size >> 1u ; offset > 0u; offset >>= 1u) {
            group_sum += subgroupShuffleDown(group_sum, offset);
	}

	// Finally store the total sum.
	if (sg_id == 0u) {
	    count_array[warp_id] = group_sum;
	}
}
