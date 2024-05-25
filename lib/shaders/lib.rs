use cgmath::Vector3;

const SCALE: u32 = 1;

#[allow(dead_code)]
fn get_child_pos(parent_pos: &Vector3<u32>, child_rel_pos: &Vector3<u32>) -> Vector3<u32> {
    parent_pos * 2 + child_rel_pos
}

#[allow(dead_code)]
fn get_unique_index(pos: &Vector3<u32>, i: u32) -> u32 {
    let d = 1 << (i + 1);
    pos.x + pos.y * d + pos.z * d * d
}

#[allow(dead_code)]
fn get_pos_from_grid_pos(pos: &Vector3<u32>, i: u32) -> Vector3<f32> {
    let s = SCALE as f32;
    let d = (1 << i) as f32;
    let scale: f32 = s / d;
    let offset: f32 = if i == 0 { 0.0 } else { scale * 0.5 };

    return (pos.cast::<f32>().unwrap() * offset) - Vector3::new(offset, offset, offset);
}

#[allow(dead_code)]
fn count_octrees_below(cd: u32, i: u32) -> u32 {
    ((8_u32.pow(i + 1) as f64) / 7.0 - (8_u32.pow(cd + 1) as f64) / 7.0) as u32
}


#[cfg(test)]
mod tests {
    use crate::shaders::compute::calculate_max_voxel;

    use super::*;
    use std::collections::HashSet;

    fn test_dim(dim: u32) {
        let parent_depth = dim;
        let mut positions = HashSet::new();

        for parent_x in 0..1 << parent_depth {
            for parent_y in 0..1 << parent_depth {
                for parent_z in 0..1 << parent_depth {
                    let parent_pos = Vector3::new(parent_x, parent_y, parent_z);
                    for child_x in 0..2 {
                        for child_y in 0..2 {
                            for child_z in 0..2 {
                                let child_rel_pos = Vector3::new(child_x, child_y, child_z);
                                let child_pos = get_child_pos(&parent_pos, &child_rel_pos);
                                assert!(child_pos.x < (1 << (parent_depth + 1)));
                                assert!(child_pos.y < (1 << (parent_depth + 1)));
                                assert!(child_pos.z < (1 << (parent_depth + 1)));
                                assert!(positions.insert(child_pos));
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(
            positions.len(),
            ((1 << (parent_depth + 1)) as f32).powf(3.0) as usize
        );
    }

    fn test_dim_index(dim: u32) {
        let parent_depth = dim;
        let mut indices = HashSet::new();

        for parent_x in 0..1 << parent_depth {
            for parent_y in 0..1 << parent_depth {
                for parent_z in 0..1 << parent_depth {
                    let parent_pos = Vector3::new(parent_x, parent_y, parent_z);
                    for child_x in 0..2 {
                        for child_y in 0..2 {
                            for child_z in 0..2 {
                                let child_rel_pos = Vector3::new(child_x, child_y, child_z);
                                let child_pos = get_child_pos(&parent_pos, &child_rel_pos);

                                assert!(child_pos.x < (1 << (parent_depth + 1)));
                                assert!(child_pos.y < (1 << (parent_depth + 1)));
                                assert!(child_pos.z < (1 << (parent_depth + 1)));

                                let index = get_unique_index(&child_pos, parent_depth + 1);

                                println!("index: {}, pos: {:?}", index, child_pos);

                                assert!(indices.insert(index));
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(
            indices.len(),
            ((1 << (parent_depth + 1)) as f32).powf(3.0) as usize
        );
    }

    fn test_pos(dim: u32) {
        let parent_depth = dim;
        let mut positions = vec![];

        for parent_x in 0..1 << parent_depth {
            for parent_y in 0..1 << parent_depth {
                for parent_z in 0..1 << parent_depth {
                    let parent_pos = Vector3::new(parent_x, parent_y, parent_z);
                    for child_x in 0..2 {
                        for child_y in 0..2 {
                            for child_z in 0..2 {
                                let child_rel_pos = Vector3::new(child_x, child_y, child_z);
                                let child_pos = get_child_pos(&parent_pos, &child_rel_pos);

                                let pos = get_pos_from_grid_pos(&child_pos, parent_depth + 1);

                                let max_width: f32 = (SCALE / (1 << parent_depth + 1)) as f32;
                                let min_scale = (SCALE as f32 / 2.0) as f32 - (max_width / 2.0);

                                println!("pos: {:?}, cpos: {:?}", pos, child_pos);
                                println!("max_width: {}, scalediv2: {}", max_width, min_scale);

                                assert!(pos.x < min_scale);
                                assert!(pos.y < min_scale);
                                assert!(pos.z < min_scale);
                                assert!(pos.x > -min_scale);
                                assert!(pos.y > -min_scale);
                                assert!(pos.z > -min_scale);

                                positions.push(pos);
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(
            positions.len(),
            ((1 << (parent_depth + 1)) as f32).powf(3.0) as usize
        );
    }

    fn assert_eq_print(i: u32, x: u32) {

        println!("actual: {}, expected: {}", i, x);
        assert!(i == x);
    }

    #[test]
    fn test_uniqueness_at_depth_1() {
        test_dim(1);
        test_dim_index(1);
        test_pos(1);
    }

    #[test]
    fn test_uniqueness_at_depth_2() {
        test_dim(2);
        test_dim_index(2);
        test_pos(2);
    }

    #[test]
    fn test_uniqueness_at_depth_3() {
        test_dim(3);
        test_dim_index(3);
        test_pos(3);
    }

    #[test]
    fn test_uniqueness_at_depth_4() {
        test_dim(4);
        test_dim_index(4);
        test_pos(4);
    }

    #[test]
    fn test_dimension_indexes() {
        assert_eq_print(count_octrees_below(1, 2), 64);
        assert_eq_print(count_octrees_below(0, 2), 72);
        assert_eq_print(count_octrees_below(2, 2), 0);
    }

    #[test]
    fn test_voxel_() {
        assert_eq_print(calculate_max_voxel(1), 64);
        assert_eq_print(calculate_max_voxel(2), 512);
    }
}
