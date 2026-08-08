//! SHA-256, so the active specification's identity can be derived from its own
//! bytes instead of being taken on trust.
//!
//! The crate has no dependencies and keeps none, so this is written by hand.
//! It is a `const fn` and usable in constant position for small inputs, but the
//! active specification is hashed at runtime: see `computed_active_spec_hash`
//! for why. Names inside the compression function follow FIPS 180-4 section 6.2
//! so the code can be read against the standard.

/// First thirty-two bits of the fractional parts of the cube roots of the
/// first sixty-four primes.
const ROUND: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// First thirty-two bits of the fractional parts of the square roots of the
/// first eight primes.
const INITIAL: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// One 64-byte message block.
const BLOCK: usize = 64;

/// The SHA-256 digest of `message`.
pub const fn digest(message: &[u8]) -> [u8; 32] {
    let mut state = INITIAL;

    let mut offset = 0;
    while offset + BLOCK <= message.len() {
        state = compress(state, message, offset);
        offset += BLOCK;
    }

    // The final blocks hold the bytes past the last whole block, the `0x80`
    // terminator, zero padding, and the 64-bit big-endian bit length. That
    // needs nine free bytes, so a remainder of 55 or less closes in one block
    // and anything longer takes two.
    let remaining = message.len() - offset;
    let mut tail = [0_u8; 2 * BLOCK];
    let mut index = 0;
    while index < remaining {
        tail[index] = message[offset + index];
        index += 1;
    }
    tail[remaining] = 0x80;

    let tail_len = if remaining + 9 <= BLOCK {
        BLOCK
    } else {
        2 * BLOCK
    };
    let encoded = ((message.len() as u64) * 8).to_be_bytes();
    index = 0;
    while index < 8 {
        tail[tail_len - 8 + index] = encoded[index];
        index += 1;
    }

    let mut block = 0;
    while block < tail_len {
        state = compress(state, &tail, block);
        block += BLOCK;
    }

    let mut output = [0_u8; 32];
    let mut word = 0;
    while word < 8 {
        let bytes = state[word].to_be_bytes();
        output[word * 4] = bytes[0];
        output[word * 4 + 1] = bytes[1];
        output[word * 4 + 2] = bytes[2];
        output[word * 4 + 3] = bytes[3];
        word += 1;
    }
    output
}

/// Mix the 64-byte block starting at `offset` into `state`.
const fn compress(state: [u32; 8], buffer: &[u8], offset: usize) -> [u32; 8] {
    let mut schedule = [0_u32; 64];
    let mut index = 0;
    while index < 16 {
        let byte = offset + index * 4;
        schedule[index] = ((buffer[byte] as u32) << 24)
            | ((buffer[byte + 1] as u32) << 16)
            | ((buffer[byte + 2] as u32) << 8)
            | (buffer[byte + 3] as u32);
        index += 1;
    }
    while index < 64 {
        let previous = schedule[index - 15];
        let recent = schedule[index - 2];
        let sigma0 = previous.rotate_right(7) ^ previous.rotate_right(18) ^ (previous >> 3);
        let sigma1 = recent.rotate_right(17) ^ recent.rotate_right(19) ^ (recent >> 10);
        schedule[index] = schedule[index - 16]
            .wrapping_add(sigma0)
            .wrapping_add(schedule[index - 7])
            .wrapping_add(sigma1);
        index += 1;
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    let mut round = 0;
    while round < 64 {
        let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choose = (e & f) ^ (!e & g);
        let first = h
            .wrapping_add(sum1)
            .wrapping_add(choose)
            .wrapping_add(ROUND[round])
            .wrapping_add(schedule[round]);
        let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let second = sum0.wrapping_add(majority);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(first);
        d = c;
        c = b;
        b = a;
        a = first.wrapping_add(second);
        round += 1;
    }

    [
        state[0].wrapping_add(a),
        state[1].wrapping_add(b),
        state[2].wrapping_add(c),
        state[3].wrapping_add(d),
        state[4].wrapping_add(e),
        state[5].wrapping_add(f),
        state[6].wrapping_add(g),
        state[7].wrapping_add(h),
    ]
}

#[cfg(test)]
mod tests {
    use super::digest;

    /// Proves the digest is available in a constant, not only at runtime.
    const ABC: [u8; 32] = digest(b"abc");

    fn hex(bytes: [u8; 32]) -> String {
        let mut text = String::with_capacity(64);
        for byte in bytes {
            text.push_str(&format!("{byte:02x}"));
        }
        text
    }

    /// Published SHA-256 values, so a wrong implementation cannot agree with
    /// itself. Each was reproduced with `shasum -a 256` before it was written
    /// here. The lengths cover the three padding shapes: a remainder that
    /// closes in one block, one that needs a second, and a message whose last
    /// whole block is followed by a short remainder.
    #[test]
    fn published_vectors_agree() {
        assert_eq!(
            hex(digest(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(ABC),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(digest(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            hex(digest(
                b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"
            )),
            "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1"
        );
    }

    /// The many-block path, at a length no const evaluation would want.
    #[test]
    fn one_million_bytes_agree() {
        let message = vec![b'a'; 1_000_000];
        assert_eq!(
            hex(digest(&message)),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    /// Every block length from empty to two full blocks reaches the same
    /// digest as the byte-at-a-time reference below, so no padding boundary is
    /// silently wrong.
    #[test]
    fn every_padding_boundary_is_reached() {
        for length in 0..=(2 * super::BLOCK + 1) {
            let message = vec![b'z'; length];
            assert_eq!(
                digest(&message),
                reference(&message),
                "length {length} disagrees"
            );
        }
    }

    /// An independent, deliberately naive SHA-256: it builds the whole padded
    /// message first, so it shares no padding arithmetic with `digest`.
    fn reference(message: &[u8]) -> [u8; 32] {
        let mut padded = message.to_vec();
        padded.push(0x80);
        while padded.len() % 64 != 56 {
            padded.push(0);
        }
        padded.extend_from_slice(&(message.len() as u64 * 8).to_be_bytes());

        let mut state = super::INITIAL;
        let mut offset = 0;
        while offset < padded.len() {
            state = super::compress(state, &padded, offset);
            offset += 64;
        }

        let mut output = [0_u8; 32];
        for (word, value) in state.iter().enumerate() {
            output[word * 4..word * 4 + 4].copy_from_slice(&value.to_be_bytes());
        }
        output
    }
}
