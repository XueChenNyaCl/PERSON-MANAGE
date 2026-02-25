use bcrypt::{hash, verify, DEFAULT_COST};
use anyhow::anyhow;

pub fn hash_password(password: &str) -> Result<String, anyhow::Error> {
    hash(password, DEFAULT_COST).map_err(|e| anyhow!(e))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, anyhow::Error> {
    verify(password, hash).map_err(|e| anyhow!(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "test_password_123";
        let hash_result = hash_password(password);
        assert!(hash_result.is_ok());
        
        let hash = hash_result.unwrap();
        let verify_result = verify_password(password, &hash);
        assert!(verify_result.is_ok());
        assert!(verify_result.unwrap());
        
        // 验证错误密码
        let wrong_password = "wrong_password";
        let verify_wrong_result = verify_password(wrong_password, &hash);
        assert!(verify_wrong_result.is_ok());
        assert!(!verify_wrong_result.unwrap());
    }
}