use crate::utlities::hash;
use crate::utlities::HASH_OUTPUT_BIT_SIZE;
use crate::ProofError;
use curv::arithmetic::traits::Modulo;
use curv::arithmetic::traits::Samplable;
use curv::arithmetic::traits::ZeroizeBN;
use curv::BigInt;
use elgamal::ElGamalPP;

/// This is implementation of Schnorr's identification protocol for elliptic curve groups or a
/// sigma protocol for Proof of knowledge of the discrete log of an Elliptic-curve point:
/// C.P. Schnorr. Efficient Identification and Signatures for Smart Cards. In
/// CRYPTO 1989, Springer (LNCS 435), pages 239–252, 1990.
/// https://pdfs.semanticscholar.org/8d69/c06d48b618a090dd19185aea7a13def894a5.pdf.
///
/// The protocol is using Fiat-Shamir Transform: Amos Fiat and Adi Shamir.
/// How to prove yourself: Practical solutions to identification and signature problems.
/// In Advances in Cryptology - CRYPTO ’86, Santa Barbara, California, USA, 1986, Proceedings,
/// pages 186–194, 1986.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DLogProof {
    pub random_point: BigInt,
    pub response: BigInt,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Witness {
    pub x: BigInt,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Statement {
    pub h: BigInt,
}

pub trait ProveDLog {
    fn prove(witness: &Witness, pp: &ElGamalPP) -> DLogProof;

    fn verify(&self, statement: &Statement, pp: &ElGamalPP) -> Result<(), ProofError>;
}

impl ProveDLog for DLogProof {
    fn prove(w: &Witness, pp: &ElGamalPP) -> DLogProof {
        let mut r: BigInt = BigInt::sample_below(&pp.q);
        let random_point = BigInt::mod_pow(&pp.g, &r, &pp.p);
        let pk = BigInt::mod_pow(&pp.g, &w.x, &pp.p);
        let e = hash(&[&random_point, &pk, &pp.g], &pp.q, HASH_OUTPUT_BIT_SIZE);
        let response = &r + &(e * &w.x);
        r.zeroize_bn();
        DLogProof {
            random_point,
            response,
        }
    }

    fn verify(&self, statement: &Statement, pp: &ElGamalPP) -> Result<(), ProofError> {
        let e = hash(
            &[&self.random_point, &statement.h, &pp.g],
            &pp.q,
            HASH_OUTPUT_BIT_SIZE,
        );

        let z = self.response.modulus(&pp.q);
        let pk_e = BigInt::mod_pow(&statement.h, &e, &pp.p);
        let pk_e_random_point = BigInt::mod_mul(&self.random_point, &pk_e, &pp.p);
        let g_z = BigInt::mod_pow(&pp.g, &z, &pp.p);

        if g_z == pk_e_random_point {
            Ok(())
        } else {
            Err(ProofError::DlogProofError)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utlities::dlog_proof::DLogProof;
    use crate::utlities::dlog_proof::{ProveDLog, Statement, Witness};
    use crate::utlities::hash;
    use curv::BigInt;
    use elgamal::rfc7919_groups::SupportedGroups;
    use elgamal::ElGamalKeyPair;
    use elgamal::ElGamalPP;

    #[test]
    fn test_dlog_proof() {
        let pp = ElGamalPP::generate_from_rfc7919(SupportedGroups::FFDHE2048);
        let keypair = ElGamalKeyPair::generate(&pp);
        let witness = Witness { x: keypair.sk.x };
        let statement = Statement { h: keypair.pk.h };

        let dlog_proof = DLogProof::prove(&witness, &pp);
        let verified = dlog_proof.verify(&statement, &pp);
        match verified {
            Ok(_t) => assert!(true),
            Err(_e) => assert!(false),
        }
    }

    #[test]
    fn test_hash() {
        let pp = ElGamalPP::generate_from_rfc7919(SupportedGroups::FFDHE2048);
        let res = hash(&[&BigInt::from(1)], &pp.q, 256);
        assert!(
            (res.bit_length() - pp.q.bit_length()) < 10
                || (pp.q.bit_length() - res.bit_length()) < 10
        );
    }
}
