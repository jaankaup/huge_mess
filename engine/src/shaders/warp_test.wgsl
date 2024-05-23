struct PrefixParams {
    data_start_index: u32,
    data_end_index: u32,
    exclusive_parts_start_index: u32,
    exclusive_parts_end_index: u32,
    stage: u32,
};

struct FmmBlock {
    index: u32,
    band_points_count: u32,
};

@group(0)
@binding(0)
var<uniform> fmm_prefix_params: PrefixParams;

@group(0)
@binding(1)
var<storage, read_write> fmm_blocks: array<FmmBlock>;

@compute
@workgroup_size(64,1,1)
fn main(@builtin(local_invocation_id)    local_id: vec3<u32>,
        @builtin(local_invocation_index) local_index: u32,
        @builtin(workgroup_id) work_group_id: vec3<u32>,
        @builtin(global_invocation_id)   global_id: vec3<u32>, 
        @builtin(subgroup_size) sg_size : u32,
        @builtin(subgroup_invocation_id) sg_id : u32) {

	subgroupBallot(local_id.x < 10);
	subgroupBarrier();

        bool jep = subgroupAll(true);
        bool jep2 = subgroupAny(true);

        subgroupAdd(1);
	subgroupMul(3);
        subgroupMax(15);
        subgroupMin(10);
        subgroupAnd(0xfu);
        subgroupOr(0x0f0u);
        subgroupXor(0xf00u);
        subgroupExclusiveAdd(5);
        subgroupExclusiveMul(7);
        subgroupInclusiveAdd(3);
        subgroupInclusiveMul(10);

        subgroupBroadcastFirst(5);
	subgroupBroadcast(123, 10u);
        subgroupShuffle(5, 5u);
        subgroupShuffleDown(5, 5u);
        subgroupShuffleUp(1, 3u);
        subgroupShuffleXor(1, 3u);
	
}
