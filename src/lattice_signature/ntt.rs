use crate::{Monomial, Polynomial};
use crate::PrimeField;
use crate::algebra::field::FiniteField;
use rand::Rng;

pub enum LatticeParams {
    Prime = 65537 ,
    Gen = 282, 
    Degree = 256,
    K = 24,
    M = 4, 
    S = 127, 
}

pub type NTTField = PrimeField<{LatticeParams::Prime as usize}>;
pub type NTTPoly = Polynomial<Monomial, NTTField, {LatticeParams::Degree as usize}>; 
pub type NTTVec = [NTTPoly; LatticeParams::M as usize]; 

pub fn poly_ntt(a : &mut NTTPoly) {
    ntt::<{LatticeParams::Degree as usize}, NTTField>(&mut a.coefficients, NTTField::new(LatticeParams::Gen as usize)); 
}

pub fn poly_intt(a: &mut NTTPoly) {
    intt::<{LatticeParams::Degree as usize}, NTTField>(&mut a.coefficients, NTTField::new(LatticeParams::Gen as usize)); 
}

pub fn rand_poly(a : &mut NTTPoly, upper : usize) {

    let d = LatticeParams::Degree as usize; 
    for i in 0..d {
        let mut rng = rand::thread_rng();
        let r = rng.gen_range(0..upper);
        a.coefficients[i] = NTTField::new(r);
    }
}

pub fn rand_vec(a : &mut NTTVec, upper : usize) {
    let m = LatticeParams::M as usize; 
    for i in 0..m {
        rand_poly(&mut a[i], upper);
    }
}

pub fn mult_array(a: &[NTTField ; {LatticeParams::Degree as usize}], b: &[NTTField ; {LatticeParams::Degree as usize}], c: &mut [NTTField ; {LatticeParams::Degree as usize}] )
{
    let d = LatticeParams::Degree as usize;
    for i in 0..d {
        c[i] = a[i]*b[i]
    }
}

// pub fn ntt_multiply(a: &mut NTTPoly, b: &mut NTTPoly, out: &mut NTTPoly) {
//     poly_ntt(&mut a);
//     poly_ntt(&mut b);
//     mult_array(&a.coefficients, &b.coefficients, &mut out.coefficients);
//     poly_intt(&mut out); 
// }
// pub fn ntt_vec_dot(a: &NTTVec, b: &NTTVec, c: &mut NTTVec) {
//     let m = LatticeParams::M as usize; 
//     for i in 0..m {
//         poly_ntt(a);
        
//     }
// }
pub fn ntt<const N: usize, F: FiniteField>(arr:  &mut [F; N], gen : F) {
    let deg_d = N; 
    assert!(deg_d.is_power_of_two(), "Length must be a power of two.");

    let nbits = deg_d.trailing_zeros() as usize;

    // Precompute powers of gen 
    let mut gens = Vec::with_capacity(nbits);
    for i in 0..nbits {
        let exp = 1usize << i; // 2^i
        gens.push(gen.pow(exp));
    }
    // The index for which root-of-unity we use:
    let mut gen_ptr = 0;

    //  Iteratively do the Gentleman–Sande butterfly.
    //  Start with stride = deg_d / 2, half the length.
    let mut stride = deg_d >> 1;
    while stride > 0 {
        let step = stride << 1;
        // println!("\nStride: {}", stride);
        // println!("arr: {:?}\n", arr);
 
        for start in (0..deg_d).step_by(step) {
            // println!("Start: {}", start);
            for i in start..(start + stride) {
                let zp = gens[gen_ptr].pow(i-start); 

                let a_val = arr[i];
                let b_val = arr[i + stride]; 
                
                arr[i] = a_val + b_val; 
                arr[i + stride] = zp*(a_val  - b_val);  
                // println!("({},{})  arr: ({:?},{:?}) zp: {:?}, j: {}", i, i + stride, arr[i],arr[i+stride], zp, j);
            }
        }

        stride >>= 1;
        gen_ptr += 1
    }
}

// pub fn ntt_dot(a: &mut NTTVec, b: &mut NTTVec) -> NTTPoly {
//     let m = LatticeParams::M as usize; 
//     for i in 0..m {
//         poly_ntt(a[i]);
//         poly_ntt(b[i]);
//     }
// }
pub fn intt<const N: usize, F : FiniteField>(arr:  &mut [F; N], gen : F) {
    let deg_d = N; 
    assert!(deg_d.is_power_of_two(), "Length must be a power of two.");


    let nbits = deg_d.trailing_zeros() as usize;

    // invert gen 
    let gen_inv = gen.inverse().unwrap(); 
    
    // Precompute powers of gen
    let mut gens = Vec::with_capacity(nbits);
    for i in 0..nbits {
        let exp = 1usize << i; // 2^i
        gens.push(gen_inv.pow(exp));
    }
    //println!("Gens: {:?}\n", gens);

    // The index for which root-of-unity we use:
    let mut gen_ptr = nbits;

    //  Iteratively do the Gentleman–Sande butterfly.
    //  Start with stride = deg_d / 2, half the length.
    let mut stride = 1; 
    while stride < deg_d {
        gen_ptr -= 1;
        // println!("\nStride: {}", stride);
        // println!("arr: {:?}\n", arr);

        let step = stride << 1; 
        for start in (0..deg_d).step_by(step) {
            // println!("Start: {}", start);
            for i in start..(start + stride) {
                let j = i - start; 
                let zp = gens[gen_ptr].pow(j); 

                let a_val = arr[i];
                let b_val = arr[i + stride]; 
                
                arr[i] = a_val + zp*b_val; 
                arr[i + stride] = a_val  - zp*b_val; 
                 
                // println!("({},{})  arr: ({:?},{:?}) zp: {:?}, j: {}", i, i + stride, arr[i],arr[i+stride], zp, j);

            }
        }

        stride <<= 1;

    }

    let scaler = F::from(deg_d);

    for i in 0..(deg_d) { 
        arr[i] = arr[i]/scaler; 
    }
}


// fn id_scheme() {
//     let mut s: NTTVec; 
//     rand_vec(&mut s, S);
//     let mut h: NTTVec;
//     rand_vec(&mut h, LatticeParams::Prime as usize - 1);

// }
#[test]
fn test_ntt() {
    let mut a = NTTPoly::new([NTTField::new(0); {LatticeParams::Degree as usize}]);
    rand_poly(&mut a,LatticeParams::Prime as usize);
    let b = a; 

    poly_ntt(&mut a);

    poly_intt(&mut a);

    assert!(a == b,  "Polynomials must be equal");

}