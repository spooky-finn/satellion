/// Selects a subset of `values` whose sum meets `target`.
///
/// Tries the Knapsack DP solver first to find an exact match (no change
/// output). Falls back to largest-first when no exact solution exists or the
/// target is too large for the DP table.
///
/// Returns the original indices of the chosen values, or an empty Vec if the
/// total available funds are insufficient.
pub fn select(values: &[u64], target: u64) -> Vec<usize> {
    if target == 0 || values.is_empty() {
        return vec![];
    }

    // Sort largest-first. The knapsack result is order-independent, but
    // sorting means the largest-first fallback picks the fewest inputs.
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(values[i]));
    let sorted: Vec<u64> = order.iter().map(|&i| values[i]).collect();

    // Bail out early if even spending every coin falls short.
    if sorted.iter().sum::<u64>() < target {
        return vec![];
    }

    // Try to find a combination that sums to exactly `target` (Knapsack DP).
    // If that search is skipped or finds no solution, fall back to the simpler
    // largest-first strategy, which always succeeds but may overshoot
    // (producing a change output).
    let sorted_indices = KnapsackSolver::new(&sorted, target)
        .solve()
        .unwrap_or_else(|| largest_first(&sorted, target));

    // Translate sorted-array indices back to the original caller indices.
    sorted_indices.into_iter().map(|i| order[i]).collect()
}

/// Picks values one by one from largest to smallest until `target` is covered.
/// Always terminates with a valid selection; may include more value than needed.
fn largest_first(sorted_values: &[u64], target: u64) -> Vec<usize> {
    let mut result = vec![];
    let mut accumulated = 0u64;
    for (i, &val) in sorted_values.iter().enumerate() {
        result.push(i);
        accumulated += val;
        if accumulated >= target {
            break;
        }
    }
    result
}

/// Searches for a subset of coin values that sums to *exactly* `target` using
/// the 0/1 Knapsack dynamic programming algorithm.
///
/// A DP table is built with one cell per satoshi value from 0 up to `target`.
/// Each cell records which coin was last added to reach that sum, enabling
/// exact reconstruction of the chosen set at the end. The table is filled with
/// a standard bottom-up pass:
///
///   for each coin:
///     for each reachable sum (high → low, to avoid reusing the same coin):
///       if (sum - coin_value) was already reachable, mark sum as reachable
///
/// Traversing high-to-low is the key 0/1 trick: it ensures a coin can only
/// extend sums that were established before *this* coin was considered.
///
/// Because the table has one entry per satoshi, its size grows linearly with
/// the target amount. Above `MAX_DP_TARGET` the solver returns `None` to keep
/// memory bounded, and the largest-first fallback takes over.
struct KnapsackSolver {
    values: Vec<u64>,
    target: u64,
}

/// 10 million satoshis (0.1 BTC). The DP table is a `Vec<i32>` of this many
/// entries, costing ~40 MB. Targets above this threshold skip the DP.
const MAX_DP_TARGET: u64 = 10_000_000;

impl KnapsackSolver {
    fn new(values: &[u64], target: u64) -> Self {
        Self { values: values.to_vec(), target }
    }

    fn solve(&self) -> Option<Vec<usize>> {
        if self.target > MAX_DP_TARGET {
            return None;
        }

        let size = self.target as usize + 1;

        // dp[v] = the index of the coin most recently added to reach sum v.
        // -1 means sum v is not yet reachable.
        // i32::MAX is a sentinel for the base case: sum 0, achieved with no coins.
        let mut dp: Vec<i32> = vec![-1; size];
        dp[0] = i32::MAX; // base case: we can always "reach" a sum of zero

        for (i, &val) in self.values.iter().enumerate() {
            // A coin larger than the target can never contribute to an exact
            // sum — skip it so it doesn't pollute the table.
            if val > self.target {
                continue;
            }
            let val = val as usize;

            // Walk from high sums down to `val`. Going high-to-low prevents
            // the same coin from being counted twice in a single pass: when we
            // update dp[v], the dp[v - val] we read was set by an *earlier*
            // coin, not by the current one in this same loop.
            for v in (val..size).rev() {
                if dp[v] == -1 && dp[v - val] != -1 {
                    // Coin i extends a previously reachable sum (v - val) to v.
                    dp[v] = i as i32;
                }
            }
        }

        // If the target cell is still -1, no exact combination exists.
        if dp[self.target as usize] == -1 {
            return None;
        }

        // Reconstruct the solution by walking backwards through the table.
        // Each cell tells us which coin was last added to reach that sum, so
        // we subtract that coin's value and repeat until we reach 0.
        let mut selected = vec![];
        let mut remaining = self.target as usize;
        while remaining > 0 {
            let i = dp[remaining] as usize;
            selected.push(i);
            remaining -= self.values[i] as usize;
        }

        Some(selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn total(values: &[u64], indices: &[usize]) -> u64 {
        indices.iter().map(|&i| values[i]).sum()
    }

    #[test]
    fn zero_target_returns_empty() {
        assert!(select(&[1, 2, 3], 0).is_empty());
    }

    #[test]
    fn insufficient_funds_returns_empty() {
        assert!(select(&[1, 2], 10).is_empty());
    }

    #[test]
    fn exact_match_via_knapsack() {
        let values = [3, 5, 7];
        let indices = select(&values, 8);
        assert_eq!(total(&values, &indices), 8);
        // Knapsack finds [3, 5] — not the greedy [7 + overshoot].
        assert_eq!(indices.len(), 2);
    }

    #[test]
    fn fallback_when_no_exact_match() {
        // [6, 9] — no subset sums to 7; fallback picks [9].
        let values = [6, 9];
        let indices = select(&values, 7);
        assert_eq!(indices.len(), 1);
        assert!(total(&values, &indices) >= 7);
    }

    #[test]
    fn single_utxo_exact() {
        let values = [10];
        let indices = select(&values, 10);
        assert_eq!(indices, vec![0]);
    }

    #[test]
    fn single_utxo_overshoot_fallback() {
        // 10 > 5, so Knapsack finds nothing; fallback returns [10].
        let values = [10];
        let indices = select(&values, 5);
        assert_eq!(indices, vec![0]);
        assert!(total(&values, &indices) >= 5);
    }

    #[test]
    fn knapsack_preferred_over_greedy() {
        // Greedy would pick [10], but Knapsack finds the exact [3, 5].
        let values = [3, 5, 10];
        let indices = select(&values, 8);
        assert_eq!(total(&values, &indices), 8);
        assert_eq!(indices.len(), 2);
    }
}
