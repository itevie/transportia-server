
use std::time::Duration;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RateLimitBucket {
    General,
    Authentication,

    Create,
    Update,
    Delete,
    Query,

    Search,

    Upload,
    Media,

    Favourite,
    Rating,
    Comment,
    Checklist,

    Import,
    ExternalData,

    Metadata,
}

pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window: Duration,
}

impl RateLimitBucket {
    pub fn config(&self) -> RateLimitConfig {
        match self {
            Self::General => RateLimitConfig {
                max_requests: 120,
                window: Duration::from_secs(60),
            },
            Self::Authentication => RateLimitConfig {
                max_requests: 5,
                window: Duration::from_secs(60),
            },

            Self::Create => RateLimitConfig {
                max_requests: 30,
                window: Duration::from_secs(60),
            },
            Self::Update => RateLimitConfig {
                max_requests: 60,
                window: Duration::from_secs(60),
            },
            Self::Delete => RateLimitConfig {
                max_requests: 20,
                window: Duration::from_secs(60),
            },
            Self::Query => RateLimitConfig {
                max_requests: 120,
                window: Duration::from_secs(60),
            },

            Self::Search => RateLimitConfig {
                max_requests: 60,
                window: Duration::from_secs(60),
            },

            Self::Upload => RateLimitConfig {
                max_requests: 60,
                window: Duration::from_secs(3600),
            },
            Self::Media => RateLimitConfig {
                max_requests: 120,
                window: Duration::from_secs(60),
            },

            Self::Favourite => RateLimitConfig {
                max_requests: 60,
                window: Duration::from_secs(60),
            },
            Self::Rating => RateLimitConfig {
                max_requests: 30,
                window: Duration::from_secs(60),
            },
            Self::Comment => RateLimitConfig {
                max_requests: 20,
                window: Duration::from_secs(600),
            },
            Self::Checklist => RateLimitConfig {
                max_requests: 60,
                window: Duration::from_secs(60),
            },

            Self::Import => RateLimitConfig {
                max_requests: 5,
                window: Duration::from_secs(3600),
            },
            Self::ExternalData => RateLimitConfig {
                max_requests: 30,
                window: Duration::from_secs(60),
            },

            Self::Metadata => RateLimitConfig {
                max_requests: 60,
                window: Duration::from_secs(60),
            },
        }
    }
}

pub trait RateLimitPolicy: Send + Sync + 'static {
    const BUCKET: RateLimitBucket;
}

pub struct GeneralRatelimit;
pub struct AuthenticationRatelimit;

pub struct CreateRatelimit;
pub struct UpdateRatelimit;
pub struct DeleteRatelimit;
pub struct QueryRatelimit;

pub struct SearchRatelimit;

pub struct UploadRatelimit;
pub struct MediaRatelimit;

pub struct FavouriteRatelimit;
pub struct RatingRatelimit;
pub struct CommentRatelimit;
pub struct ChecklistRatelimit;

pub struct ImportRatelimit;
pub struct ExternalDataRatelimit;

pub struct MetadataRatelimit;

macro_rules! rate_limit_policies {
    ($($marker:ty => $bucket:ident),* $(,)?) => {
        $(
            impl RateLimitPolicy for $marker {
                const BUCKET: RateLimitBucket = RateLimitBucket::$bucket;
            }
        )*
    };
}

rate_limit_policies! {
    GeneralRatelimit => General,
    AuthenticationRatelimit => Authentication,

    CreateRatelimit => Create,
    UpdateRatelimit => Update,
    DeleteRatelimit => Delete,
    QueryRatelimit => Query,

    SearchRatelimit => Search,

    UploadRatelimit => Upload,
    MediaRatelimit => Media,

    FavouriteRatelimit => Favourite,
    RatingRatelimit => Rating,
    CommentRatelimit => Comment,
    ChecklistRatelimit => Checklist,

    ImportRatelimit => Import,
    ExternalDataRatelimit => ExternalData,

    MetadataRatelimit => Metadata,
}
