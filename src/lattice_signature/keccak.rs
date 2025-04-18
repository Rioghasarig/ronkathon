/// Example configuration constants to match the C macros.
/// You can adjust these if you have a different rate or output-length requirement.
const C_KECCAK_B: usize = 1600; // The Keccak-f width.
const C_KECCAK_R: usize = 1088; // The 'rate' in bits; typical for 512-bit capacity.
const C_KECCAK_R_SIZE_IN_BYTES: usize = C_KECCAK_R / 8;
const C_KECCAK_NUMBER_OF_ROUNDS: usize = 24;

// If you want a "fixed" output length, set it here, up to C_KECCAK_R_SIZE_IN_BYTES. 
// For example, 32 bytes for a 256-bit hash:
const C_KECCAK_FIXED_OUTPUT_LENGTH_IN_BYTES: usize = 32;

// The C code sets crypto_hash_BYTES depending on macros:
const CRYPTO_HASH_BYTES: usize = C_KECCAK_FIXED_OUTPUT_LENGTH_IN_BYTES;
// Ensure we don't exceed the 'rate' for the one-shot hash:
static_assertions::const_assert!(CRYPTO_HASH_BYTES <= C_KECCAK_R_SIZE_IN_BYTES);

/// For C_KECCAK_B = 1600, we use `u64` lanes.
type TKeccakLane = u64;

/// Rotate left. This replicates the C macro:
/// ```c
/// #define ROL(a, offset) ( ((a) << ((offset) % 64)) ^ ((a) >> (64 - ((offset) % 64))) )
/// ```
#[inline]
fn rol(a: u64, offset: u64) -> u64 {
    // offset % 64 is done explicitly below, 
    // because shifting by >= 64 in Rust causes a panic in debug builds.
    let s = offset % 64;
    (a << s) ^ (a >> (64 - s))
}

/// For the C code's ROL_mult8 macro:
/// ```c
/// #if ((cKeccakB/25) == 8)
///   #define ROL_mult8(a, offset) (a)
/// #else
///   #define ROL_mult8(a, offset) ROL(a, offset)
/// #endif
/// ```
/// For cKeccakB=1600, (1600/25) = 64, so we must use `rol`.
#[inline]
fn rol_mult8(a: u64, offset: u64) -> u64 {
    rol(a, offset)
}

/// Round constants for Keccak-f. This matches the C array.
static KECCAKF_ROUND_CONSTANTS: [u64; C_KECCAK_NUMBER_OF_ROUNDS] = [
    0x0000000000000001u64,
    0x0000000000008082u64,
    0x800000000000808au64,
    0x8000000080008000u64,
    0x000000000000808bu64,
    0x0000000080000001u64,
    0x8000000080008081u64,
    0x8000000000008009u64,
    0x000000000000008au64,
    0x0000000000000088u64,
    0x0000000080008009u64,
    0x000000008000000au64,
    0x000000008000808bu64,
    0x800000000000008bu64,
    0x8000000000008089u64,
    0x8000000000008003u64,
    0x8000000000008002u64,
    0x8000000000000080u64,
    0x000000000000800au64,
    0x800000008000000au64,
    0x8000000080008081u64,
    0x8000000000008080u64,
    0x0000000080000001u64,
    0x8000000080008008u64,
];

