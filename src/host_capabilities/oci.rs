use anyhow::{anyhow, Result};
use oci_spec::image::{ImageConfiguration, ImageIndex, ImageManifest};
use serde::{Deserialize, Serialize};
use serde_json::json;
#[cfg(test)]
use tests::mock_wapc as wapc_guest;

/// Response to manifest digest request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestDigestResponse {
    pub digest: String,
}

/// An image, or image index, OCI manifest
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum OciManifestResponse {
    //Using  box here to make linter happy. It complains about the different sizes between the two
    //enum elements. See more here:
    //https://rust-lang.github.io/rust-clippy/master/index.html#/large_enum_variant
    /// An OCI image manifest
    Image(Box<ImageManifest>),
    /// An OCI image index manifest
    ImageIndex(Box<ImageIndex>),
}

/// Response to manifest and config request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OciManifestAndConfigResponse {
    pub manifest: ImageManifest,
    pub digest: String,
    pub config: ImageConfiguration,
}

/// Computes the digest of the OCI object referenced by `image`
pub fn get_manifest_digest(image: &str) -> Result<ManifestDigestResponse> {
    let req = json!(image);
    let msg = serde_json::to_vec(&req)
        .map_err(|e| anyhow!("error serializing the validation request: {}", e))?;
    let response_raw = wapc_guest::host_call("kubewarden", "oci", "v1/manifest_digest", &msg)
        .map_err(|e| anyhow!("error invoking wapc oci.manifest_digest: {:?}", e))?;

    let response: ManifestDigestResponse = serde_json::from_slice(&response_raw)?;

    Ok(response)
}

/// Fetches OCI manifest referenced by `image`
pub fn get_manifest(image: &str) -> Result<OciManifestResponse> {
    let req = json!(image);
    let msg = serde_json::to_vec(&req)
        .map_err(|e| anyhow!("error serializing the validation request: {}", e))?;
    let response_raw = wapc_guest::host_call("kubewarden", "oci", "v1/oci_manifest", &msg)
        .map_err(|e| anyhow!("error invoking wapc oci.manifest_digest: {:?}", e))?;
    let response: OciManifestResponse = serde_json::from_slice(&response_raw)?;
    Ok(response)
}

