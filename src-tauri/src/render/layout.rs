use super::{BLOCK_GAP, OUTPUT_HEIGHT};

pub struct BlockLayout {
    pub a_x: usize,
    pub a_y: usize,
    pub b_x: usize,
    pub b_y: usize,
    pub block_w: usize,
    pub block_h: usize,
}

impl BlockLayout {
    pub fn for_fixture(index: usize, total: usize) -> Self {
        let safe_total = total.max(1);
        // Distribute height evenly: usable pixels split across all blocks,
        // with the first `extra` blocks getting 1 extra pixel so nothing
        // is wasted at the bottom.
        let usable = OUTPUT_HEIGHT.saturating_sub(safe_total * BLOCK_GAP);
        let base_h = (usable / safe_total).max(1);
        let extra = usable % safe_total;

        // y position: each previous block contributes (GAP + its height)
        let y_before: usize = (0..index)
            .map(|i| BLOCK_GAP + base_h + if i < extra { 1 } else { 0 })
            .sum();
        let y = BLOCK_GAP + y_before;

        let block_h = base_h + if index < extra { 1 } else { 0 };
        let block_w = 42usize;

        Self {
            a_x: BLOCK_GAP,
            a_y: y,
            b_x: BLOCK_GAP + block_w + BLOCK_GAP,
            b_y: y,
            block_w,
            block_h,
        }
    }
}
