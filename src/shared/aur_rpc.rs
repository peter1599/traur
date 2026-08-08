use crate::shared::models::AurPackage;
use serde::Deserialize;

const AUR_RPC_BASE: &str = "https://aur.archlinux.org/rpc/v5";

#[derive(Deserialize)]
struct RpcResponse {
    results: Vec<AurPackage>,
}

fn info_url(names: &[&str]) -> Result<reqwest::Url, String> {
    let mut url = reqwest::Url::parse(&format!("{AUR_RPC_BASE}/info"))
        .map_err(|e| format!("Failed to build AUR URL: {e}"))?;
    {
        let mut query = url.query_pairs_mut();
        for name in names {
            query.append_pair("arg[]", name);
        }
    }
    Ok(url)
}

fn maintainer_url(maintainer: &str) -> Result<reqwest::Url, String> {
    let mut url = reqwest::Url::parse(&format!("{AUR_RPC_BASE}/search"))
        .map_err(|e| format!("Failed to build AUR URL: {e}"))?;
    url.path_segments_mut()
        .map_err(|_| "Failed to build AUR maintainer URL".to_string())?
        .push(maintainer);
    url.query_pairs_mut().append_pair("by", "maintainer");
    Ok(url)
}

fn get_json(url: reqwest::Url) -> Result<RpcResponse, String> {
    reqwest::blocking::get(url)
        .map_err(|e| format!("HTTP request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("AUR request returned an error: {e}"))?
        .json()
        .map_err(|e| format!("Failed to parse AUR response: {e}"))
}

/// Look up one package while preserving the difference between not found and request failure.
pub fn find_package_info(package_name: &str) -> Result<Option<AurPackage>, String> {
    let resp = get_json(info_url(&[package_name])?)?;
    Ok(resp.results.into_iter().next())
}

/// Fetch info for multiple packages in a single request.
pub fn fetch_packages_info(names: &[&str]) -> Result<Vec<AurPackage>, String> {
    Ok(get_json(info_url(names)?)?.results)
}

/// Fetch all packages maintained by a given user.
pub fn fetch_maintainer_packages(maintainer: &str) -> Result<Vec<AurPackage>, String> {
    Ok(get_json(maintainer_url(maintainer)?)?.results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_url_encodes_package_names() {
        let names = ["notepad++", "name with spaces"];
        let url = info_url(names.as_slice()).unwrap();

        assert_eq!(
            url.query_pairs().collect::<Vec<_>>(),
            vec![
                ("arg[]".into(), "notepad++".into()),
                ("arg[]".into(), "name with spaces".into()),
            ]
        );
        assert!(url.as_str().contains("arg%5B%5D=notepad%2B%2B"));
    }

    #[test]
    fn maintainer_url_encodes_path_segment() {
        let url = maintainer_url("user/name").unwrap();

        assert!(url.as_str().contains("/search/user%2Fname?by=maintainer"));
    }
}
