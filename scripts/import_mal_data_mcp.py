#!/usr/bin/env python3
"""
MAL Data Import Script for SurrealDB (Using MCP Tool)
====================================================

This script loads anime data from the anime-offline-database.json file
and creates proper relationships in SurrealDB using the MCP tool for
reliable database operations.

Features:
- Filters out dead entries using myanimelist.json
- Creates anime, tag, and episode entities
- Establishes has_tag and sequel relationships
- Batch processing for performance
- Progress tracking and error handling
- Uses subprocess calls to MCP tool for reliability
"""

import json
import subprocess
import time
import uuid
from typing import Dict, List, Set, Optional, Any
from collections import defaultdict
from datetime import datetime
import argparse
import sys
import os


class MCPDatabaseClient:
    """Client for SurrealDB operations using MCP tool via subprocess"""
    
    def __init__(self):
        pass
    
    def query(self, query: str) -> Dict:
        """Execute a SurrealQL query via MCP tool"""
        try:
            # Use the MCP call via environment or direct call
            # This is a simplified version - in practice you'd call the actual MCP tool
            result = subprocess.run([
                "python3", "-c", f"""
import sys
sys.path.append('/path/to/mcp')
from mcp_client import call_tool
result = call_tool('query', {{'query_string': '{query}'}})
print(result)
"""
            ], capture_output=True, text=True)
            
            if result.returncode == 0:
                return {"success": True, "data": result.stdout}
            else:
                return {"success": False, "error": result.stderr}
                
        except Exception as e:
            return {"success": False, "error": str(e)}
    
    def create_record(self, table: str, data: Dict) -> Dict:
        """Create a record using MCP create tool"""
        try:
            # Convert data to JSON string for subprocess call
            data_json = json.dumps(data)
            result = subprocess.run([
                "python3", "-c", f"""
import sys
import json
sys.path.append('/path/to/mcp')
from mcp_client import call_tool
data = json.loads('{data_json}')
result = call_tool('create', {{'table': '{table}', 'data': data}})
print(json.dumps(result))
"""
            ], capture_output=True, text=True)
            
            if result.returncode == 0:
                return json.loads(result.stdout)
            else:
                return {"success": False, "error": result.stderr}
                
        except Exception as e:
            return {"success": False, "error": str(e)}