/// The core Keccak-f permutation.
/// This function combines absorbing `laneCount` lanes from `input` into `state`
/// and then runs the permutation on `state`.
#[allow(non_snake_case)]
pub fn keccak_f(state: &mut [TKeccakLane], input: &[TKeccakLane], lane_count: usize) {
    // Absorb step: XOR the input lanes into `state`.
    {
        let mut i = lane_count;
        while i > 0 {
            let idx = i - 1;
            state[idx] ^= input[idx];
            i -= 1;
        }
    }

    // Permutation step.
    // We break it out by named variables, exactly like the original code.
    {
        let (
            mut Aba, mut Abe, mut Abi, mut Abo, mut Abu,
            mut Aga, mut Age, mut Agi, mut Ago, mut Agu,
            mut Aka, mut Ake, mut Aki, mut Ako, mut Aku,
            mut Ama, mut Ame, mut Ami, mut Amo, mut Amu,
            mut Asa, mut Ase, mut Asi, mut Aso, mut Asu
        ): (u64, u64, u64, u64, u64,
            u64, u64, u64, u64, u64,
            u64, u64, u64, u64, u64,
            u64, u64, u64, u64, u64,
            u64, u64, u64, u64, u64);

        let (
            mut Eba, mut Ebe, mut Ebi, mut Ebo, mut Ebu,
            mut Ega, mut Ege, mut Egi, mut Ego, mut Egu,
            mut Eka, mut Eke, mut Eki, mut Eko, mut Eku,
            mut Ema, mut Eme, mut Emi, mut Emo, mut Emu,
            mut Esa, mut Ese, mut Esi, mut Eso, mut Esu
        ): (u64, u64, u64, u64, u64,
            u64, u64, u64, u64, u64,
            u64, u64, u64, u64, u64,
            u64, u64, u64, u64, u64,
            u64, u64, u64, u64, u64);
        
        let (
            mut BCa, mut BCe, mut BCi, mut BCo, mut BCu
        ): (u64, u64, u64, u64, u64);

        let (
            mut Da, mut De, mut Di, mut Do, mut Du
        ): (u64, u64, u64, u64, u64);

        // Load from state
        Aba = state[ 0];
        Abe = state[ 1];
        Abi = state[ 2];
        Abo = state[ 3];
        Abu = state[ 4];
        Aga = state[ 5];
        Age = state[ 6];
        Agi = state[ 7];
        Ago = state[ 8];
        Agu = state[ 9];
        Aka = state[10];
        Ake = state[11];
        Aki = state[12];
        Ako = state[13];
        Aku = state[14];
        Ama = state[15];
        Ame = state[16];
        Ami = state[17];
        Amo = state[18];
        Amu = state[19];
        Asa = state[20];
        Ase = state[21];
        Asi = state[22];
        Aso = state[23];
        Asu = state[24];

        // Round loop (unrolled two at a time, as in the C code)
        let mut round = 0;
        while round < C_KECCAK_NUMBER_OF_ROUNDS {
            // --- First half of the 2-round unroll ---

            // prepareTheta
            BCa = Aba ^ Aga ^ Aka ^ Ama ^ Asa;
            BCe = Abe ^ Age ^ Ake ^ Ame ^ Ase;
            BCi = Abi ^ Agi ^ Aki ^ Ami ^ Asi;
            BCo = Abo ^ Ago ^ Ako ^ Amo ^ Aso;
            BCu = Abu ^ Agu ^ Aku ^ Amu ^ Asu;

            // thetaRhoPiChiIotaPrepareTheta(round, A, E)
            Da = BCu ^ rol(BCe, 1);
            De = BCa ^ rol(BCi, 1);
            Di = BCe ^ rol(BCo, 1);
            Do = BCi ^ rol(BCu, 1);
            Du = BCo ^ rol(BCa, 1);

            Aba ^= Da;
            BCa = Aba;
            Age ^= De;
            BCe = rol(Age, 44);
            Aki ^= Di;
            BCi = rol(Aki, 43);
            Amo ^= Do;
            BCo = rol(Amo, 21);
            Asu ^= Du;
            BCu = rol(Asu, 14);

            // Chi + Iota
            Eba = BCa ^ ((!BCe) & BCi);
            Eba = Eba ^ KECCAKF_ROUND_CONSTANTS[round];
            Ebe = BCe ^ ((!BCi) & BCo);
            Ebi = BCi ^ ((!BCo) & BCu);
            Ebo = BCo ^ ((!BCu) & BCa);
            Ebu = BCu ^ ((!BCa) & BCe);

            Abo ^= Do;
            BCa = rol(Abo, 28);
            Agu ^= Du;
            BCe = rol(Agu, 20);
            Aka ^= Da;
            BCi = rol(Aka,  3);
            Ame ^= De;
            BCo = rol(Ame, 45);
            Asi ^= Di;
            BCu = rol(Asi, 61);

            Ega = BCa ^ ((!BCe) & BCi);
            Ege = BCe ^ ((!BCi) & BCo);
            Egi = BCi ^ ((!BCo) & BCu);
            Ego = BCo ^ ((!BCu) & BCa);
            Egu = BCu ^ ((!BCa) & BCe);

            Abe ^= De;
            BCa = rol(Abe,  1);
            Agi ^= Di;
            BCe = rol(Agi,  6);
            Ako ^= Do;
            BCi = rol(Ako, 25);
            Amu ^= Du;
            BCo = rol_mult8(Amu, 8);
            Asa ^= Da;
            BCu = rol(Asa, 18);

            Eka = BCa ^ ((!BCe) & BCi);
            Eke = BCe ^ ((!BCi) & BCo);
            Eki = BCi ^ ((!BCo) & BCu);
            Eko = BCo ^ ((!BCu) & BCa);
            Eku = BCu ^ ((!BCa) & BCe);

            Abu ^= Du;
            BCa = rol(Abu, 27);
            Aga ^= Da;
            BCe = rol(Aga, 36);
            Ake ^= De;
            BCi = rol(Ake, 10);
            Ami ^= Di;
            BCo = rol(Ami, 15);
            Aso ^= Do;
            BCu = rol_mult8(Aso, 56);

            Ema = BCa ^ ((!BCe) & BCi);
            Eme = BCe ^ ((!BCi) & BCo);
            Emi = BCi ^ ((!BCo) & BCu);
            Emo = BCo ^ ((!BCu) & BCa);
            Emu = BCu ^ ((!BCa) & BCe);

            Abi ^= Di;
            BCa = rol(Abi, 62);
            Ago ^= Do;
            BCe = rol(Ago, 55);
            Aku ^= Du;
            BCi = rol(Aku, 39);
            Ama ^= Da;
            BCo = rol(Ama, 41);
            Ase ^= De;
            BCu = rol(Ase,  2);

            Esa = BCa ^ ((!BCe) & BCi);
            Ese = BCe ^ ((!BCi) & BCo);
            Esi = BCi ^ ((!BCo) & BCu);
            Eso = BCo ^ ((!BCu) & BCa);
            Esu = BCu ^ ((!BCa) & BCe);

            // --- Second half of the 2-round unroll: use E* as input, A* as output ---

            // prepareTheta
            BCa = Eba ^ Ega ^ Eka ^ Ema ^ Esa;
            BCe = Ebe ^ Ege ^ Eke ^ Eme ^ Ese;
            BCi = Ebi ^ Egi ^ Eki ^ Emi ^ Esi;
            BCo = Ebo ^ Ego ^ Eko ^ Emo ^ Eso;
            BCu = Ebu ^ Egu ^ Eku ^ Emu ^ Esu;

            Da = BCu ^ rol(BCe, 1);
            De = BCa ^ rol(BCi, 1);
            Di = BCe ^ rol(BCo, 1);
            Do = BCi ^ rol(BCu, 1);
            Du = BCo ^ rol(BCa, 1);

            Eba = Eba ^ Da;
            BCa = Eba;
            Ege = Ege ^ De;
            BCe = rol(Ege, 44);
            Eki = Eki ^ Di;
            BCi = rol(Eki, 43);
            Emo = Emo ^ Do;
            BCo = rol(Emo, 21);
            Esu = Esu ^ Du;
            BCu = rol(Esu, 14);

            Aba = BCa ^ ((!BCe) & BCi);
            Aba = Aba ^ KECCAKF_ROUND_CONSTANTS[round + 1];
            Abe = BCe ^ ((!BCi) & BCo);
            Abi = BCi ^ ((!BCo) & BCu);
            Abo = BCo ^ ((!BCu) & BCa);
            Abu = BCu ^ ((!BCa) & BCe);

            Ebo = Ebo ^ Do;
            BCa = rol(Ebo, 28);
            Egu = Egu ^ Du;
            BCe = rol(Egu, 20);
            Eka = Eka ^ Da;
            BCi = rol(Eka, 3);
            Eme = Eme ^ De;
            BCo = rol(Eme, 45);
            Esi = Esi ^ Di;
            BCu = rol(Esi, 61);

            Aga = BCa ^ ((!BCe) & BCi);
            Age = BCe ^ ((!BCi) & BCo);
            Agi = BCi ^ ((!BCo) & BCu);
            Ago = BCo ^ ((!BCu) & BCa);
            Agu = BCu ^ ((!BCa) & BCe);

            Ebe = Ebe ^ De;
            BCa = rol(Ebe, 1);
            Egi = Egi ^ Di;
            BCe = rol(Egi, 6);
            Eko = Eko ^ Do;
            BCi = rol(Eko, 25);
            Emu = Emu ^ Du;
            BCo = rol_mult8(Emu, 8);
            Esa = Esa ^ Da;
            BCu = rol(Esa, 18);

            Aka = BCa ^ ((!BCe) & BCi);
            Ake = BCe ^ ((!BCi) & BCo);
            Aki = BCi ^ ((!BCo) & BCu);
            Ako = BCo ^ ((!BCu) & BCa);
            Aku = BCu ^ ((!BCa) & BCe);

            Ebu = Ebu ^ Du;
            BCa = rol(Ebu, 27);
            Ega = Ega ^ Da;
            BCe = rol(Ega, 36);
            Eke = Eke ^ De;
            BCi = rol(Eke, 10);
            Emi = Emi ^ Di;
            BCo = rol(Emi, 15);
            Eso = Eso ^ Do;
            BCu = rol_mult8(Eso, 56);

            Ama = BCa ^ ((!BCe) & BCi);
            Ame = BCe ^ ((!BCi) & BCo);
            Ami = BCi ^ ((!BCo) & BCu);
            Amo = BCo ^ ((!BCu) & BCa);
            Amu = BCu ^ ((!BCa) & BCe);

            Ebi = Ebi ^ Di;
            BCa = rol(Ebi, 62);
            Ego = Ego ^ Do;
            BCe = rol(Ego, 55);
            Eku = Eku ^ Du;
            BCi = rol(Eku, 39);
            Ema = Ema ^ Da;
            BCo = rol(Ema, 41);
            Ese = Ese ^ De;
            BCu = rol(Ese, 2);

            Asa = BCa ^ ((!BCe) & BCi);
            Ase = BCe ^ ((!BCi) & BCo);
            Asi = BCi ^ ((!BCo) & BCu);
            Aso = BCo ^ ((!BCu) & BCa);
            Asu = BCu ^ ((!BCa) & BCe);


            round += 2;

            // Now we feed these new A values into the next iteration if not done.
        }

        // Final copy to `state`.
        state[ 0] = Aba;
        state[ 1] = Abe;
        state[ 2] = Abi;
        state[ 3] = Abo;
        state[ 4] = Abu;
        state[ 5] = Aga;
        state[ 6] = Age;
        state[ 7] = Agi;
        state[ 8] = Ago;
        state[ 9] = Agu;
        state[10] = Aka;
        state[11] = Ake;
        state[12] = Aki;
        state[13] = Ako;
        state[14] = Aku;
        state[15] = Ama;
        state[16] = Ame;
        state[17] = Ami;
        state[18] = Amo;
        state[19] = Amu;
        state[20] = Asa;
        state[21] = Ase;
        state[22] = Asi;
        state[23] = Aso;
        state[24] = Asu;
    }
}

