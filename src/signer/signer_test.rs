#[cfg(test)]
mod tests {
    use crate::signer::{load_private_key, sign_with_rsa};
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD as BASE64;
    use rsa::RsaPrivateKey;
    use rsa::pkcs1::{EncodeRsaPrivateKey, LineEnding};
    use rsa::pkcs8::EncodePrivateKey;

    /// 生成测试用 RSA 密钥对（PKCS#1 PEM 格式）
    fn generate_pkcs1_pem() -> (String, RsaPrivateKey) {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("生成密钥失败");
        let pem = private_key
            .to_pkcs1_pem(LineEnding::LF)
            .expect("编码 PKCS#1 PEM 失败");
        (pem.to_string(), private_key)
    }

    /// 生成测试用 RSA 密钥对（PKCS#8 PEM 格式）
    fn generate_pkcs8_pem() -> (String, RsaPrivateKey) {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("生成密钥失败");
        let pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .expect("编码 PKCS#8 PEM 失败");
        (pem.to_string(), private_key)
    }

    /// 生成裸 Base64 编码的私钥（无 PEM 头尾，PKCS#1 DER）
    fn generate_raw_base64() -> (String, RsaPrivateKey) {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("生成密钥失败");
        let der = private_key
            .to_pkcs1_der()
            .expect("编码 PKCS#1 DER 失败");
        let raw_base64 = BASE64.encode(der.as_bytes());
        (raw_base64, private_key)
    }

    // ========== load_private_key 单元测试 ==========

    #[test]
    fn test_load_private_key_pkcs1() {
        let (pem, _) = generate_pkcs1_pem();
        let key = load_private_key(&pem).expect("加载 PKCS#1 私钥失败");
        key.validate().expect("私钥验证失败");
    }

    #[test]
    fn test_load_private_key_pkcs8() {
        let (pem, _) = generate_pkcs8_pem();
        let key = load_private_key(&pem).expect("加载 PKCS#8 私钥失败");
        key.validate().expect("私钥验证失败");
    }

    #[test]
    fn test_load_private_key_raw_base64() {
        let (raw, _) = generate_raw_base64();
        let key = load_private_key(&raw).expect("加载裸 Base64 私钥失败");
        key.validate().expect("私钥验证失败");
    }

    #[test]
    fn test_load_private_key_invalid() {
        let result = load_private_key("invalid-key-data");
        assert!(result.is_err(), "加载无效私钥应返回错误");
    }

    #[test]
    fn test_load_private_key_empty() {
        let result = load_private_key("");
        assert!(result.is_err(), "加载空私钥应返回错误");
    }

    // ========== sign_with_rsa 单元测试 ==========

    #[test]
    fn test_sign_with_rsa_pkcs1() {
        let (pem, private_key) = generate_pkcs1_pem();
        let content = "tiger_id=test123&timestamp=1234567890";

        let signature = sign_with_rsa(&pem, content).expect("签名失败");
        assert!(!signature.is_empty(), "签名结果不应为空");

        // 验证签名是有效的 Base64
        let sig_bytes = BASE64.decode(&signature).expect("签名结果不是有效的 Base64");

        // 使用公钥验签
        use rsa::pkcs1v15::VerifyingKey;
        use sha1::Sha1;
        use signature::Verifier;
        let public_key = private_key.to_public_key();
        let verifying_key = VerifyingKey::<Sha1>::new(public_key);
        let sig = rsa::pkcs1v15::Signature::try_from(sig_bytes.as_slice()).expect("签名格式错误");
        verifying_key
            .verify(content.as_bytes(), &sig)
            .expect("验签失败");
    }

    #[test]
    fn test_sign_with_rsa_pkcs8() {
        let (pem, private_key) = generate_pkcs8_pem();
        let content = "biz_content={}&method=market_state";

        let signature = sign_with_rsa(&pem, content).expect("签名失败");
        assert!(!signature.is_empty(), "签名结果不应为空");

        // 使用公钥验签
        use rsa::pkcs1v15::VerifyingKey;
        use sha1::Sha1;
        use signature::Verifier;
        let public_key = private_key.to_public_key();
        let verifying_key = VerifyingKey::<Sha1>::new(public_key);
        let sig_bytes = BASE64.decode(&signature).expect("Base64 解码失败");
        let sig = rsa::pkcs1v15::Signature::try_from(sig_bytes.as_slice()).expect("签名格式错误");
        verifying_key
            .verify(content.as_bytes(), &sig)
            .expect("验签失败");
    }

    #[test]
    fn test_sign_with_rsa_different_content_different_signature() {
        let (pem, _) = generate_pkcs1_pem();
        let sig1 = sign_with_rsa(&pem, "content1").expect("签名 content1 失败");
        let sig2 = sign_with_rsa(&pem, "content2").expect("签名 content2 失败");
        assert_ne!(sig1, sig2, "不同内容的签名不应相同");
    }

    #[test]
    fn test_sign_with_rsa_invalid_key() {
        let result = sign_with_rsa("invalid-key", "test content");
        assert!(result.is_err(), "使用无效私钥签名应返回错误");
    }
}

// ========== Property 4 属性测试：RSA 签名-验签 round-trip ==========
#[cfg(test)]
mod property_tests {
    use crate::signer::sign_with_rsa;
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD as BASE64;
    use proptest::prelude::*;
    use rsa::RsaPrivateKey;
    use rsa::pkcs1::{EncodeRsaPrivateKey, LineEnding};

    /// 生成一个固定的测试密钥对（属性测试中避免每次都生成密钥，太慢）
    fn test_key_pair() -> (String, RsaPrivateKey) {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("生成密钥失败");
        let pem = private_key
            .to_pkcs1_pem(LineEnding::LF)
            .expect("编码 PEM 失败");
        (pem.to_string(), private_key)
    }

    // Feature: multi-language-sdks, Property 4: RSA 签名-验签 round-trip
    // **Validates: Requirements 3.2**
    //
    // 对于任意非空字符串内容和有效的 RSA 密钥对，使用私钥对内容进行
    // SHA1WithRSA 签名后，使用对应公钥验签应成功。
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]
        #[test]
        fn rsa_sign_verify_roundtrip(content in ".{1,200}") {
            let (pem, private_key) = test_key_pair();

            // 签名
            let signature = sign_with_rsa(&pem, &content).expect("签名失败");
            prop_assert!(!signature.is_empty(), "签名结果不应为空");

            // 验证签名是有效的 Base64
            let sig_bytes = BASE64.decode(&signature).expect("签名结果不是有效的 Base64");

            // 使用公钥验签
            use rsa::pkcs1v15::VerifyingKey;
            use sha1::Sha1;
            use signature::Verifier;
            let public_key = private_key.to_public_key();
            let verifying_key = VerifyingKey::<Sha1>::new(public_key);
            let sig = rsa::pkcs1v15::Signature::try_from(sig_bytes.as_slice()).expect("签名格式错误");
            verifying_key
                .verify(content.as_bytes(), &sig)
                .map_err(|e| TestCaseError::Fail(format!("验签失败: {}", e).into()))?;
        }
    }
}