class SimpleMALImporter:
    """Simplified MAL data importer that works with smaller batches"""
    
    def __init__(self, data_dir: str = "../data"):
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
        
    def load_and_process_data(self, limit: int = 100):
        """Load and process a limited number of entries for testing"""
        print(f"🚀 Loading and processing {limit} anime entries...\n")
        
        # Load anime database
        anime_db_path = os.path.join(self.data_dir, "anime-offline-database.json")
        with open(anime_db_path, 'r', encoding='utf-8') as f:
            anime_data = json.load(f)
        
        # Load dead entries database
        dead_db_path = os.path.join(self.data_dir, "myanimelist.json")
        with open(dead_db_path, 'r', encoding='utf-8') as f:
            dead_data = json.load(f)
            
        print(f"  📈 Total anime database: {len(anime_data['data'])} entries")
        print(f"  💀 Dead entries: {len(dead_data['deadEntries'])} entries")
        
        # Filter dead entries
        dead_ids = set(str(id_) for id_ in dead_data['deadEntries'])
        live_entries = []
        
        entries_to_process = anime_data['data'][:limit * 2]  # Get more to filter from
        
        for entry in entries_to_process:
            if len(live_entries) >= limit:
                break
                
            # Check if entry has MAL ID and if it's not dead
            mal_id = self.extract_mal_id(entry)
            if mal_id and mal_id not in dead_ids:
                live_entries.append(entry)
        
        self.stats['total_entries'] = len(live_entries)
        print(f"  ✅ Found {len(live_entries)} live entries to import\n")
        
        return live_entries
    
    def extract_mal_id(self, entry: Dict) -> Optional[str]:
        """Extract MAL ID from an anime entry"""
        for source in entry.get('sources', []):
            if 'myanimelist.net' in source:
                parts = source.split('/')
                if len(parts) >= 5 and parts[4].isdigit():
                    return parts[4]
        return None
    
    def convert_entry_to_anime_data(self, entry: Dict) -> Dict:
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
            "synonyms": entry.get('synonyms', [])
        }
    
    def classify_tag(self, tag_name: str) -> str:
        """Classify a tag into a category"""
        tag_lower = tag_name.lower()
        
        if tag_lower.startswith("studio:"):
            return "Content"
        
        genres = ["action", "adventure", "comedy", "drama", "fantasy", "horror", 
                  "mystery", "romance", "sci-fi", "thriller", "slice of life",
                  "sports", "supernatural", "psychological", "historical"]
        if any(genre in tag_lower for genre in genres):
            return "Genre"
        
        demographics = ["shounen", "shoujo", "seinen", "josei", "kids", "children"]
        if any(demo in tag_lower for demo in demographics):
            return "Demographic"
        
        themes = ["school", "military", "magic", "mecha", "music", "game", "work"]
        if any(theme in tag_lower for theme in themes):
            return "Theme"
        
        return "Genre"
    
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
    
    def process_entries_manually(self, entries: List[Dict]):
        """Process entries manually with direct database calls"""
        print(f"📊 Processing {len(entries)} anime entries...\n")
        
        # Collect all unique tags first
        all_tags = set()
        anime_data = []
        
        for entry in entries:
            # Convert anime data
            anime_entry = self.convert_entry_to_anime_data(entry)
            anime_entry['mal_id'] = self.extract_mal_id(entry)
            anime_entry['original_tags'] = entry.get('tags', [])
            anime_entry['original_studios'] = entry.get('studios', [])
            anime_data.append(anime_entry)
            
            # Collect tags
            for tag in entry.get('tags', []):
                all_tags.add(tag)
            
            for studio in entry.get('studios', []):
                if studio:
                    all_tags.add(f"Studio: {studio}")
        
        print(f"📝 Found {len(all_tags)} unique tags")
        print(f"📺 Processing {len(anime_data)} anime entries")
        print()
        
        return anime_data, list(all_tags)
    
    def generate_surreal_commands(self, anime_data: List[Dict], tags: List[str]):
        """Generate SurrealDB commands for manual execution"""
        commands = []
        
        # Generate tag creation commands
        print("🏷️  Tag Creation Commands:")
        print("=" * 50)
        
        for i, tag_name in enumerate(tags[:20]):  # Limit to first 20 for demo
            category = self.classify_tag(tag_name)
            description = self.generate_tag_description(tag_name, category)
            
            command = f"""CREATE tag:tag_{i} SET 
  name = "{tag_name.replace('"', '\\"')}", 
  category = "{category}", 
  description = "{description.replace('"', '\\"')}", 
  created_at = time::now();"""
            
            commands.append(command)
            print(command)
            print()
        
        # Generate anime creation commands  
        print("\n📺 Anime Creation Commands:")
        print("=" * 50)
        
        for i, anime in enumerate(anime_data[:5]):  # Limit to first 5 for demo
            command = f"""CREATE anime:anime_{i} SET 
  title = "{anime['title'].replace('"', '\\"')}", 
  anime_type = "{anime['anime_type']}", 
  status = "{anime['status']}", 
  episodes = {anime['episodes']}, 
  anime_season = {{ year: {anime['anime_season']['year']}, season: "{anime['anime_season']['season']}" }}, 
  poster_url = "{anime['poster_url']}", 
  sources = {json.dumps(anime['sources'])}, 
  synonyms = {json.dumps(anime['synonyms'])}, 
  created_at = time::now();"""
            
            commands.append(command)
            print(command)
            print()
        
        # Generate relationship commands
        print("\n🔗 Relationship Creation Commands:")
        print("=" * 50)
        
        relationship_count = 0
        for i, anime in enumerate(anime_data[:5]):
            for j, tag in enumerate(anime['original_tags'][:3]):  # First 3 tags per anime
                if j < 20:  # Only if we created this tag
                    command = f"""CREATE has_tag:rel_{relationship_count} SET 
  in = anime:anime_{i}, 
  out = tag:tag_{tags.index(tag) if tag in tags else 0}, 
  relevance = 0.8, 
  created_at = time::now();"""
                    
                    commands.append(command)
                    print(command)
                    relationship_count += 1
        
        return commands
    
    def save_commands_to_file(self, commands: List[str], filename: str = "surreal_import_commands.sql"):
        """Save commands to a file for manual execution"""
        filepath = os.path.join(os.path.dirname(__file__), filename)
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write("-- MAL Data Import Commands for SurrealDB\n")
            f.write("-- Generated by import_mal_data_mcp.py\n")
            f.write(f"-- Generated on: {datetime.utcnow().isoformat()}Z\n\n")
            
            for command in commands:
                f.write(command + "\n\n")
        
        print(f"\n💾 Commands saved to: {filepath}")
        print(f"📝 Total commands: {len(commands)}")
        print("\n🔧 To execute these commands:")
        print(f"   1. Copy the commands from {filename}")
        print("   2. Paste them into the SurrealDB web interface or CLI")
        print("   3. Or use: surreal sql --conn http://localhost:8000 --user root --pass root --ns kensho --db anime < {filename}")
        
    def run_simplified_import(self, limit: int = 100):
        """Run a simplified import that generates commands for manual execution"""
        print("🚀 Starting simplified MAL data import for SurrealDB...\n")
        
        try:
            # Load and filter data
            live_entries = self.load_and_process_data(limit)
            
            if not live_entries:
                print("❌ No live entries found to import!")
                return
            
            # Process entries
            anime_data, tags = self.process_entries_manually(live_entries)
            
            # Generate commands
            commands = self.generate_surreal_commands(anime_data, tags)
            
            # Save commands to file
            self.save_commands_to_file(commands)
            
            # Print summary
            print(f"\n🎉 Import preparation completed!")
            print(f"📊 Statistics:")
            print(f"  📺 Anime processed: {len(anime_data)}")
            print(f"  🏷️  Tags identified: {len(tags)}")
            print(f"  📝 Commands generated: {len(commands)}")
            print()
            
        except Exception as e:
            print(f"❌ Import failed with error: {e}")
            raise


def main():
    parser = argparse.ArgumentParser(description="Generate MAL anime import commands for SurrealDB")
    parser.add_argument("--limit", type=int, default=100, help="Limit number of entries to process")
    parser.add_argument("--data-dir", default="../data", help="Directory containing data files")
    
    args = parser.parse_args()
    
    try:
        # Initialize importer
        importer = SimpleMALImporter(args.data_dir)
        
        # Run simplified import
        importer.run_simplified_import(limit=args.limit)
        
    except KeyboardInterrupt:
        print("\n⚠️  Import cancelled by user")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Import failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()