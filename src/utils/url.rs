use crate::utils::regex::*;
use crate::config::Config;

// Check if URL is a supported GitHub URL
pub fn is_github_url(path: &str) -> bool {
    GITHUB_RELEASES.is_match(path)
        || GITHUB_GIST.is_match(path)
        || GITHUB_TAGS.is_match(path)
        || GITHUB_GIT_INFO.is_match(path)
        || GITHUB_RAW.is_match(path)
        || GITHUB_BLOB_RAW.is_match(path)
}

// Check if URL is a supported GitLab URL
pub fn is_gitlab_url(path: &str, config: &Config) -> bool {
    config.git_services.gitlab_enabled && (
        GITLAB_PROJECTS.is_match(path)
        || GITLAB_RAW.is_match(path)
        || GITLAB_BLOBS.is_match(path)
    )
}

// Check if URL is a supported Bitbucket URL
pub fn is_bitbucket_url(path: &str, config: &Config) -> bool {
    config.git_services.bitbucket_enabled && (
        BITBUCKET_REPO.is_match(path)
        || BITBUCKET_RAW.is_match(path)
    )
}

// Check if URL is any supported Git service URL
pub fn is_supported_url(path: &str, config: &Config) -> bool {
    is_github_url(path) || is_gitlab_url(path, config) || is_bitbucket_url(path, config)
}