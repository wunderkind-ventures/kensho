# Kenshō Frontend - Viewing Guide

## Prerequisites

1. **Install Trunk** (WASM bundler for Rust):
```bash
cargo install trunk
```

2. **Add WASM target** to Rust:
```bash
rustup target add wasm32-unknown-unknown
```

## Running the Frontend

### Option 1: Development Server (Recommended)
```bash
cd frontend
trunk serve
```
This will:
- Build the WASM application
- Start a dev server at `http://localhost:8080`
- Auto-reload on file changes

### Option 2: Build for Production
```bash
cd frontend
trunk build --release
```
This creates optimized files in `frontend/dist/`

## Accessing the Application

Once running, open your browser to:
- **Development**: http://localhost:8080
- **With Backend**: Ensure backend is running on http://localhost:3000

## Current Frontend Features

The frontend currently includes:
- Home page with anime grid
- Search functionality  
- Login page
- Anime details page
- Video player component
- Season browser

## Troubleshooting

### Backend Connection Issues
If the frontend can't connect to the backend:

1. **Start the backend first**:
```bash
cd backend
cargo run --bin backend-server
```

2. **Check CORS settings** - The backend should allow `http://localhost:8080`

3. **Verify API endpoint** in `.env`:
```
API_URL=http://localhost:3000/api
```

### Build Errors
If you encounter build errors:

1. **Clean and rebuild**:
```bash
trunk clean
trunk serve
```

2. **Check Rust version**:
```bash
rustc --version  # Should be 1.75+
```

3. **Update dependencies**:
```bash
cargo update
```

## Quick Start Commands

```bash
# Install dependencies and run
cargo install trunk
rustup target add wasm32-unknown-unknown
cd frontend
trunk serve

# Open browser to http://localhost:8080
```

## Development Tips

- Use browser DevTools to inspect network requests
- Check console for WASM errors
- The frontend uses Dioxus, a React-like framework for Rust
- Components are in `src/components/`
- Pages are in `src/pages/`

## Current Status

⚠️ **Note**: The frontend is a POC implementation. Some features require:
- Backend running with Redis
- Valid Crunchyroll credentials for streaming
- Sample anime data loaded in the database

For the best experience, ensure the backend is fully operational with all dependencies.