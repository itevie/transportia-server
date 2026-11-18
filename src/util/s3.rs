use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::{Client, Config};
use aws_smithy_types::byte_stream::ByteStream;

pub async fn create_r2_client() -> Client {
    let account_id = std::env::var("R2_ACCOUNT_ID").unwrap();
    let access_key = std::env::var("R2_ACCESS_KEY_ID").unwrap();
    let secret_key = std::env::var("R2_SECRET_ACCESS_KEY").unwrap();

    let credentials = Credentials::new(access_key, secret_key, None, None, "r2");

    let config = Config::builder()
        .region(aws_sdk_s3::config::Region::new("auto"))
        .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", account_id))
        .credentials_provider(credentials)
        .build();

    Client::from_conf(config)
}

pub async fn upload_image(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    data: Vec<u8>,
    content_type: &str,
) -> Result<(), aws_sdk_s3::Error> {
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(data))
        .content_type(content_type)
        .send()
        .await?;

    Ok(())
}
