//! Chia cột cho khối sự kiện trùng giờ: (start-min, end-min) -> (col, cols).

pub fn assign(ranges: &[(i32, i32)]) -> Vec<(i32, i32)> {
    let mut order: Vec<usize> = (0..ranges.len()).collect();
    order.sort_by_key(|&i| ranges[i]);
    let mut out = vec![(0, 1); ranges.len()];
    let mut cluster: Vec<usize> = vec![];
    let mut ends: Vec<i32> = vec![]; // end của mỗi cột trong cụm
    let mut cluster_end = i32::MIN;
    for i in order {
        let (s, e) = ranges[i];
        if s >= cluster_end {
            close(&mut cluster, &mut ends, &mut out);
        }
        let col = ends.iter().position(|&end| end <= s).unwrap_or(ends.len());
        if col == ends.len() {
            ends.push(e);
        } else {
            ends[col] = e;
        }
        out[i].0 = col as i32;
        cluster.push(i);
        cluster_end = if cluster.len() == 1 { e } else { cluster_end.max(e) };
    }
    close(&mut cluster, &mut ends, &mut out);
    out
}

fn close(cluster: &mut Vec<usize>, ends: &mut Vec<i32>, out: &mut [(i32, i32)]) {
    for &i in cluster.iter() {
        out[i].1 = ends.len() as i32;
    }
    cluster.clear();
    ends.clear();
}
