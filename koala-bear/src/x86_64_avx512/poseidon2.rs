use core::arch::x86_64::__m512i;

use p3_monty_31::{
    add, halve_avx512, mul_neg_2_exp_neg_8_avx512, mul_neg_2_exp_neg_n_avx512,
    mul_neg_2_exp_neg_two_adicity_avx512, sub, InternalLayerParametersAVX512,
};

use crate::{KoalaBearInternalLayerParameters, KoalaBearParameters};

impl InternalLayerParametersAVX512<16> for KoalaBearInternalLayerParameters {
    type ArrayLike = [__m512i; 15];

    /// For the KoalaBear field and width 16 we multiply by the diagonal matrix:
    /// D = [-2, 1, 2, 1/2, 3, 4, -1/2, -3, -4, 1/2^8, -1/2^8, 1/8, -1/8, -1/16, 1/2^24, -1/2^24].
    /// The inputs must be in canonical form, otherwise the result is undefined.
    /// Even when the inputs are in canonical form, we make no guarantees on the output except that, provided
    /// the output is piped directly into add_sum the vector will be modified such that x[i] = D[i]*x[i] + sum.
    #[inline(always)]
    unsafe fn diagonal_mul(input: &mut [__m512i; 15]) {
        // As far as we know this is optimal in that it need the fewest instructions to perform all of these
        // multiplications. (Note that -1, 0 are not allowed on the diagonal for technical reasons).
        // If there exist other number b for which x*b mod P can be computed quickly this diagonal can be updated.

        // The strategy is very simple. 2, 3, 4, -3, -4 are implemented using addition.
        //                              1/2, -1/2 using the custom half function.
        //                              and the remainder utilizing the custom functions for multiplication by 2^{-n}.

        // Note that for -3, -4, -1/2 we actually output 3x, 4x, x/2 and the negative is dealt with in add_sum by subtracting
        // this from the summation instead of adding it.

        // Note that input only contains the last 15 elements of the state.
        // The first element is handled separately as we need to apply the s-box to it.

        // input[0] is being multiplied by 1 so we can also ignore it.

        // input[1]-> sum + 2*input[1]
        input[1] = add::<KoalaBearParameters>(input[1], input[1]);

        // input[2]-> sum + input[2]/2
        input[2] = halve_avx512::<KoalaBearParameters>(input[2]);

        // input[3]-> sum + 3*input[3]
        let acc3 = add::<KoalaBearParameters>(input[3], input[3]);
        input[3] = add::<KoalaBearParameters>(acc3, input[3]);

        // input[4]-> sum + 4*input[4]
        let acc4 = add::<KoalaBearParameters>(input[4], input[4]);
        input[4] = add::<KoalaBearParameters>(acc4, acc4);

        // input[5]-> sum - input[5]/2
        input[5] = halve_avx512::<KoalaBearParameters>(input[5]);

        // input[6]-> sum - 3*input[6]
        let acc6 = add::<KoalaBearParameters>(input[6], input[6]);
        input[6] = add::<KoalaBearParameters>(acc6, input[6]);

        // input[7]-> sum - 4*input[7]
        let acc7 = add::<KoalaBearParameters>(input[7], input[7]);
        input[7] = add::<KoalaBearParameters>(acc7, acc7);

        // input[8]-> sum + input[8]/2^8
        // This outputs the negative of what we want. This will be handled in add_sum.
        input[8] = mul_neg_2_exp_neg_8_avx512::<KoalaBearParameters, 16>(input[8]);

        // input[9] -> sum - input[9]/2^8
        input[9] = mul_neg_2_exp_neg_8_avx512::<KoalaBearParameters, 16>(input[9]);

        // input[10] -> sum + input[10]/2^3
        // This outputs the negative of what we want. This will be handled in add_sum.
        input[10] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 3, 21>(input[10]);

        // input[11] -> sum - input[11]/2^3
        input[11] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 3, 21>(input[11]);

        // input[12] -> sum - input[12]/2^4
        input[12] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 4, 20>(input[12]);

        // input[13] -> sum + input[13]/2^24
        // This outputs the negative of what we want. This will be handled in add_sum.
        input[13] = mul_neg_2_exp_neg_two_adicity_avx512::<KoalaBearParameters, 24, 7>(input[13]);

        // input[14] -> sum - input[14]/2^24
        input[14] = mul_neg_2_exp_neg_two_adicity_avx512::<KoalaBearParameters, 24, 7>(input[14]);
    }

