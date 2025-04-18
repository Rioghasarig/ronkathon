use ronkathon::{
  algebra::{field::prime::PlutoScalarField, group::FiniteCyclicGroup},
  curve::{pluto_curve::PlutoBaseCurve, AffinePoint},
  diffie_hellman::ecdh::compute_shared_secret,
};

const G: AffinePoint<PlutoBaseCurve> = AffinePoint::GENERATOR;

fn main() {
  let alice_secret = PlutoScalarField::new(420);
  let bob_secret = PlutoScalarField::new(69);

  let alice_public = G * alice_secret;
  let bob_public = G * bob_secret;

  let shared_secret_alice = bob_public * alice_secret; // (G * bob_secret) * alice_secret
  let shared_secret_bob = alice_public * bob_secret;// (G * alice_secret) * bob_secret 

  assert_eq!(shared_secret_alice, shared_secret_bob);

  println!("Shared secret: {:#?} {:#?}", shared_secret_alice, shared_secret_bob);
  //println!("G: {:#?}", G);

  // let mut H = G; 
  // println!("H: {:#?}", H); 
  // for i in 1..33{
  //   H = G + H;
  //   println!("{}, H: {:#?}", i+1, H);  
  // }

  // Since G has order 17 , G*n is 
  // Discrete Log example solve: 5^x = 3 mod 61 
  // Elliptic curve 'discrete log' (given H and G) solve G*n  = H 

  // public_info = f(secret_info)
}
