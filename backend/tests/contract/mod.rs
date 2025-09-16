// Contract tests module
// These tests verify API contracts defined in specs/001-project-kensh-poc/contracts/openapi.yaml

mod common;

// Contract test modules
mod test_anime_get;
mod test_search;
mod test_browse_season;
mod test_episodes_get;
mod test_auth_login;
mod test_auth_logout;
mod test_auth_refresh;
mod test_stream;