/// Fetches OCI image manifest and configuration referenced by `image`
pub fn get_manifest_and_config(image: &str) -> Result<OciManifestAndConfigResponse> {
    let req = json!(image);
    let msg = serde_json::to_vec(&req)
        .map_err(|e| anyhow!("error serializing the validation request: {}", e))?;
    let response_raw =
        wapc_guest::host_call("kubewarden", "oci", "v1/oci_manifest_config", &msg)
            .map_err(|e| anyhow!("error invoking wapc oci.manifest_and_config: {:?}", e))?;

    let response: OciManifestAndConfigResponse = serde_json::from_slice(&response_raw)?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::automock;
    use oci_spec::image::{
        Arch, ConfigBuilder, Descriptor, DescriptorBuilder, Digest, History, HistoryBuilder,
        ImageConfigurationBuilder, ImageIndexBuilder, ImageManifestBuilder, MediaType, Os,
        PlatformBuilder, RootFsBuilder, SCHEMA_VERSION,
    };
    use serial_test::serial;
    use std::str::FromStr;

    #[automock()]
    pub mod wapc {
        use wapc_guest::CallResult;

        // needed for creating mocks
        #[allow(dead_code)]
        pub fn host_call(_binding: &str, _ns: &str, _op: &str, _msg: &[u8]) -> CallResult {
            Ok(vec![u8::from(true)])
        }
    }
    fn create_oci_index_image_manifest() -> ImageIndex {
        let manifests: Vec<Descriptor> = [
            (
                32654,
                "sha256:9834876dcfb05cb167a5c24953eba58c4ac89b1adf57f28f2f9d09af107ee8f0",
                "amd64",
                "linux",
            ),
            (
                16724,
                "sha256:3c3a4604a545cdc127456d94e421cd355bca5b528f4a9c1905b15da2eb4a4c6b",
                "arm64",
                "linux",
            ),
            (
                73109,
                "sha256:ec4b8955958665577945c89419d1af06b5f7636b4ac3da7f12184802ad867736",
                "unknown",
                "unknown",
            ),
        ]
        .iter()
        .map(|l| {
            let plataform_builder = PlatformBuilder::default();
            let arch = match l.2 {
                "amd64" => Arch::Amd64,
                "arm64" => Arch::ARM64,
                "unknown" => Arch::Other("unknown".to_owned()),
                _ => Arch::Other("other".to_owned()),
            };
            let os = match l.3 {
                "linux" => Os::Linux,
                "unknown" => Os::Other("unknown".to_owned()),
                _ => Os::Other("other".to_owned()),
            };
            let platform = plataform_builder
                .architecture(arch)
                .os(os)
                .build()
                .expect("build platform");
            let digest = Digest::from_str(l.1).expect("parse digest");
            let size: u64 = u64::try_from(l.0).expect("parse size");
            DescriptorBuilder::default()
                .media_type(MediaType::ImageLayerGzip)
                .size(size)
                .platform(platform)
                .digest(digest)
                .build()
                .expect("build layer")
        })
        .collect();
        ImageIndexBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .media_type(MediaType::ImageIndex)
            .manifests(manifests)
            .build()
            .expect("build image manifest")
    }

    fn create_oci_image_manifest() -> ImageManifest {
        let digest = Digest::from_str(
            "sha256:b5b2b2c507a0944348e0303114d8d93aaaa081732b86451d9bce1f432a537bc7",
        )
        .expect("parse digest");
        let config = DescriptorBuilder::default()
            .media_type(MediaType::ImageConfig)
            .size(7023u64)
            .digest(digest)
            .build()
            .expect("build config descriptor");

        let layers: Vec<Descriptor> = [
            (
                32654,
                "sha256:9834876dcfb05cb167a5c24953eba58c4ac89b1adf57f28f2f9d09af107ee8f0",
            ),
            (
                16724,
                "sha256:3c3a4604a545cdc127456d94e421cd355bca5b528f4a9c1905b15da2eb4a4c6b",
            ),
            (
                73109,
                "sha256:ec4b8955958665577945c89419d1af06b5f7636b4ac3da7f12184802ad867736",
            ),
        ]
        .iter()
        .map(|l| {
            let digest = Digest::from_str(l.1).expect("parse digest");
            let size = u64::try_from(l.0).expect("parse size");
            DescriptorBuilder::default()
                .media_type(MediaType::ImageLayerGzip)
                .size(size)
                .digest(digest)
                .build()
                .expect("build manifest")
        })
        .collect();

        ImageManifestBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .media_type(MediaType::ImageManifest)
            .config(config)
            .layers(layers)
            .build()
            .expect("build image manifest")
    }

    fn create_oci_image_configuration() -> ImageConfiguration {
        let config = ConfigBuilder::default()
            .user("65533:65533".to_string())
            .exposed_ports(vec!["3000/tcp".to_string()])
            .env(vec![
                "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
            ])
            .entrypoint(vec!["/policy-server".to_string()])
            .working_dir("/".to_string())
            .build()
            .expect("build config");

        let history: Vec<History> = [
            (
                "2024-06-17T17:55:55.014762797Z",
                "COPY /etc/passwd /etc/passwd # buildkit",
                "buildkit.dockerfile.v0",
            ),
            (
                "2024-06-17T17:55:55.101579791Z",
                "COPY /etc/group /etc/group # buildkit",
                "buildkit.dockerfile.v0",
            ),
            (
                "2024-06-17T17:55:55.616407556Z",
                "COPY --chmod=0755 policy-server-x86_64 /policy-server # buildkit",
                "buildkit.dockerfile.v0",
            ),
            (
                "2024-06-17T17:55:55.630019968Z",
                "ADD Cargo.lock /Cargo.lock # buildkit",
                "buildkit.dockerfile.v0",
            ),
        ]
        .iter()
        .map(|l| {
            HistoryBuilder::default()
                .created(l.0.to_string())
                .created_by(l.1.to_string())
                .comment(l.2.to_string())
                .build()
                .expect("build history")
        })
        .collect();
        let rootfs = RootFsBuilder::default()
            .diff_ids(vec![
                "sha256:d721137c9798b29b57611789af80d5fa864be33288150fdd8c35f88cf24998be"
                    .to_string(),
                "sha256:a1be1b83b5a388de0ac5baacd2afaf632c8ae61cffbbf4306d5e9be92b5b46b9"
                    .to_string(),
                "sha256:8be913709f915800a634431f239c34e330a9f9a09ea944d38ead4d5f201d22e7"
                    .to_string(),
                "sha256:69bb22b8d1508715424f4d89f415843acceba84087277078a30dda65c47796d7"
                    .to_string(),
            ])
            .typ("layers".to_string())
            .build()
            .expect("build rootfs");
        ImageConfigurationBuilder::default()
            .architecture(Arch::Amd64)
            .config(config)
            .history(history)
            .created("2024-06-17T17:55:55.630019968Z".to_string())
            .os(Os::Linux)
            .rootfs(rootfs)
            .build()
            .expect("build image configuration")
    }

    // these tests need to run sequentially because mockall creates a global context to create the mocks
    #[serial]
    #[test]
    fn verify_oci_image_manifest() {
        let ctx = mock_wapc::host_call_context();
        ctx.expect()
            .once()
            .withf(|binding: &str, ns: &str, op: &str, msg: &[u8]| {
                binding == "kubewarden"
                    && ns == "oci"
                    && op == "v1/oci_manifest"
                    && std::str::from_utf8(msg).unwrap()
                        == "\"ghcr.io/kubewarden/policy-server:latest\""
            })
            .returning(|_, _, _, _| Ok(serde_json::to_vec(&create_oci_image_manifest()).unwrap()));
        let response = get_manifest("ghcr.io/kubewarden/policy-server:latest")
            .expect("failed to get oci manifest reponse");
        match response {
            OciManifestResponse::Image(image) => {
                assert_eq!(*image, create_oci_image_manifest());
            }
            OciManifestResponse::ImageIndex(_) => panic!("Invalid oci manifest type returned"),
        }
    }

    // these tests need to run sequentially because mockall creates a global context to create the mocks
    #[serial]
    #[test]
    fn verify_oci_index_image_manifest() {
        let ctx = mock_wapc::host_call_context();
        ctx.expect()
            .once()
            .withf(|binding: &str, ns: &str, op: &str, msg: &[u8]| {
                binding == "kubewarden"
                    && ns == "oci"
                    && op == "v1/oci_manifest"
                    && std::str::from_utf8(msg).unwrap()
                        == "\"ghcr.io/kubewarden/policy-server:latest\""
            })
            .returning(|_, _, _, _| {
                Ok(serde_json::to_vec(&create_oci_index_image_manifest()).unwrap())
            });
        let response = get_manifest("ghcr.io/kubewarden/policy-server:latest")
            .expect("failed to get oci manifest reponse");
        match response {
            OciManifestResponse::Image(_) => panic!("Invalid oci manifest type returned"),
            OciManifestResponse::ImageIndex(image) => {
                assert_eq!(*image, create_oci_index_image_manifest());
            }
        }
    }

    // these tests need to run sequentially because mockall creates a global context to create the mocks
    #[serial]
    #[test]
    fn verify_oci_image_manifest_and_config() {
        let ctx = mock_wapc::host_call_context();
        ctx.expect()
            .once()
            .withf(|binding: &str, ns: &str, op: &str, msg: &[u8]| {
                binding == "kubewarden"
                    && ns == "oci"
                    && op == "v1/oci_manifest_config"
                    && std::str::from_utf8(msg).unwrap()
                        == "\"ghcr.io/kubewarden/policy-server:latest\""
            })
            .returning(|_, _, _, _| {
                let response_raw = serde_json::to_vec(&OciManifestAndConfigResponse {
                    manifest: create_oci_image_manifest(),
                    digest: "sha256:983".to_owned(),
                    config: create_oci_image_configuration(),
                })
                .expect("serialize response");
                Ok(response_raw)
            });
        let response = get_manifest_and_config("ghcr.io/kubewarden/policy-server:latest")
            .expect("failed to get oci manifest reponse");
        assert_eq!(response.config, create_oci_image_configuration());
        assert_eq!(response.manifest, create_oci_image_manifest());
        assert_eq!(response.digest, "sha256:983");
    }
}