    /// Add sum to every element of input.
    /// Sum must be in canonical form and input must be exactly the output of diagonal mul.
    /// If either of these does not hold, the result is undefined.
    unsafe fn add_sum(input: &mut [__m512i; 15], sum: __m512i) {
        input[..5]
            .iter_mut()
            .for_each(|x| *x = add::<KoalaBearParameters>(sum, *x));

        // Diagonal mul multiplied these by 1/2, 3, 4 instead of -1/2, -3, -4 so we need to subtract instead of adding.
        input[5..8]
            .iter_mut()
            .for_each(|x| *x = sub::<KoalaBearParameters>(sum, *x));

        // We either add sum or subtract from sum depending on if diagonal mul returned the positive or negative.
        // We are breaking the canonical assumption as input lies in [0, P] and not [0, P) but we are doing it
        // in a way allowed by the add and sum specifications.
        input[8] = sub::<KoalaBearParameters>(sum, input[8]);
        input[9] = add::<KoalaBearParameters>(sum, input[9]);
        input[10] = sub::<KoalaBearParameters>(sum, input[10]);
        input[11] = add::<KoalaBearParameters>(sum, input[11]);
        input[12] = add::<KoalaBearParameters>(sum, input[12]);
        input[13] = sub::<KoalaBearParameters>(sum, input[13]);
        input[14] = add::<KoalaBearParameters>(sum, input[14]);
    }
}

impl InternalLayerParametersAVX512<24> for KoalaBearInternalLayerParameters {
    type ArrayLike = [__m512i; 23];

    /// For the KoalaBear field and width 16 we multiply by the diagonal matrix:
    /// D = [-2, 1, 2, 1/2, 3, 4, -1/2, -3, -4, 1/2^8, -1/2^8, 1/4, 1/8, -1/8, 1/16, -1/16, 1/32, -1/32, 1/64, -1/64, -1/2^7, -1/2^9, 1/2**24, -1/2**24]
    /// The inputs must be in canonical form, otherwise the result is undefined.
    /// Even when the inputs are in canonical form, we make no guarantees on the output except that, provided
    /// the output is piped directly into add_sum, the vector will be modified such that x[i] = D[i]*x[i] + sum.
    #[inline(always)]
    unsafe fn diagonal_mul(input: &mut [__m512i; 23]) {
        // As far as we know this is optimal in that it need the fewest instructions to perform all of these
        // multiplications. (Note that -1, 0 are not allowed on the diagonal for technical reasons).
        // If there exist other number b for which x*b mod P can be computed quickly this diagonal can be updated.

        // The strategy is very simple. 2, 3, 4, -3, -4 are implemented using addition.
        //                              1/2, -1/2 using the custom half function.
        //                              and the remainder utilizing the custom functions for multiplication by 2^{-n}.

        // Note that for -3, -4, -1/2 we actually output 3x, 4x, x/2 and the negative is dealt with in add_sum by subtracting
        // this from the summation instead of adding it.

        // Note that input only contains the last 23 elements of the state.
        // The first element is handled separately as we need to apply the s-box to it.

        // input[0] is being multiplied by 1 so we can also ignore it.

        // input[1] -> sum + 2*input[1]
        input[1] = add::<KoalaBearParameters>(input[1], input[1]);

        // input[2] -> sum + input[2]/2
        input[2] = halve_avx512::<KoalaBearParameters>(input[2]);

        // input[3] -> sum + 3*input[3]
        let acc3 = add::<KoalaBearParameters>(input[3], input[3]);
        input[3] = add::<KoalaBearParameters>(acc3, input[3]);

        // input[4] -> sum + 4*input[4]
        let acc4 = add::<KoalaBearParameters>(input[4], input[4]);
        input[4] = add::<KoalaBearParameters>(acc4, acc4);

        // input[5] -> sum - input[5]/2
        input[5] = halve_avx512::<KoalaBearParameters>(input[5]);

        // input[6] -> sum - 3*input[6]
        let acc6 = add::<KoalaBearParameters>(input[6], input[6]);
        input[6] = add::<KoalaBearParameters>(acc6, input[6]);

        // input[7] -> sum - 4*input[7]
        let acc7 = add::<KoalaBearParameters>(input[7], input[7]);
        input[7] = add::<KoalaBearParameters>(acc7, acc7);

        // input[8] -> sum + input[8]/2^8
        // This outputs the negative of what we want. This will be handled in add_sum.
        input[8] = mul_neg_2_exp_neg_8_avx512::<KoalaBearParameters, 16>(input[8]);

        // input[9] -> sum - input[9]/2^8
        input[9] = mul_neg_2_exp_neg_8_avx512::<KoalaBearParameters, 16>(input[9]);

        // input[10] -> sum + input[10]/2^2
        // This outputs the negative of what we want. This will be handled in add_sum.
        input[10] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 2, 22>(input[10]);

        // input[11] -> sum + input[11]/2^3
        // This outputs the negative of what we want. This will be handled in add_sum.
        input[11] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 3, 21>(input[11]);

        // input[12] -> sum - input[12]/2^3
        input[12] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 3, 21>(input[12]);

        // input[13] -> sum + input[13]/2^4
        // This outputs the negative of what we want. This will be handled in add_sum.
        input[13] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 4, 20>(input[13]);

        // input[14] -> sum - input[14]/2^4
        input[14] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 4, 20>(input[14]);

        // input[15] -> sum + input[15]/2^5
        // This outputs the negative of what we want. This will be handled in add_sum.
        input[15] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 5, 19>(input[15]);

        // input[16] -> sum - input[16]/2^5
        input[16] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 5, 19>(input[16]);

        // input[17] -> sum + input[17]/2^6
        // This outputs the negative of what we want. This will be handled in add_sum.
        input[17] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 6, 18>(input[17]);

        // input[18] -> sum - input[18]/2^6
        input[18] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 6, 18>(input[18]);

        // input[19] -> sum - input[19]/2^7
        input[19] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 7, 17>(input[19]);

        // input[20] -> sum - input[20]/2^9
        input[20] = mul_neg_2_exp_neg_n_avx512::<KoalaBearParameters, 9, 15>(input[20]);

        // input[21] -> sum + input[21]/2^24
        // This outputs the negative of what we want. This will be handled in add_sum.
        input[21] = mul_neg_2_exp_neg_two_adicity_avx512::<KoalaBearParameters, 24, 7>(input[21]);

        // input[22] -> sum - input[22]/2^24
        input[22] = mul_neg_2_exp_neg_two_adicity_avx512::<KoalaBearParameters, 24, 7>(input[22]);
    }

