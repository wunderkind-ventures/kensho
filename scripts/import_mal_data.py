#!/usr/bin/env python3
"""
MAL Data Import Script for SurrealDB
====================================

This script loads anime data from the anime-offline-database.json file
and creates proper relationships in SurrealDB.

Features:
- Filters out dead entries using myanimelist.json
- Creates anime, tag, and episode entities
- Establishes has_tag and sequel relationships
- Batch processing for performance
- Progress tracking and error handling
"""

import json
import requests
import time
import uuid
from typing import Dict, List, Set, Optional, Any
from collections import defaultdict
from datetime import datetime
import argparse
import sys
import os
from urllib.parse import urlparse


class SurrealDBClient:
    """HTTP client for SurrealDB operations"""
    
    def __init__(self, url: str = "http://localhost:8000", username: str = "root", password: str = "root"):
        self.base_url = url.rstrip('/')
        self.session = requests.Session()
        self.ns = "kensho"
        self.db = "anime"
        
        # Authenticate
        self._signin(username, password)
        
    def _signin(self, username: str, password: str):
        """Authenticate with SurrealDB"""
        auth_data = f"{username}:{password}"
        import base64
        encoded_auth = base64.b64encode(auth_data.encode()).decode()
        self.session.headers.update({
            "Authorization": f"Basic {encoded_auth}"
        })
        
        # Also try the /signin endpoint
        try:
            response = self.session.post(f"{self.base_url}/signin", json={
                "user": username,
                "pass": password
            })
            if response.status_code == 200 and response.text:
                # Store the token if returned
                token = response.text.strip('"')  # Remove quotes if present
                self.session.headers.update({
                    "Authorization": f"Bearer {token}"
                })
        except Exception as e:
            print(f"⚠️  Authentication warning: {e}")
        
    def query(self, query: str, variables: Optional[Dict] = None) -> Dict:
        """Execute a SurrealQL query"""
        payload = {"query": query}
        if variables:
            payload["variables"] = variables
            
        headers = {
            "Content-Type": "application/json",
            "NS": self.ns,
            "DB": self.db
        }
        
        response = self.session.post(f"{self.base_url}/sql", json=payload, headers=headers)
        
        if response.status_code != 200:
            raise Exception(f"Query failed: {response.status_code} - {response.text}")
            
        return response.json()
    
    def create_record(self, table: str, data: Dict, record_id: Optional[str] = None) -> Dict:
        """Create a record in the specified table"""
        if record_id:
            query = f"CREATE {table}:{record_id} CONTENT $data"
        else:
            query = f"CREATE {table} CONTENT $data"
        
        result = self.query(query, {"data": data})
        return result
    
    def relate(self, from_id: str, relation: str, to_id: str, data: Optional[Dict] = None) -> Dict:
        """Create a relationship between two records"""
        if data:
            query = f"RELATE {from_id}->{relation}->{to_id} CONTENT $data"
            return self.query(query, {"data": data})
        else:
            query = f"RELATE {from_id}->{relation}->{to_id}"
            return self.query(query)


