use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("Invalid signature format")]
    InvalidFormat,
    #[error("Signature verification failed")]
    VerificationFailed,
    #[error("Public key not found")]
    PublicKeyNotFound,
    #[error("Signature mismatch: expected {expected}, got {actual}")]
    SignatureMismatch { expected: String, actual: String },
}

/// Ed25519 signature for plugin verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSignature {
    pub algorithm: String, // "ed25519"
    pub signature_bytes: Vec<u8>,
    pub public_key: Vec<u8>,
    pub signed_hash: String, // SHA256 hash of plugin archive
    pub timestamp: i64,
    pub signer: String, // Author/publisher name
}

/// Signature verifier for plugin security
pub struct SignatureVerifier {
    trusted_keys: Vec<Vec<u8>>, // List of trusted public keys
}

impl SignatureVerifier {
    pub fn new() -> Self {
        Self {
            trusted_keys: Self::load_default_trusted_keys(),
        }
    }

    pub fn with_trusted_keys(trusted_keys: Vec<Vec<u8>>) -> Self {
        Self { trusted_keys }
    }

    /// Add a trusted public key
    pub fn add_trusted_key(&mut self, public_key: Vec<u8>) {
        if !self.trusted_keys.contains(&public_key) {
            self.trusted_keys.push(public_key);
        }
    }

    /// Verify plugin signature
    pub async fn verify_plugin(&self, plugin_dir: &Path, signature: &PluginSignature) -> Result<bool> {
        // Check if public key is trusted
        if !self.is_key_trusted(&signature.public_key) {
            tracing::warn!("Plugin signed with untrusted key: {}", signature.signer);
            return Ok(false);
        }

        // Calculate hash of plugin archive
        let plugin_hash = self.calculate_plugin_hash(plugin_dir).await?;

        // Verify hash matches
        if plugin_hash != signature.signed_hash {
            return Err(SignatureError::SignatureMismatch {
                expected: signature.signed_hash.clone(),
                actual: plugin_hash,
            }
            .into());
        }

        // Verify Ed25519 signature
        self.verify_ed25519_signature(
            &signature.signature_bytes,
            &signature.public_key,
            signature.signed_hash.as_bytes(),
        )
    }

    /// Verify Ed25519 signature (using ring or ed25519-dalek would go here)
    fn verify_ed25519_signature(&self, signature: &[u8], public_key: &[u8], _message: &[u8]) -> Result<bool> {
        // NOTE: This is a placeholder. In production, you would use:
        // - ed25519-dalek crate for verification
        // - ring crate for cryptographic operations
        //
        // Example with ed25519-dalek:
        // use ed25519_dalek::{PublicKey, Signature, Verifier};
        // let public_key = PublicKey::from_bytes(public_key)?;
        // let signature = Signature::from_bytes(signature)?;
        // public_key.verify(message, &signature).is_ok()

        tracing::warn!("Ed25519 signature verification is stubbed - implement with ed25519-dalek");

        // For now, basic checks
        if signature.len() != 64 {
            return Err(SignatureError::InvalidFormat.into());
        }

        if public_key.len() != 32 {
            return Err(SignatureError::InvalidFormat.into());
        }

        // Stub: return true if key is trusted
        Ok(self.is_key_trusted(public_key))
    }

    /// Check if a public key is trusted
    fn is_key_trusted(&self, public_key: &[u8]) -> bool {
        self.trusted_keys.iter().any(|key| key == public_key)
    }

    /// Calculate SHA256 hash of entire plugin directory
    async fn calculate_plugin_hash(&self, plugin_dir: &Path) -> Result<String> {
        let mut hasher = Sha256::new();

        // Collect all files in plugin directory (sorted for determinism)
        let mut file_paths = Vec::new();
        let mut entries = fs::read_dir(plugin_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                file_paths.push(path);
            }
        }

        file_paths.sort();

        // Hash each file's contents
        for file_path in file_paths {
            let contents = fs::read(&file_path).await?;
            hasher.update(&contents);

            // Also hash the filename for integrity
            if let Some(filename) = file_path.file_name() {
                hasher.update(filename.to_string_lossy().as_bytes());
            }
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Load default trusted public keys (Gausian team keys)
    fn load_default_trusted_keys() -> Vec<Vec<u8>> {
        // In production, these would be hardcoded Gausian team public keys
        // or loaded from a trusted keyring file
        vec![
            // Example placeholder key (32 bytes for Ed25519)
            // Using 0xFF so it's different from generated keys (which use 0x00)
            vec![0xFFu8; 32],
        ]
    }
}

/// Signature generator for plugin developers
pub struct SignatureGenerator {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
}

impl SignatureGenerator {
    /// Create new signature generator with key pair
    pub fn new(private_key: Vec<u8>, public_key: Vec<u8>) -> Result<Self> {
        if private_key.len() != 64 {
            return Err(SignatureError::InvalidFormat.into());
        }

        if public_key.len() != 32 {
            return Err(SignatureError::InvalidFormat.into());
        }

        Ok(Self {
            private_key,
            public_key,
        })
    }

