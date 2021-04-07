use {
    s3::{bucket::Bucket, creds::Credentials, Region},
    std::{env, path::PathBuf},
};

#[derive(Debug)]
pub struct Conf {
    /// Where should the main database be stored.
    pub sqlite_file: PathBuf,
    /// The handle to the S3 bucket where we upload pages.
    pub bucket: Bucket,
    /// How many new stories should we fetch from the HNs APIs.
    pub new_stories_limit: usize,
    /// How many stories can a page display at most.
    pub stories_per_page: usize,
    /// If set to true, we won't upload the html to S3 but instead store it into
    /// "pages" directory.
    pub store_html_locally: bool,
}

impl Conf {
    /// Creates config from env vars.
    ///
    /// # Panic
    /// If any mandatory var is missing or malformed.
    pub fn new() -> Self {
        let sqlite_file = env::var(vars::SQLITE_FILE)
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                panic!("Missing env var {}.", vars::SQLITE_FILE)
            });
        log::debug!("{}={:?}", vars::SQLITE_FILE, sqlite_file);

        // swallows parsing errors but it's ok for our use case
        let new_stories_limit = env::var(vars::NEW_STORIES_LIMIT)
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(defaults::NEW_STORIES_LIMIT);
        log::debug!("{}={:?}", vars::NEW_STORIES_LIMIT, new_stories_limit);

        let stories_per_page = env::var(vars::STORIES_PER_PAGE)
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(defaults::STORIES_PER_PAGE);
        log::debug!("{}={:?}", vars::STORIES_PER_PAGE, stories_per_page);

        let store_html_locally = env::var(vars::STORE_HTML_LOCALLY)
            .map(|s| match s.trim() {
                "ok" | "yes" | "1" | "true" => true,
                _ => false,
            })
            .unwrap_or(false);
        log::debug!("{}={:?}", vars::STORE_HTML_LOCALLY, store_html_locally);

        let content_cache_header = env::var(vars::CONTENT_CACHE_HEADER)
            .ok()
            .unwrap_or_else(|| defaults::CONTENT_CACHE_HEADER.to_string());
        log::debug!(
            "{}={:?}",
            vars::CONTENT_CACHE_HEADER,
            content_cache_header
        );

        let bucket_name = env::var(vars::BUCKET_NAME).unwrap_or_else(|_| {
            panic!("Missing env var {}.", vars::BUCKET_NAME)
        });
        log::debug!("{}={:?}", vars::BUCKET_NAME, bucket_name);

        let bucket_region: Region = env::var(vars::BUCKET_REGION)
            .ok()
            .and_then(|region| region.parse().ok())
            .unwrap_or_else(|| {
                panic!("Missing or invalid env var {}.", vars::BUCKET_REGION)
            });
        log::debug!("{}={:?}", vars::BUCKET_REGION, bucket_region);

        // default creds are read from env
        // AWS_ACCESS_KEY_ID
        // AWS_SECRET_ACCESS_KEY
        let creds = Credentials::default().expect("Missing AWS creds");

        let mut bucket = Bucket::new(&bucket_name, bucket_region, creds)
            .expect("Cannot create bucket handle");
        bucket.add_header("Cache-Control", &content_cache_header);

        Self {
            bucket,
            new_stories_limit,
            sqlite_file,
            store_html_locally,
            stories_per_page,
        }
    }
}

mod vars {
    pub const SQLITE_FILE: &str = "SQLITE_FILE";
    pub const BUCKET_NAME: &str = "BUCKET_NAME";
    pub const BUCKET_REGION: &str = "BUCKET_REGION";
    pub const STORE_HTML_LOCALLY: &str = "STORE_HTML_LOCALLY"; // opt
    pub const NEW_STORIES_LIMIT: &str = "NEW_STORIES_LIMIT"; // opt
    pub const STORIES_PER_PAGE: &str = "STORIES_PER_PAGE"; // opt
    pub const CONTENT_CACHE_HEADER: &str = "CONTENT_CACHE_HEADER"; // opt
}

mod defaults {
    pub const NEW_STORIES_LIMIT: usize = 50;
    pub const STORIES_PER_PAGE: usize = 30;
    pub const CONTENT_CACHE_HEADER: &str = "public, max-age: 300";
}
