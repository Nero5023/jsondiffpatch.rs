pub fn lcs<T: PartialEq>(arr0: &[T], arr1: &[T]) -> Vec<(usize, usize)> {
    let len0 = arr0.len();
    let len1 = arr1.len();
    let mut dp = vec![vec![0; len1 + 1]; len0 + 1];
    for (i, v0) in arr0.iter().enumerate() {
        for (j, v1) in arr1.iter().enumerate() {
            if v0 == v1 {
                dp[i + 1][j + 1] = dp[i][j] + 1;
            } else {
                dp[i + 1][j + 1] = dp[i + 1][j].max(dp[i][j + 1]);
            }
        }
    }

    let mut i = len0;
    let mut j = len1;
    let mut res = vec![];
    while i > 0 && j > 0 {
        if dp[i][j] == dp[i - 1][j] {
            i -= 1;
        } else if dp[i][j] == dp[i][j - 1] {
            j -= 1;
        } else {
            assert!(arr0[i - 1] == arr1[j - 1]);
            res.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        }
    }

    res.reverse();
    res
}

#[cfg(test)]
mod tests {
    use crate::lcs::lcs;

    #[test]
    fn test_lcs() {
        let arr0: Vec<char> = "abcde".chars().collect();
        let arr1: Vec<char> = "ace".chars().collect();
        let res = lcs(&arr0, &arr1);
        assert_eq!(res, vec![(0, 0), (2, 1), (4, 2)]);
    }

    fn str2vec(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn test_lcs_empty_and_another() {
        let lcs1 = lcs(&str2vec(""), &str2vec("abcdef"));
        assert_eq!(lcs1.len(), 0);
        let lcs2 = lcs(&str2vec("abcdef"), &str2vec(""));
        assert_eq!(lcs2.len(), 0);
    }

    #[test]
    fn test_lcs_two_same_seq() {
        let s = "abcde";
        let lcs_vec = lcs(&str2vec(s), &str2vec(s));
        assert_eq!(lcs_vec, vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)]);
    }

    #[test]
    fn test_lcs_prefix_seq() {
        let s = "abcdef";
        let prefix = "abc";
        let lcs1 = lcs(&str2vec(s), &str2vec(prefix));
        assert_eq!(lcs1, vec![(0, 0), (1, 1), (2, 2)]);

        let lcs2 = lcs(&str2vec(prefix), &str2vec(s));
        assert_eq!(lcs2, vec![(0, 0), (1, 1), (2, 2)]);
    }

    #[test]
    fn test_lcs_no_common_seq() {
        let lcs_vec = lcs(&str2vec("abcdef"), &str2vec("ghijkl"));
        assert_eq!(lcs_vec.len(), 0);
    }

    #[test]
    fn test_lcs_same_in_mid() {
        let s1 = "abcdefgh";
        let s2 = "bdeg";
        let lcs_vec = lcs(&str2vec(s1), &str2vec(s2));
        assert_eq!(lcs_vec, vec![(1, 0), (3, 1), (4, 2), (6, 3)]);
    }

    #[test]
    fn test_lcs_repeat_elem() {
        let s1 = "abcbdbebf";
        let s2 = "bbbb";
        let lcs_vec = lcs(&str2vec(s1), &str2vec(s2));
        assert_eq!(lcs_vec, vec![(1, 0), (3, 1), (5, 2), (7, 3)]);
    }

    #[test]
    fn test_lcs_non_unique_elem() {
        let s1 = "abcdabcd";
        let s2 = "gabhakbf";
        let lcs_vec = lcs(&str2vec(s1), &str2vec(s2));
        assert_eq!(lcs_vec, vec![(0, 1), (1, 2), (4, 4), (5, 6)]);
    }

    #[test]
    fn test_lcs_common_pre_suf() {
        let s1 = "abctotodef";
        let s2 = "abctatatadef";
        let lcs_vec = lcs(&str2vec(s1), &str2vec(s2));
        assert_eq!(
            lcs_vec,
            vec![
                (0, 0),
                (1, 1),
                (2, 2),
                (3, 3),
                (5, 5),
                (7, 9),
                (8, 10),
                (9, 11)
            ]
        );
    }
}