    /// Generate a new Ed25519 key pair
    pub fn generate_keypair() -> Result<(Vec<u8>, Vec<u8>)> {
        // NOTE: This is a placeholder. In production, use ed25519-dalek:
        // use ed25519_dalek::Keypair;
        // use rand::rngs::OsRng;
        //
        // let mut csprng = OsRng{};
        // let keypair = Keypair::generate(&mut csprng);
        // Ok((keypair.secret.to_bytes().to_vec(), keypair.public.to_bytes().to_vec()))

        tracing::warn!("Key generation is stubbed - implement with ed25519-dalek");

        // Stub: return placeholder keys
        let private_key = vec![0u8; 64];
        let public_key = vec![0u8; 32];

        Ok((private_key, public_key))
    }

    /// Sign a plugin directory
    pub async fn sign_plugin(&self, plugin_dir: &Path, signer_name: &str) -> Result<PluginSignature> {
        // Calculate plugin hash
        let verifier = SignatureVerifier::new();
        let plugin_hash = verifier.calculate_plugin_hash(plugin_dir).await?;

        // Sign the hash with Ed25519
        let signature_bytes = self.sign_message(plugin_hash.as_bytes())?;

        Ok(PluginSignature {
            algorithm: "ed25519".to_string(),
            signature_bytes,
            public_key: self.public_key.clone(),
            signed_hash: plugin_hash,
            timestamp: chrono::Utc::now().timestamp(),
            signer: signer_name.to_string(),
        })
    }

    /// Sign a message with Ed25519
    fn sign_message(&self, _message: &[u8]) -> Result<Vec<u8>> {
        // NOTE: This is a placeholder. In production, use ed25519-dalek:
        // use ed25519_dalek::{Keypair, Signature, Signer};
        //
        // let keypair = Keypair::from_bytes(&self.private_key)?;
        // let signature = keypair.sign(message);
        // Ok(signature.to_bytes().to_vec())

        tracing::warn!("Message signing is stubbed - implement with ed25519-dalek");

        // Stub: return placeholder signature (64 bytes)
        Ok(vec![0u8; 64])
    }

    /// Save signature to file
    pub async fn save_signature(&self, signature: &PluginSignature, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(signature)?;
        fs::write(path, json).await?;
        Ok(())
    }

    /// Load signature from file
    pub async fn load_signature(path: &Path) -> Result<PluginSignature> {
        let json = fs::read_to_string(path).await?;
        let signature = serde_json::from_str(&json)?;
        Ok(signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_signature_generation() {
        let (private_key, public_key) = SignatureGenerator::generate_keypair().unwrap();
        assert_eq!(private_key.len(), 64);
        assert_eq!(public_key.len(), 32);
    }

    #[tokio::test]
    async fn test_plugin_signing_and_verification() {
        let temp_dir = tempdir().unwrap();
        let plugin_dir = temp_dir.path().join("test-plugin");
        fs::create_dir(&plugin_dir).await.unwrap();

        // Create a test file in plugin
        let test_file = plugin_dir.join("plugin.wasm");
        fs::write(&test_file, b"test plugin content").await.unwrap();

        // Generate key pair
        let (private_key, public_key) = SignatureGenerator::generate_keypair().unwrap();

        // Sign the plugin
        let generator = SignatureGenerator::new(private_key, public_key.clone()).unwrap();
        let signature = generator.sign_plugin(&plugin_dir, "test_author").await.unwrap();

        // Verify signature
        let mut verifier = SignatureVerifier::new();
        verifier.add_trusted_key(public_key);

        let is_valid = verifier.verify_plugin(&plugin_dir, &signature).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_untrusted_key_rejection() {
        let temp_dir = tempdir().unwrap();
        let plugin_dir = temp_dir.path().join("test-plugin");
        fs::create_dir(&plugin_dir).await.unwrap();

        let test_file = plugin_dir.join("plugin.wasm");
        fs::write(&test_file, b"test content").await.unwrap();

        // Generate key pair
        let (private_key, public_key) = SignatureGenerator::generate_keypair().unwrap();

        // Sign with one key
        let generator = SignatureGenerator::new(private_key, public_key).unwrap();
        let signature = generator.sign_plugin(&plugin_dir, "test_author").await.unwrap();

        // Verify with different trusted keys (should fail)
        let verifier = SignatureVerifier::new(); // Default keys only
        let is_valid = verifier.verify_plugin(&plugin_dir, &signature).await.unwrap();
        assert!(!is_valid); // Should be false because key is not trusted
    }

    #[tokio::test]
    async fn test_signature_persistence() {
        let temp_dir = tempdir().unwrap();
        let signature_path = temp_dir.path().join("signature.json");

        let (private_key, public_key) = SignatureGenerator::generate_keypair().unwrap();
        let generator = SignatureGenerator::new(private_key, public_key).unwrap();

        // Create plugin dir
        let plugin_dir = temp_dir.path().join("plugin");
        fs::create_dir(&plugin_dir).await.unwrap();
        fs::write(plugin_dir.join("test.wasm"), b"content").await.unwrap();

        // Generate and save signature
        let signature = generator.sign_plugin(&plugin_dir, "author").await.unwrap();
        generator.save_signature(&signature, &signature_path).await.unwrap();

        // Load signature
        let loaded_signature = SignatureGenerator::load_signature(&signature_path).await.unwrap();

        assert_eq!(signature.algorithm, loaded_signature.algorithm);
        assert_eq!(signature.signer, loaded_signature.signer);
        assert_eq!(signature.signed_hash, loaded_signature.signed_hash);
    }
}
