pub mod anime;
pub mod episode;
pub mod tag;
pub mod session;
pub mod relationships;
pub mod anime_offline_db;
pub mod dead_entries_db;

#[cfg(test)]
mod tests;

pub use anime::{Anime, AnimeStatus, AnimeType, AnimeSeason, Season, ImdbData, AnimeSummary, AnimeDetail, RelatedAnime};
pub use episode::{Episode, EpisodeResponse, EpisodeListResponse};
pub use tag::{Tag, TagCategory, TagResponse};
pub use session::{Session, SessionCreate, SessionResponse, Claims};
pub use relationships::{HasTag, IsSequelOf, IsPrequelOf, RelatedTo, RelationType, BelongsTo, RelationshipQueries};
pub use anime_offline_db::{AnimeOfflineDatabase, AnimeOfflineEntry, OfflineAnimeType, OfflineAnimeStatus, DatabaseStats};
pub use dead_entries_db::{DeadEntriesDatabase, DeadEntriesStats, HasMalId};
