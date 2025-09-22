# MAL Data Import Scripts

This directory contains scripts to import MyAnimeList (MAL) anime data into SurrealDB with full relationship modeling.

## 📋 Prerequisites

1. **SurrealDB running**: Make sure SurrealDB is running (usually via Docker)
2. **Python 3**: Required for the import script
3. **Data files**: The following JSON files should be in the `../data/` directory:
   - `anime-offline-database.json` - Main anime database
   - `myanimelist.json` - Dead entries database for filtering

## 🚀 Quick Start

The easiest way to get started is with the shell script wrapper:

```bash
# Test import (100 entries)
./import_mal_data.sh test

# Small import (1000 entries with relationships)
./import_mal_data.sh small

# Medium import (5000 entries with relationships)
./import_mal_data.sh medium
```

## 📁 Files

### `import_mal_data.py`
The main Python script that handles the data import process. Features:
- **Dead entry filtering**: Uses `myanimelist.json` to filter out invalid entries
- **Batch processing**: Imports data in batches for better performance
- **Relationship creation**: Creates `has_tag` and `is_sequel` relationships
- **Episode generation**: Creates episode records based on anime metadata
- **Progress tracking**: Shows detailed progress and statistics
- **Error handling**: Robust error handling with detailed logging

### `import_mal_data.sh`  
Convenient shell script wrapper with predefined configurations:
- **Pre-flight checks**: Verifies SurrealDB connection and data files
- **Dependency management**: Checks and installs Python dependencies
- **Multiple import modes**: Test, small, medium, large, and full imports
- **Flexible options**: Supports custom limits and skip flags

## 🎯 Import Modes

| Mode | Entries | Relationships | Episodes | Use Case |
|------|---------|---------------|----------|----------|
| `test` | 100 | ✅ | ✅ | Quick testing |
| `small` | 1,000 | ✅ | ✅ | Development |
| `medium` | 5,000 | ✅ | ✅ | Demo/staging |
| `large` | 10,000 | ✅ | ✅ | Production subset |
| `full` | ~39,000 | ✅ | ✅ | Full dataset |
| `anime-only` | 1,000 | ❌ | ❌ | Anime entities only |

## 🔧 Usage Examples

### Using the Shell Script (Recommended)

```bash
# Basic usage
./import_mal_data.sh test                    # Import 100 entries
./import_mal_data.sh small                   # Import 1000 entries
./import_mal_data.sh medium                  # Import 5000 entries

# Custom options
./import_mal_data.sh --limit 500 anime-only  # 500 anime without relationships
./import_mal_data.sh --skip-episodes small   # Skip episode creation
./import_mal_data.sh --db-url http://localhost:8001 test  # Custom DB URL

# Help
./import_mal_data.sh help
```

### Using the Python Script Directly

```bash
# Basic import
python3 import_mal_data.py --limit 100

# Skip relationships and episodes
python3 import_mal_data.py --limit 1000 --skip-relationships --skip-episodes

# Custom SurrealDB connection
python3 import_mal_data.py --db-url http://localhost:8001 --username admin --password secret

# Full options
python3 import_mal_data.py \
  --limit 5000 \
  --db-url http://localhost:8000 \
  --username root \
  --password root \
  --data-dir ../data
```

## 📊 What Gets Imported

### Anime Entities
- **Title, synopsis, type, status, episodes**
- **Season information** (year, season)
- **Poster URLs and source links**
- **Alternative titles/synonyms**

### Tag Entities  
- **Genre tags** (Action, Drama, Comedy, etc.)
- **Theme tags** (School, Military, Magic, etc.)
- **Demographic tags** (Shounen, Seinen, etc.)
- **Studio tags** (Production studios)

### Relationships
- **`has_tag`**: Anime → Tags with relevance scores
- **`is_sequel`**: Anime → Anime sequel relationships
- **Episodes**: Individual episode records linked to anime

### Episode Records
- **Episode numbers, titles, duration**
- **Air dates and synopsis** (where available)
- **Linked to parent anime**

## 🏗️ Database Schema

The import creates the following SurrealDB structure:

```sql
-- Core entities
DEFINE TABLE anime SCHEMAFULL;
DEFINE TABLE tag SCHEMAFULL;  
DEFINE TABLE episode SCHEMAFULL;

-- Relationship tables
DEFINE TABLE has_tag SCHEMALESS;     -- Anime → Tags
DEFINE TABLE is_sequel SCHEMALESS;   -- Anime → Anime
```

## 📈 Performance & Statistics

The import process provides detailed statistics:

- **Import counts**: Anime, tags, episodes imported
- **Relationship counts**: Tag and sequel relationships created
- **Dead entry filtering**: Number of invalid entries filtered out
- **Error tracking**: Failed imports with error details
- **Success rate**: Percentage of successful imports
- **Timing**: Total import duration

## 🔍 Example Queries

After import, you can run powerful graph queries:

```sql
-- Get anime with their tags
SELECT title, ->has_tag->tag.name AS tags FROM anime:some_id;

-- Find sequels
SELECT title, ->is_sequel->anime.title AS sequels FROM anime WHERE title @@ "Attack";

-- Tag-based recommendations  
SELECT title FROM anime WHERE id IN (
  SELECT in FROM has_tag WHERE out IN (
    SELECT out FROM has_tag WHERE in = "anime:some_id"
  ) AND in != "anime:some_id"
);

-- Episodes for an anime
SELECT * FROM episode WHERE anime_id = "anime:some_id" ORDER BY episode_number;
```

## ⚡ Performance Tips

1. **Start small**: Use `test` or `small` mode first
2. **Skip episodes**: Use `--skip-episodes` for faster imports
3. **Monitor resources**: Large imports can be memory-intensive
4. **Use batching**: The script automatically batches for performance

## 🐛 Troubleshooting

### SurrealDB Connection Issues
```bash
# Check if SurrealDB is running
curl http://localhost:8000/health

# Start SurrealDB with Docker
docker-compose up -d surrealdb
```

### Data File Issues
```bash
# Check if data files exist
ls -la ../data/anime-offline-database.json
ls -la ../data/myanimelist.json
```

### Python Dependencies
```bash
# Install requests manually
pip3 install requests

# Check Python version
python3 --version
```

## 📝 Notes

- **Dead entries**: ~32K of ~39K entries are filtered out as "dead" (invalid/removed from MAL)
- **Rate limiting**: Built-in delays prevent overwhelming the database
- **Idempotent**: Safe to run multiple times (may create duplicates)
- **Memory usage**: Large imports may require significant memory
- **Duration**: Full import can take several hours depending on system

## 🎯 Next Steps

After importing data, you can:

1. **Test queries** using the SurrealDB web interface
2. **Build APIs** using the imported graph data
3. **Create recommendation engines** based on tag relationships
4. **Implement search** using the full-text indexes
5. **Add user interactions** (ratings, favorites, watch lists)

---

For more advanced usage or customization, see the source code in `import_mal_data.py`.