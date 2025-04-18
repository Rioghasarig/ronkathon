
use crate::lattice_signature::reduce::reduce32;

const N: usize = 128;

#[derive(Debug, Clone, Copy)]
struct Poly {
    coeffs: [i32; N],
}

fn poly_reduce(a : &mut Poly) {  
    for i in 0..N {
        a.coeffs[i] = reduce32(a.coeffs[i]);
    }
}


#[test]
fn test() {
    let a: i32 = -7; 
    println!("Output: {}", a>> 31);
}