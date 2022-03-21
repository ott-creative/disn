use crate::error::{Error, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client;
use aws_sdk_s3::Endpoint;
use poem::http::Uri;

pub struct AwsService;

impl AwsService {
    pub async fn list_bucket() -> Result<String> {
        let ep = Endpoint::immutable(Uri::from_static("https://sgp1.digitaloceanspaces.com"));
        let conf = aws_config::load_from_env().await;
        let s3_conf = aws_sdk_s3::config::Builder::from(&conf)
            .endpoint_resolver(ep)
            .build();
        let s3 = Client::from_conf(s3_conf);
        let buckets = s3.list_buckets().send().await;
        println!("got buckets: {:#?}", buckets);

        Ok(" ".to_string())
    }
}
