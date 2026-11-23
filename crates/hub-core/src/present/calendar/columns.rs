//! Chia cột cho các khối lịch trùng giờ.

pub(super) fn assign(ranges: &[(i32, i32)]) -> Vec<(i32, i32)> {
    let mut order: Vec<usize> = (0..ranges.len()).collect();
    order.sort_by_key(|&i| ranges[i]);
    let mut out = vec![(0, 1); ranges.len()];
    let mut cluster: Vec<usize> = vec![];
    let mut ends: Vec<i32> = vec![];
    let mut cluster_end = i32::MIN;
    for i in order {
        let (start, end) = ranges[i];
        if start >= cluster_end {
            close(&mut cluster, &mut ends, &mut out);
        }
        let col = ends
            .iter()
            .position(|&column_end| column_end <= start)
            .unwrap_or(ends.len());
        if col == ends.len() {
            ends.push(end);
        } else {
            ends[col] = end;
        }
        out[i].0 = col as i32;
        cluster.push(i);
        cluster_end = if cluster.len() == 1 {
            end
        } else {
            cluster_end.max(end)
        };
    }
    close(&mut cluster, &mut ends, &mut out);
    out
}

fn close(cluster: &mut Vec<usize>, ends: &mut Vec<i32>, out: &mut [(i32, i32)]) {
    for &index in cluster.iter() {
        out[index].1 = ends.len() as i32;
    }
    cluster.clear();
    ends.clear();
}
