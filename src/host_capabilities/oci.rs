use anyhow::{anyhow, Result};
use oci_spec::image::{ImageIndex, ImageManifest};
use serde::{Deserialize, Serialize};
use serde_json::json;
#[cfg(test)]
use tests::mock_wapc as wapc_guest;

/// Response to manifest digest request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestDigestResponse {
    /// list of Ips that have been resolved
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

/// Computes the digest of the OCI object referenced by `image`
pub fn manifest_digest(image: &str) -> Result<ManifestDigestResponse> {
    let req = json!(image);
    let msg = serde_json::to_vec(&req)
        .map_err(|e| anyhow!("error serializing the validation request: {}", e))?;
    let response_raw = wapc_guest::host_call("kubewarden", "oci", "v1/manifest_digest", &msg)
        .map_err(|e| anyhow!("error invoking wapc oci.manifest_digest: {:?}", e))?;

    let response: ManifestDigestResponse = serde_json::from_slice(&response_raw)?;

    Ok(response)
}

/// Fetches OCI manifest referenced by `image`
pub fn manifest(image: &str) -> Result<OciManifestResponse> {
    let req = json!(image);
    let msg = serde_json::to_vec(&req)
        .map_err(|e| anyhow!("error serializing the validation request: {}", e))?;
    let response_raw = wapc_guest::host_call("kubewarden", "oci", "v1/oci_manifest", &msg)
        .map_err(|e| anyhow!("error invoking wapc oci.manifest_digest: {:?}", e))?;

    let response: OciManifestResponse = serde_json::from_slice(&response_raw)?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::automock;
    use oci_spec::image::{
        Arch, Descriptor, DescriptorBuilder, ImageIndexBuilder, ImageManifestBuilder, MediaType,
        Os, PlatformBuilder, SCHEMA_VERSION,
    };
    use serial_test::serial;

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
            DescriptorBuilder::default()
                .media_type(MediaType::ImageLayerGzip)
                .size(l.0)
                .platform(platform)
                .digest(l.1.to_owned())
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
        let config = DescriptorBuilder::default()
            .media_type(MediaType::ImageConfig)
            .size(7023)
            .digest("sha256:b5b2b2c507a0944348e0303114d8d93aaaa081732b86451d9bce1f432a537bc7")
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
            return DescriptorBuilder::default()
                .media_type(MediaType::ImageLayerGzip)
                .size(l.0)
                .digest(l.1.to_owned())
                .build()
                .expect("build manifest");
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
        let response = manifest("ghcr.io/kubewarden/policy-server:latest")
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
        let response = manifest("ghcr.io/kubewarden/policy-server:latest")
            .expect("failed to get oci manifest reponse");
        match response {
            OciManifestResponse::Image(_) => panic!("Invalid oci manifest type returned"),
            OciManifestResponse::ImageIndex(image) => {
                assert_eq!(*image, create_oci_index_image_manifest());
            }
        }
    }
}
