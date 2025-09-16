# Kenshō API Testing

## Using Postman

### Import Collections and Environment

1. Open Postman
2. Click **Import** button
3. Select the following files:
   - `kensho-api.postman_collection.json` - API endpoints collection
   - `kensho-local.postman_environment.json` - Environment variables

### Configure Environment

1. Click the environment dropdown (top right) and select "Kenshō Local Environment"
2. Click the eye icon to view/edit variables
3. Update these values:
   - `crunchyroll_email` - Your Crunchyroll email
   - `crunchyroll_password` - Your Crunchyroll password
   - `anime_id` - A valid UUID from your database (optional)

### Test Flow

1. **Health Check** - Verify server is running
2. **Login** - Authenticate with Crunchyroll (automatically saves token)
3. **Search Anime** - Search for anime by query
4. **Get Anime by ID** - Get detailed anime information
5. **Get Episodes** - List all episodes for an anime
6. **Browse Season** - Browse anime by season (e.g., 2024/spring)
7. **Get Stream URL** - Get streaming URL (requires authentication)
8. **Refresh Token** - Refresh expired token
9. **Logout** - End session

## Using curl

Quick test scripts are provided in `test-api.sh` for command-line testing.

## Features

- **Automatic Token Management**: Login endpoint automatically saves tokens to environment
- **Token Refresh**: Refresh endpoint updates tokens automatically
- **Pre-configured Variables**: Common test values are pre-set
- **Request Examples**: Each endpoint includes example data

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| base_url | API server URL | http://localhost:3000 |
| crunchyroll_email | Your Crunchyroll email | (update this) |
| crunchyroll_password | Your Crunchyroll password | (update this) |
| auth_token | JWT token (auto-set) | (empty) |
| refresh_token | Refresh token (auto-set) | (empty) |
| anime_id | Test anime UUID | 550e8400-e29b-41d4-a716-446655440000 |
| episode_number | Test episode number | 1 |
| search_query | Search term | spy family |
| year | Browse year | 2024 |
| season | Browse season | spring |

## Notes

- The server must be running (`cargo run --bin backend-server`)
- Redis must be running for authentication to work
- For POC, streaming URLs return mock data if Crunchyroll integration fails