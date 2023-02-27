use nb_theory::generate_prime;
use num_bigint::BigUint;
use num_bigint_dig::{BigUint as BigUintDig, ModInverse};
use num_traits::ToPrimitive;
use poly::{Poly, PolyContext, Representation};
use std::sync::Arc;

mod nb_theory;
mod poly;

/// Stores all the pre-computation
/// values.
///
/// 1. Poly Contexts of all levels
/// 2. pre-computations at all level
/// 3.
struct BfvParameters {
    ciphertext_moduli: Vec<u64>,
    ciphertext_moduli_sizes: Vec<usize>,
    pub ciphertext_poly_contexts: Vec<Arc<PolyContext>>,

    pub plaintext_modulus: u64,

    // Encryption
    ql_modt: Vec<u64>,
    neg_t_inv_modql: Vec<Poly>,

    // Decryption
    pub t_qlhat_inv_modql_divql_modt: Vec<Vec<u64>>,
    pub t_bqlhat_inv_modql_divql_modt: Vec<Vec<u64>>,
    pub t_qlhat_inv_modql_divql_frac: Vec<Vec<f64>>,
    pub t_bqlhat_inv_modql_divql_frac: Vec<Vec<f64>>,
    pub max_bit_size_by2: usize,
}

impl BfvParameters {
    /// creates new bfv parameteres with necessary values
    pub fn new(
        ciphertext_moduli_sizes: &[usize],
        plaintext_modulus: u64,
        polynomial_degree: usize,
    ) -> BfvParameters {
        // generate primes
        let mut ciphertext_moduli = vec![];
        ciphertext_moduli_sizes.iter().for_each(|size| {
            let mut upper_bound = 1u64 << size;
            loop {
                if let Some(prime) =
                    generate_prime(*size, 2 * polynomial_degree as u64, upper_bound)
                {
                    if !ciphertext_moduli.contains(&prime) {
                        ciphertext_moduli.push(prime);
                        break;
                    } else {
                        upper_bound = prime;
                    }
                } else {
                    // not enough primes
                    assert!(false);
                }
            }
        });

        // create contexts for all levels
        let moduli_count = ciphertext_moduli.len();
        let mut poly_contexts = vec![];
        for i in 0..moduli_count {
            let moduli_at_level = ciphertext_moduli[..moduli_count - i].to_vec();
            poly_contexts.push(Arc::new(PolyContext::new(
                moduli_at_level.as_slice(),
                polynomial_degree,
            )));
        }

        // ENCRYPTION //
        let mut ql_modt = vec![];
        let mut neg_t_inv_modql = vec![];
        poly_contexts.iter().for_each(|poly_context| {
            let q = poly_context.modulus();
            let q_dig = poly_context.modulus_dig();

            // [Q * t]_t
            ql_modt.push((q % plaintext_modulus).to_u64().unwrap());

            // [(-t)^-1]_Q
            let neg_t_inv_modq = BigUint::from_bytes_le(
                &(&q_dig - plaintext_modulus)
                    .mod_inverse(q_dig)
                    .unwrap()
                    .to_biguint()
                    .unwrap()
                    .to_bytes_le(),
            );
            let mut neg_t_inv_modq = Poly::try_convert_from_biguint(
                &[neg_t_inv_modq],
                poly_context,
                &Representation::Coefficient,
            );
            neg_t_inv_modq.change_representation(Representation::Evaluation);
            neg_t_inv_modql.push(neg_t_inv_modq);
        });

        // DECRYPTION //
        // Pre computation for decryption
        let b = ciphertext_moduli_sizes.iter().max().unwrap() / 2;
        let mut t_qlhat_inv_modql_divql_modt = vec![];
        let mut t_bqlhat_inv_modql_divql_modt = vec![];
        let mut t_qlhat_inv_modql_divql_frac = vec![];
        let mut t_bqlhat_inv_modql_divql_frac = vec![];
        poly_contexts.iter().for_each(|poly_context| {
            let ql = poly_context.modulus();
            let ql_dig = poly_context.modulus_dig();

            let mut rationals = vec![];
            let mut brationals = vec![];
            let mut fractionals = vec![];
            let mut bfractionals = vec![];

            poly_context.moduli.iter().for_each(|qi| {
                // [qihat_inv]_qi
                let qihat_inv = BigUint::from_bytes_le(
                    &(&ql_dig / qi)
                        .mod_inverse(BigUintDig::from(*qi))
                        .unwrap()
                        .to_biguint()
                        .unwrap()
                        .to_bytes_le(),
                );

                // [round((t * qihat_inv_modq) / qi)]_t
                let rational = (((&qihat_inv * plaintext_modulus) / qi) % plaintext_modulus)
                    .to_u64()
                    .unwrap();
                let brational = (((((&qihat_inv * (1u64 << b)) % qi) * plaintext_modulus) / qi)
                    % plaintext_modulus)
                    .to_u64()
                    .unwrap();

                // ((t * qihat_inv_modqi) % qi) / qi
                let fractional = ((&qihat_inv * plaintext_modulus) % qi).to_f64().unwrap()
                    / qi.to_f64().unwrap();
                let bfractional = ((((&qihat_inv * (1u64 << b)) % qi) * plaintext_modulus) % qi)
                    .to_f64()
                    .unwrap()
                    / qi.to_f64().unwrap();

                rationals.push(rational);
                brationals.push(brational);
                fractionals.push(fractional);
                bfractionals.push(bfractional);
            });

            t_qlhat_inv_modql_divql_modt.push(rationals);
            t_bqlhat_inv_modql_divql_modt.push(brationals);
            t_qlhat_inv_modql_divql_frac.push(fractionals);
            t_bqlhat_inv_modql_divql_frac.push(bfractionals)
        });

        BfvParameters {
            ciphertext_moduli,
            ciphertext_moduli_sizes: ciphertext_moduli_sizes.to_vec(),
            ciphertext_poly_contexts: poly_contexts,
            plaintext_modulus,
            ql_modt,
            neg_t_inv_modql,
            t_qlhat_inv_modql_divql_modt,
            t_bqlhat_inv_modql_divql_modt,
            t_qlhat_inv_modql_divql_frac,
            t_bqlhat_inv_modql_divql_frac,
            max_bit_size_by2: b,
        }
    }
}

struct Ciphertext {}

struct SecretKey {}
