//! RSA signing and verification implementation.
//! Supports PKCS#1 and PKCS#8 PEM private key loading, SHA1WithRSA signing,
//! and RSA public key signature verification.

use crate::error::TigerError;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rsa::pkcs1v15::{SigningKey, VerifyingKey, Signature};
use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs8::DecodePublicKey;
use sha1::Sha1;
use signature::{Signer, SignatureEncoding, Verifier};

/// 加载 RSA 私钥，支持以下格式：
/// - PKCS#1 PEM（BEGIN RSA PRIVATE KEY）
/// - PKCS#8 PEM（BEGIN PRIVATE KEY）
/// - 裸 Base64 编码的 DER 数据（无 PEM 头尾）
pub fn load_private_key(key_str: &str) -> Result<RsaPrivateKey, TigerError> {
    if key_str.is_empty() {
        return Err(TigerError::Auth("私钥不能为空".to_string()));
    }

    // 尝试 PKCS#1 PEM 格式
    if let Ok(key) = <RsaPrivateKey as rsa::pkcs1::DecodeRsaPrivateKey>::from_pkcs1_pem(key_str) {
        return Ok(key);
    }

    // 尝试 PKCS#8 PEM 格式
    if let Ok(key) = <RsaPrivateKey as rsa::pkcs8::DecodePrivateKey>::from_pkcs8_pem(key_str) {
        return Ok(key);
    }

    // 尝试裸 Base64 编码的 DER 数据
    if let Ok(der_bytes) = BASE64.decode(key_str.trim()) {
        // 先尝试 PKCS#1 DER
        if let Ok(key) =
            <RsaPrivateKey as rsa::pkcs1::DecodeRsaPrivateKey>::from_pkcs1_der(&der_bytes)
        {
            return Ok(key);
        }
        // 再尝试 PKCS#8 DER
        if let Ok(key) =
            <RsaPrivateKey as rsa::pkcs8::DecodePrivateKey>::from_pkcs8_der(&der_bytes)
        {
            return Ok(key);
        }
    }

    Err(TigerError::Auth(
        "无法解析私钥：不是有效的 PKCS#1、PKCS#8 PEM 或 Base64 格式".to_string(),
    ))
}

/// 使用 RSA 私钥对内容进行 SHA1WithRSA 签名，返回 Base64 编码的签名字符串。
pub fn sign_with_rsa(private_key_str: &str, content: &str) -> Result<String, TigerError> {
    let private_key = load_private_key(private_key_str)?;
    let signing_key = SigningKey::<Sha1>::new(private_key);
    let signature = signing_key
        .sign(content.as_bytes());
    Ok(BASE64.encode(signature.to_bytes()))
}

/// Verify a SHA1WithRSA signature using an RSA public key.
/// `public_key_str`: Base64-encoded DER public key (PKCS#8 SubjectPublicKeyInfo format)
/// `content`: the original content that was signed
/// `signature_b64`: Base64-encoded signature to verify
/// Returns Ok(true) if verification succeeds, or an error if it fails.
pub fn verify_with_rsa(public_key_str: &str, content: &str, signature_b64: &str) -> Result<bool, TigerError> {
    if public_key_str.is_empty() {
        return Err(TigerError::Auth("public key must not be empty".to_string()));
    }
    if signature_b64.is_empty() {
        return Err(TigerError::Auth("signature must not be empty".to_string()));
    }

    // Decode the Base64 public key to DER bytes
    let der_bytes = BASE64.decode(public_key_str.trim())
        .map_err(|e| TigerError::Auth(format!("failed to decode public key base64: {}", e)))?;

    // Parse as PKCS#8 SubjectPublicKeyInfo DER
    let public_key = RsaPublicKey::from_public_key_der(&der_bytes)
        .map_err(|e| TigerError::Auth(format!("failed to parse public key: {}", e)))?;

    // Decode the Base64 signature
    let sig_bytes = BASE64.decode(signature_b64.trim())
        .map_err(|e| TigerError::Auth(format!("failed to decode signature base64: {}", e)))?;

    let verifying_key = VerifyingKey::<Sha1>::new(public_key);
    let signature = Signature::try_from(sig_bytes.as_slice())
        .map_err(|e| TigerError::Auth(format!("invalid signature format: {}", e)))?;

    verifying_key.verify(content.as_bytes(), &signature)
        .map_err(|e| TigerError::Auth(format!("response signature verification failed: {}", e)))?;

    Ok(true)
}

#[path = "signer_test.rs"]
#[cfg(test)]
mod signer_test;
