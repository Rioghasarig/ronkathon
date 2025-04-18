const Q: i32 = 8380417;
pub fn  reduce32(a: i32) -> i32 {
    let  t = (a + (1 << 22)) >> 23;
    return a - t*Q;
}

pub fn caddq(a: i32) -> i32 {
    let t = a + (a >> 31) & Q;
    return t;
}