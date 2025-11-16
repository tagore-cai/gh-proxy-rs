use once_cell::sync::Lazy;
use regex::Regex;

// GitHub patterns
pub static GITHUB_RELEASES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:https?:\/\/)?github\.com\/.+?\/.+?\/(?:releases|archive)\/.*$").expect("Invalid regex GITHUB_RELEASES")
});

pub static GITHUB_BLOB_RAW: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:https?:\/\/)?github\.com\/.+?\/.+?\/(?:blob|raw)\/.*$").expect("Invalid regex GITHUB_BLOB_RAW"));

pub static GITHUB_GIT_INFO: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:https?:\/\/)?github\.com\/.+?\/.+?\/(?:info|git-).*").expect("Invalid regex GITHUB_GIT_INFO"));

pub static GITHUB_RAW: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:https?:\/\/)?raw\.(?:githubusercontent|github)\.com\/.+?\/.+?\/.+?\/.+$")
        .expect("Invalid regex GITHUB_RAW")
});

pub static GITHUB_GIST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:https?:\/\/)?gist\.(?:githubusercontent|github)\.com\/.+?\/.+?\/.+$").expect("Invalid regex GITHUB_GIST")
});

pub static GITHUB_TAGS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:https?:\/\/)?github\.com\/.+?\/.+?\/tags.*$").expect("Invalid regex GITHUB_TAGS"));

// GitLab patterns
pub static GITLAB_PROJECTS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:https?:\/\/)?gitlab\.com\/.+?\/.+?\/(?:-/|repository/archive\.tar\.gz).*").expect("Invalid regex GITLAB_PROJECTS")
});

pub static GITLAB_RAW: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:https?:\/\/)?gitlab\.com\/.+?\/.+?\/(?:-/)?raw\/.*$").expect("Invalid regex GITLAB_RAW")
});

pub static GITLAB_BLOBS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:https?:\/\/)?gitlab\.com\/.+?\/.+?\/(?:-/)?blob\/.*$").expect("Invalid regex GITLAB_BLOBS")
});

// Bitbucket patterns
pub static BITBUCKET_REPO: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:https?:\/\/)?bitbucket\.org\/.+?\/.+?\/(?:get|downloads).*").expect("Invalid regex BITBUCKET_REPO")
});

pub static BITBUCKET_RAW: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:https?:\/\/)?bitbucket\.org\/.+?\/.+?\/(?:raw|src)\/.*$").expect("Invalid regex BITBUCKET_RAW")
});