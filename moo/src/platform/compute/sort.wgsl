struct SortParams {
    num_elements: u32,
    block_height: u32,  // For local sort steps
    block_width: u32,   // For flip/disperse
    algo: u32,          // 0 = Local Bitonic, 1 = Big Flip, 2 = Big Disperse
}

struct GridPair {
    cell_id: u32,
    particle_id: u32,
}

@group(0) @binding(0) var<uniform> params: SortParams;
@group(0) @binding(1) var<storage, read_write> data: array<GridPair>;

@compute @workgroup_size(256)
fn bitonic_sort(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // This shader implements one step of the Bitonic Sort.
    // The CPU must dispatch this multiple times with varying block parameters.
    
    let i = global_id.x;
    let n = params.num_elements;
    
    // We process pairs, so we need i and i^constant
    // But standard bitonic is usually:
    // for k = 2, 4, ... n
    //   for j = k/2 ... 1
    //     parallel check swap
    
    // Here we assume the dispatch is set up such that each thread handles ONE comparison (2 elements).
    // Or each thread handles one element and finds its partner?
    // Let's assume thread i handles element i? No, thread i handles one SWAP operation?
    // Usually: Global Thread ID x maps to an Element Index?
    
    // Simplified Bitonic Step:
    // 3 Parameters define the step:
    // h = block_height (current sorted sequence size being built)
    // algo = phase (e.g. Local vs Global)
    
    // Actually, let's use the standard "Bitonic Merge Sort" single step kernel.
    // j = block_width (distance of compare)
    // k = block_height (monotonic sequence length * 2)
    
    // But WGPU needs synchronization.
    // Strategy: Each Dispatch is ONE stage of the sort.
    
    let j = params.block_width; 
    let k = params.block_height;
    
    // Logic:
    let ixj = i ^ j; // XOR to find partner
    
    if (ixj > i) {
        if (i >= n || ixj >= n) { return; }
        
        let a = data[i];
        let b = data[ixj];
        
        // Direction?
        // (i & k) == 0 means UP, else DOWN
        let dir = (i & k) == 0u;
        
        if (dir) {
            // Ascending
            if (a.cell_id > b.cell_id) {
                // Swap
                data[i] = b;
                data[ixj] = a;
            }
        } else {
            // Descending
            if (a.cell_id < b.cell_id) {
                // Swap
                data[i] = b;
                data[ixj] = a;
            }
        }
    }
}