/// This function mirrors the original `int crypto_hash(...)`.
/// It consumes `input` of length `inlen`, hashes it (Keccak with rate `C_KECCAK_R`),
/// and writes `CRYPTO_HASH_BYTES` bytes to `out`.
///
/// Returns 0 on success, matching the C code convention.
pub fn crypto_hash(
    out: &mut [u8],
    input: &[u8],
) -> i32 {
    assert!(out.len() >= CRYPTO_HASH_BYTES);

    // Keccak state is 25 lanes of TKeccakLane (u64 for cKeccakB=1600)
    let mut state = [0u64; 25];

    // A temporary buffer for the last partial block.
    let mut temp = [0u8; C_KECCAK_R_SIZE_IN_BYTES];

    let mut inlen = input.len();
    let mut offset = 0;

    // Process as many full blocks of size `C_KECCAK_R_SIZE_IN_BYTES` as possible
    while inlen >= C_KECCAK_R_SIZE_IN_BYTES {
        // Safe slice of the input block
        let block = &input[offset..offset + C_KECCAK_R_SIZE_IN_BYTES];

        // XOR into the state
        // We reinterpret the block as an array of TKeccakLane (u64)
        let lane_count = C_KECCAK_R_SIZE_IN_BYTES / std::mem::size_of::<TKeccakLane>();
        let in_lanes: &[u64] = unsafe {
            std::slice::from_raw_parts(
                block.as_ptr() as *const u64,
                lane_count
            )
        };

        keccak_f(&mut state, in_lanes, lane_count);

        offset += C_KECCAK_R_SIZE_IN_BYTES;
        inlen  -= C_KECCAK_R_SIZE_IN_BYTES;
    }

    // --- Padding the last partial block ---
    // Copy whatever remains to temp
    temp[..inlen].copy_from_slice(&input[offset..offset + inlen]);

    // Add the '01' byte
    temp[inlen] = 0x01;
    let mut inlen = inlen + 1;

    // Zero the rest
    for b in &mut temp[inlen..] {
        *b = 0;
    }

    // Add the '80' bit in the very last byte
    temp[C_KECCAK_R_SIZE_IN_BYTES - 1] |= 0x80;

    // Absorb final block
    let lane_count = C_KECCAK_R_SIZE_IN_BYTES / std::mem::size_of::<TKeccakLane>();
    let in_lanes: &[u64] = unsafe {
        std::slice::from_raw_parts(
            temp.as_ptr() as *const u64,
            lane_count
        )
    };
    keccak_f(&mut state, in_lanes, lane_count);

    // Copy the first CRYPTO_HASH_BYTES of the state as the hash output
    // Reinterpret the `state` array of u64 as bytes
    let state_bytes = unsafe {
        std::slice::from_raw_parts(
            state.as_ptr() as *const u8,
            25 * std::mem::size_of::<u64>()
        )
    };
    out[..CRYPTO_HASH_BYTES].copy_from_slice(&state_bytes[..CRYPTO_HASH_BYTES]);

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keccak_crypto_hash_length() {
        let mut out = [0u8; CRYPTO_HASH_BYTES];
        let msg = b"abc";
        let r = crypto_hash(&mut out, msg);
        assert_eq!(r, 0);
        // You can print or compare out[] to known test vectors.
    }
}