    /// Add sum to every element of input.
    /// Sum must be in canonical form and input must be exactly the output of diagonal mul.
    /// If either of these does not hold, the result is undefined.
    unsafe fn add_sum(input: &mut [__m512i; 23], sum: __m512i) {
        input[..5]
            .iter_mut()
            .for_each(|x| *x = add::<KoalaBearParameters>(sum, *x));

        // Diagonal mul multiplied these by 1/2, 3, 4 instead of -1/2, -3, -4 so we need to subtract instead of adding.
        input[5..8]
            .iter_mut()
            .for_each(|x| *x = sub::<KoalaBearParameters>(sum, *x));

        // We either add sum or subtract from sum depending on if diagonal mul returned the positive or negative.
        // We are breaking the canonical assumption as input lies in [0, P] and not [0, P) but we are doing it
        // in a way allowed by the add and sum specifications.
        input[8] = sub::<KoalaBearParameters>(sum, input[8]);
        input[9] = add::<KoalaBearParameters>(sum, input[9]);
        input[10] = sub::<KoalaBearParameters>(sum, input[10]);
        input[11] = sub::<KoalaBearParameters>(sum, input[11]);
        input[12] = add::<KoalaBearParameters>(sum, input[12]);
        input[13] = sub::<KoalaBearParameters>(sum, input[13]);
        input[14] = add::<KoalaBearParameters>(sum, input[14]);
        input[15] = sub::<KoalaBearParameters>(sum, input[15]);
        input[16] = add::<KoalaBearParameters>(sum, input[16]);
        input[17] = sub::<KoalaBearParameters>(sum, input[17]);
        input[18] = add::<KoalaBearParameters>(sum, input[18]);
        input[19] = add::<KoalaBearParameters>(sum, input[19]);
        input[20] = add::<KoalaBearParameters>(sum, input[20]);
        input[21] = sub::<KoalaBearParameters>(sum, input[21]);
        input[22] = add::<KoalaBearParameters>(sum, input[22]);
    }
}

#[cfg(test)]
mod tests {
    use p3_field::AbstractField;
    use p3_symmetric::Permutation;
    use rand::Rng;

    use crate::{KoalaBear, PackedKoalaBearAVX512, Poseidon2KoalaBear};

    type F = KoalaBear;
    type Perm16 = Poseidon2KoalaBear<16>;
    type Perm24 = Poseidon2KoalaBear<24>;

    /// Test that the output is the same as the scalar version on a random input.
    #[test]
    fn test_avx512_poseidon2_width_16() {
        let mut rng = rand::thread_rng();

        // Our Poseidon2 implementation.
        let poseidon2 = Perm16::new_from_rng_128(&mut rng);

        let input: [F; 16] = rng.gen();

        let mut expected = input;
        poseidon2.permute_mut(&mut expected);

        let mut avx512_input = input.map(PackedKoalaBearAVX512::from_f);
        poseidon2.permute_mut(&mut avx512_input);

        let avx512_output = avx512_input.map(|x| x.0[0]);

        assert_eq!(avx512_output, expected);
    }

    /// Test that the output is the same as the scalar version on a random input.
    #[test]
    fn test_avx512_poseidon2_width_24() {
        let mut rng = rand::thread_rng();

        // Our Poseidon2 implementation.
        let poseidon2 = Perm24::new_from_rng_128(&mut rng);

        let input: [F; 24] = rng.gen();

        let mut expected = input;
        poseidon2.permute_mut(&mut expected);

        let mut avx512_input = input.map(PackedKoalaBearAVX512::from_f);
        poseidon2.permute_mut(&mut avx512_input);

        let avx512_output = avx512_input.map(|x| x.0[0]);

        assert_eq!(avx512_output, expected);
    }
}