class MALDataImporter:
    """Main class for importing MAL data into SurrealDB"""
    
    def __init__(self, db_client: SurrealDBClient, data_dir: str = "../data"):
        self.db = db_client
        self.data_dir = data_dir
        self.stats = {
            "total_entries": 0,
            "dead_filtered": 0,
            "imported_anime": 0,
            "imported_tags": 0,
            "imported_episodes": 0,
            "tag_relationships": 0,
            "anime_relationships": 0,
            "errors": 0
        }
        
        # Caches for performance
        self.anime_id_map = {}  # MAL ID -> SurrealDB ID
        self.tag_name_map = {}  # Tag name -> SurrealDB ID
        
    def load_data_files(self):
        """Load the JSON data files"""
        print("📂 Loading data files...")
        
        # Load anime database
        anime_db_path = os.path.join(self.data_dir, "anime-offline-database.json")
        with open(anime_db_path, 'r', encoding='utf-8') as f:
            self.anime_data = json.load(f)
        
        # Load dead entries database
        dead_db_path = os.path.join(self.data_dir, "myanimelist.json")
        with open(dead_db_path, 'r', encoding='utf-8') as f:
            self.dead_data = json.load(f)
            
        print(f"  📈 Anime database: {len(self.anime_data['data'])} entries")
        print(f"  💀 Dead entries: {len(self.dead_data['deadEntries'])} entries")
        
    def filter_dead_entries(self, limit: Optional[int] = None) -> List[Dict]:
        """Filter out dead entries from the anime data"""
        print("🔍 Filtering out dead entries...")
        
        dead_ids = set(str(id_) for id_ in self.dead_data['deadEntries'])
        live_entries = []
        
        entries_to_process = self.anime_data['data'][:limit] if limit else self.anime_data['data']
        self.stats['total_entries'] = len(entries_to_process)
        
        for entry in entries_to_process:
            # Check if entry has MAL ID and if it's not dead
            mal_id = self.extract_mal_id(entry)
            if mal_id and mal_id not in dead_ids:
                live_entries.append(entry)
        
        self.stats['dead_filtered'] = self.stats['total_entries'] - len(live_entries)
        print(f"  💀 Filtered {self.stats['dead_filtered']} dead entries")
        print(f"  ✅ {len(live_entries)} live entries to import")
        
        return live_entries
    
    def extract_mal_id(self, entry: Dict) -> Optional[str]:
        """Extract MAL ID from an anime entry"""
        for source in entry.get('sources', []):
            if 'myanimelist.net' in source:
                # Extract ID from URL like https://myanimelist.net/anime/16498/Shingeki_no_Kyojin
                parts = source.split('/')
                if len(parts) >= 5 and parts[4].isdigit():
                    return parts[4]
        return None
    
    def create_tags(self, entries: List[Dict]) -> int:
        """Create all unique tags from the entries"""
        print("🏷️  Creating tags...")
        
        all_tags = set()
        
        # Collect all unique tags
        for entry in entries:
            # Add genre tags
            for tag in entry.get('tags', []):
                all_tags.add(tag)
            
            # Add studio tags
            for studio in entry.get('studios', []):
                if studio:
                    all_tags.add(f"Studio: {studio}")
        
        print(f"    🔍 Found {len(all_tags)} unique tags")
        
        created = 0
        for tag_name in all_tags:
            try:
                category = self.classify_tag(tag_name)
                tag_data = {
                    "name": tag_name,
                    "category": category,
                    "description": self.generate_tag_description(tag_name, category),
                    "created_at": datetime.utcnow().isoformat() + "Z"
                }
                
                result = self.db.create_record("tag", tag_data)
                if result:
                    # Extract the created tag ID
                    tag_id = result[0]['id'] if isinstance(result, list) and result else None
                    if tag_id:
                        self.tag_name_map[tag_name] = tag_id
                        created += 1
                        
                        if created % 100 == 0:
                            print(f"    📝 Created {created} tags...")
                            
            except Exception as e:
                print(f"    ❌ Failed to create tag '{tag_name}': {e}")
                self.stats['errors'] += 1
                
        self.stats['imported_tags'] = created
        return created
    
    def classify_tag(self, tag_name: str) -> str:
        """Classify a tag into a category"""
        tag_lower = tag_name.lower()
        
        # Studio tags
        if tag_lower.startswith("studio:"):
            return "Content"
        
        # Common genres
        genres = ["action", "adventure", "comedy", "drama", "fantasy", "horror", 
                  "mystery", "romance", "sci-fi", "thriller", "slice of life",
                  "sports", "supernatural", "psychological", "historical"]
        if any(genre in tag_lower for genre in genres):
            return "Genre"
        
        # Demographics
        demographics = ["shounen", "shoujo", "seinen", "josei", "kids", "children"]
        if any(demo in tag_lower for demo in demographics):
            return "Demographic"
        
        # Themes
        themes = ["school", "military", "magic", "mecha", "music", "game", "work"]
        if any(theme in tag_lower for theme in themes):
            return "Theme"
        
        return "Genre"  # Default
    
    def generate_tag_description(self, tag_name: str, category: str) -> str:
        """Generate a description for a tag"""
        if tag_name.startswith("Studio: "):
            return f"Animation studio: {tag_name[8:]}"
        else:
            category_map = {
                "Genre": "Genre",
                "Theme": "Theme", 
                "Demographic": "Target demographic",
                "Content": "Content tag"
            }
            return f"{category_map.get(category, 'Tag')}: {tag_name}"
    
    def import_anime(self, entries: List[Dict]) -> int:
        """Import anime entries in batches"""
        print("📺 Importing anime entries...")
        
        batch_size = 50
        imported = 0
        
        for i in range(0, len(entries), batch_size):
            batch = entries[i:i+batch_size]
            print(f"  📦 Processing batch {i//batch_size + 1} ({i+len(batch)}/{len(entries)} entries)...")
            
            for entry in batch:
                try:
                    anime_data = self.convert_entry_to_anime(entry)
                    result = self.db.create_record("anime", anime_data)
                    
                    if result:
                        anime_id = result[0]['id'] if isinstance(result, list) and result else None
                        if anime_id:
                            # Store MAL ID mapping
                            mal_id = self.extract_mal_id(entry)
                            if mal_id:
                                self.anime_id_map[mal_id] = anime_id
                            imported += 1
                            
                except Exception as e:
                    print(f"    ❌ Failed to import '{entry.get('title', 'Unknown')}': {e}")
                    self.stats['errors'] += 1
            
            # Progress update and rate limiting
            if (i // batch_size) % 10 == 0:
                print(f"    📈 Imported {imported} anime so far...")
            
            time.sleep(0.1)  # Rate limiting
        
        self.stats['imported_anime'] = imported
        return imported
    
    def convert_entry_to_anime(self, entry: Dict) -> Dict:
        """Convert an offline database entry to SurrealDB anime format"""
        # Parse season data
        season_data = None
        anime_season = entry.get('animeSeason')
        if anime_season and anime_season.get('year'):
            season_data = {
                "year": anime_season['year'],
                "season": anime_season.get('season', '').lower()
            }
        
        # Default season if not available
        if not season_data:
            season_data = {"year": 2000, "season": "unknown"}
        
        return {
            "title": entry.get('title', 'Unknown Title'),
            "synopsis": entry.get('synopsis') or None,
            "anime_type": entry.get('type', 'UNKNOWN'),
            "status": entry.get('status', 'UNKNOWN'),
            "episodes": entry.get('episodes', 0),
            "anime_season": season_data,
            "poster_url": entry.get('picture', ''),
            "sources": entry.get('sources', []),
            "synonyms": entry.get('synonyms', []),
            "created_at": datetime.utcnow().isoformat() + "Z"
        }
    
    def create_tag_relationships(self, entries: List[Dict]) -> int:
        """Create tag relationships for anime"""
        print("🏷️  Creating tag relationships...")
        
        created = 0
        for entry in entries:
            mal_id = self.extract_mal_id(entry)
            anime_id = self.anime_id_map.get(mal_id)
            
            if not anime_id:
                continue
            
            # Create tag relationships
            for tag_name in entry.get('tags', []):
                tag_id = self.tag_name_map.get(tag_name)
                if tag_id:
                    try:
                        relevance = self.calculate_tag_relevance(tag_name, entry)
                        relation_data = {
                            "relevance": relevance,
                            "created_at": datetime.utcnow().isoformat() + "Z"
                        }
                        
                        # Create unique relationship ID
                        rel_id = f"tag_{len(str(created))}"
                        self.db.query(f"CREATE has_tag:{rel_id} SET in = '{anime_id}', out = '{tag_id}', relevance = {relevance}, created_at = time::now()")
                        created += 1
                        
                    except Exception as e:
                        print(f"    ❌ Failed to create tag relationship: {e}")
                        self.stats['errors'] += 1
            
            # Create studio tag relationships
            for studio in entry.get('studios', []):
                if studio:
                    studio_tag_name = f"Studio: {studio}"
                    tag_id = self.tag_name_map.get(studio_tag_name)
                    if tag_id:
                        try:
                            rel_id = f"studio_{len(str(created))}"
                            self.db.query(f"CREATE has_tag:{rel_id} SET in = '{anime_id}', out = '{tag_id}', relevance = 1.0, created_at = time::now()")
                            created += 1
                        except Exception as e:
                            print(f"    ❌ Failed to create studio relationship: {e}")
                            self.stats['errors'] += 1
        
        self.stats['tag_relationships'] = created
        return created
    
    def calculate_tag_relevance(self, tag_name: str, entry: Dict) -> float:
        """Calculate relevance score for a tag"""
        tag_count = len(entry.get('tags', []))
        if tag_count == 0:
            return 1.0
        
        # Base relevance inversely proportional to number of tags
        base_relevance = min(10.0 / max(tag_count, 1), 1.0)
        
        # Boost relevance for important tags
        important_tags = ["action", "drama", "comedy", "romance"]
        if tag_name.lower() in important_tags:
            base_relevance *= 1.2
        
        return min(base_relevance, 1.0)
    
    def create_anime_relationships(self, entries: List[Dict]) -> int:
        """Create sequel relationships between anime"""
        print("🔗 Creating anime relationships...")
        
        created = 0
        for entry in entries:
            source_mal_id = self.extract_mal_id(entry)
            source_anime_id = self.anime_id_map.get(source_mal_id)
            
            if not source_anime_id:
                continue
            
            # Process related anime URLs
            for related_url in entry.get('relatedAnime', []):
                related_mal_id = self.extract_mal_id_from_url(related_url)
                related_anime_id = self.anime_id_map.get(related_mal_id)
                
                if related_anime_id:
                    try:
                        rel_id = f"related_{len(str(created))}"
                        self.db.query(f"CREATE is_sequel:{rel_id} SET in = '{source_anime_id}', out = '{related_anime_id}', relationship_type = 'related', created_at = time::now()")
                        created += 1
                    except Exception as e:
                        print(f"    ❌ Failed to create anime relationship: {e}")
                        self.stats['errors'] += 1
        
        self.stats['anime_relationships'] = created
        return created
    
    def extract_mal_id_from_url(self, url: str) -> Optional[str]:
        """Extract MAL ID from a URL"""
        if 'myanimelist.net' in url:
            parts = url.split('/')
            for part in parts:
                if part.isdigit():
                    return part
        return None
    
    def create_episode_records(self, entries: List[Dict]) -> int:
        """Create episode records for anime"""
        print("📺 Creating episode records...")
        
        created = 0
        for entry in entries:
            mal_id = self.extract_mal_id(entry)
            anime_id = self.anime_id_map.get(mal_id)
            
            if not anime_id:
                continue
            
            episode_count = min(entry.get('episodes', 0), 500)  # Cap at 500
            if episode_count <= 0:
                continue
            
            for ep_num in range(1, episode_count + 1):
                try:
                    episode_data = {
                        "anime_id": anime_id,
                        "episode_number": ep_num,
                        "title": f"Episode {ep_num}",
                        "synopsis": None,
                        "duration": 24,  # Default 24 minutes
                        "air_date": None,
                        "created_at": datetime.utcnow().isoformat() + "Z"
                    }
                    
                    self.db.create_record("episode", episode_data)
                    created += 1
                    
                except Exception as e:
                    print(f"    ❌ Failed to create episode {ep_num}: {e}")
                    self.stats['errors'] += 1
        
        self.stats['imported_episodes'] = created
        return created
    
    def print_summary(self):
        """Print import summary statistics"""
        print("\n🎉 Import completed!")
        print("\n📊 Final Statistics:")
        print(f"  📺 Anime imported: {self.stats['imported_anime']}")
        print(f"  🏷️  Tags created: {self.stats['imported_tags']}")
        print(f"  📹 Episodes created: {self.stats['imported_episodes']}")
        print(f"  🔗 Anime relationships: {self.stats['anime_relationships']}")
        print(f"  🏷️  Tag relationships: {self.stats['tag_relationships']}")
        print(f"  💀 Dead entries filtered: {self.stats['dead_filtered']}")
        print(f"  ❌ Errors encountered: {self.stats['errors']}")
        print()
        
        if self.stats['total_entries'] > 0:
            success_rate = (self.stats['imported_anime'] / self.stats['total_entries']) * 100
            print(f"✅ Success rate: {success_rate:.1f}%")
        
        if self.stats['errors'] > 0:
            print(f"⚠️  {self.stats['errors']} errors encountered during import")
    
    def run_import(self, limit: Optional[int] = None, skip_relationships: bool = False, skip_episodes: bool = False):
        """Run the complete import process"""
        print("🚀 Starting comprehensive MAL data import to SurrealDB...\n")
        
        try:
            # Load data files
            self.load_data_files()
            
            # Filter dead entries
            live_entries = self.filter_dead_entries(limit)
            
            if not live_entries:
                print("❌ No live entries found to import!")
                return
            
            # Phase 1: Create tags
            if not skip_relationships:
                self.create_tags(live_entries)
                print(f"  ✅ Created {self.stats['imported_tags']} unique tags\n")
            
            # Phase 2: Import anime
            self.import_anime(live_entries)
            print(f"  ✅ Imported {self.stats['imported_anime']} anime entries\n")
            
            # Phase 3: Create relationships
            if not skip_relationships:
                self.create_anime_relationships(live_entries)
                print(f"  ✅ Created {self.stats['anime_relationships']} anime relationships\n")
                
                self.create_tag_relationships(live_entries)
                print(f"  ✅ Created {self.stats['tag_relationships']} tag relationships\n")
            
            # Phase 4: Create episodes
            if not skip_episodes:
                self.create_episode_records(live_entries)
                print(f"  ✅ Created {self.stats['imported_episodes']} episode records\n")
            
            # Final verification
            print("🔍 Final verification...")
            try:
                result = self.db.query("SELECT count() as count FROM anime GROUP ALL")
                total_anime = result[0]['count'] if result else 0
                print(f"  📊 Total anime in database: {total_anime}\n")
            except:
                print("  ⚠️  Could not verify final count\n")
            
            # Print summary
            self.print_summary()
            
        except Exception as e:
            print(f"❌ Import failed with error: {e}")
            raise


def main():
    parser = argparse.ArgumentParser(description="Import MAL anime data into SurrealDB")
    parser.add_argument("--limit", type=int, help="Limit number of entries to import")
    parser.add_argument("--skip-relationships", action="store_true", help="Skip creating relationships")
    parser.add_argument("--skip-episodes", action="store_true", help="Skip creating episode records")
    parser.add_argument("--db-url", default="http://localhost:8000", help="SurrealDB URL")
    parser.add_argument("--username", default="root", help="SurrealDB username")
    parser.add_argument("--password", default="root", help="SurrealDB password")
    parser.add_argument("--data-dir", default="../data", help="Directory containing data files")
    
    args = parser.parse_args()
    
    try:
        # Initialize SurrealDB client
        print(f"🔌 Connecting to SurrealDB at {args.db_url}...")
        db_client = SurrealDBClient(args.db_url, args.username, args.password)
        print("  ✅ Connected successfully")
        
        # Initialize importer
        importer = MALDataImporter(db_client, args.data_dir)
        
        # Run import
        importer.run_import(
            limit=args.limit,
            skip_relationships=args.skip_relationships,
            skip_episodes=args.skip_episodes
        )
        
    except KeyboardInterrupt:
        print("\n⚠️  Import cancelled by user")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Import failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()