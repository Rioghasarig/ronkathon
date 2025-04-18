import math
def pow(base: int, exp: int, modulus: int) -> int:
    result = 1 % modulus  # Ensure result starts within modulo space
    base %= modulus  # Reduce base modulo first
    
    while exp > 0:
        if (exp & 1) == 1:  # If the least significant bit is 1
            result = (result * base) % modulus
        base = (base * base) % modulus  # Square base and reduce mod
        exp >>= 1  # Right shift exponent (divide by 2)
    
    return result

def ntt_iter(a, gen=13, modulus=17):
  deg_d = len(a)

  # Start with stride = 1.
  stride = 1

  # Shuffle the input array in bit-reversal order.
  nbits = int(math.log2(deg_d))
  res = [a[i] for i in range(deg_d)]

  # Pre-compute the generators used in different stages of the recursion.
  gen = pow(gen, deg_d-1, modulus)
  gens = [pow(gen, 2**i, modulus) for i in range(nbits)]
  # The first layer uses the lowest (2nd) root of unity, hence the last one.
  gen_ptr = len(gens) - 1

  # Iterate until the last layer.
  while stride < deg_d:
    # For each stride, iterate over all N//(stride*2) slices.
    print('Stride: ', stride)
    for start in range(0, deg_d, stride * 2):
      # For each pair of the CT butterfly operation.
      for i in range(start, start + stride):
        # Compute the omega multiplier. Here j = i - start.
        zp = pow(gens[gen_ptr], i - start, modulus)
        # Cooley-Tukey butterfly.
        a = res[i]
        b = res[i+stride]
        res[i] = (a + zp * b) % modulus
        res[i+stride] = (a - zp * b) % modulus
        print(f'({i},{i+stride}) res: ({res[i]},{res[i+stride]}) zp: {zp}')
    # Grow the stride.
    stride <<= 1
    # Move to the next root of unity.
    gen_ptr -= 1

  return res

a = [2, 5, 3, 7]
a_ntt_iter = ntt_iter(a)
print(a_ntt_iter)
