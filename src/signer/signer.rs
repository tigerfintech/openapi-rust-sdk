//! RSA 签名功能实现。
//! 支持 PKCS#1 和 PKCS#8 PEM 格式私钥加载，以及 SHA1WithRSA 签名。

use crate::error::TigerError;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rsa::pkcs1v15::SigningKey;
use rsa::RsaPrivateKey;
use sha1::Sha1;
use signature::{Signer, SignatureEncoding};

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

#[path = "signer_test.rs"]
#[cfg(test)]
mod signer_test;
