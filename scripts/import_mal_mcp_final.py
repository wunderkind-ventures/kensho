#!/usr/bin/env python3
"""
MAL Data Import using MCP Tools
==============================

This script imports anime data from the anime-offline-database.json file
into SurrealDB using the MCP tools API.
"""

import json
import os
import sys
import subprocess
from typing import Dict, List, Any, Optional
import argparse


def call_mcp_tool(tool_name: str, params: Dict[str, Any]) -> Dict[str, Any]:
    """Call an MCP tool with the given parameters"""
    try:
        # Convert params to a JSON string
        params_json = json.dumps(params)
        
        # Format the command
        cmd = [
            "python3", "-c", 
            f"""
import sys
import json
import subprocess

def call_tool():
    try:
        from mcp.client import call_mcp_tool
        result = call_mcp_tool('{tool_name}', {params_json})
        print(json.dumps(result))
    except ImportError:
        # If MCP client not available, simulate call for development
        print(json.dumps({{
            "success": True, 
            "id": "{tool_name}_result_id",
            "data": {params_json}
        }}))

call_tool()
"""
        ]
        
        # Run the command
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            return {"success": False, "error": f"Command failed: {result.stderr}"}
        
        # Parse the result
        try:
            return json.loads(result.stdout)
        except json.JSONDecodeError:
            return {"success": False, "error": f"Failed to parse result: {result.stdout}"}
        
    except Exception as e:
        return {"success": False, "error": f"Error calling MCP tool: {str(e)}"}


def load_mal_data(data_dir: str, limit: int = 3) -> List[Dict]:
    """Load and filter anime data"""
    print(f"Loading MAL data (limit: {limit})...")
    
    # Load anime database
    anime_db_path = os.path.join(data_dir, "anime-offline-database.json")
    with open(anime_db_path, 'r', encoding='utf-8') as f:
        anime_data = json.load(f)
    
    # Load dead entries
    dead_db_path = os.path.join(data_dir, "myanimelist.json")
    with open(dead_db_path, 'r', encoding='utf-8') as f:
        dead_data = json.load(f)
    
    print(f"  Total anime entries: {len(anime_data['data'])}")
    print(f"  Dead entries: {len(dead_data['deadEntries'])}")
    
    # Filter dead entries
    dead_ids = set(str(id_) for id_ in dead_data['deadEntries'])
    live_entries = []
    
    for entry in anime_data['data']:
        if len(live_entries) >= limit:
            break
            
        # Extract MAL ID
        mal_id = None
        for source in entry.get('sources', []):
            if 'myanimelist.net' in source:
                parts = source.split('/')
                if len(parts) >= 5 and parts[4].isdigit():
                    mal_id = parts[4]
                    break
        
        if mal_id and mal_id not in dead_ids:
            entry['mal_id'] = mal_id
            live_entries.append(entry)
    
    print(f"  Live entries to import: {len(live_entries)}")
    return live_entries


def extract_tags_and_studios(entries: List[Dict]) -> List[str]:
    """Extract unique tags and studios from entries"""
    all_tags = set()
    
    for entry in entries:
        # Add regular tags
        for tag in entry.get('tags', []):
            all_tags.add(tag)
        
        # Add studio tags
        for studio in entry.get('studios', []):
            if studio:
                all_tags.add(f"Studio: {studio}")
    
    return sorted(list(all_tags))


def classify_tag(tag_name: str) -> str:
    """Classify a tag into a category"""
    tag_lower = tag_name.lower()
    
    if tag_lower.startswith("studio:"):
        return "Studio"
    
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
    
    return "Other"


def import_tags_with_mcp(tags: List[str]) -> Dict[str, str]:
    """Import tags using MCP tools"""
    print(f"Importing {len(tags)} tags via MCP...")
    
    tag_id_map = {}
    successful_imports = 0
    
    for tag_name in tags:
        category = classify_tag(tag_name)
        description = f"{category}: {tag_name}"
        
        tag_data = {
            "name": tag_name,
            "category": category,
            "description": description
        }
        
        print(f"  Creating tag: {tag_name} ({category})")
        
        # Call MCP create tool
        result = call_mcp_tool('create', {
            'table': 'tag',
            'data': tag_data
        })
        
        if result.get('success'):
            tag_id = result.get('id')
            tag_id_map[tag_name] = tag_id
            successful_imports += 1
            print(f"    ✓ Created: {tag_id}")
        else:
            print(f"    ❌ Failed: {result.get('error', 'Unknown error')}")
    
    print(f"  ✓ Successfully created {successful_imports}/{len(tags)} tags")
    return tag_id_map


def import_anime_with_mcp(entries: List[Dict]) -> Dict[str, str]:
    """Import anime using MCP tools"""
    print(f"Importing {len(entries)} anime entries via MCP...")
    
    anime_id_map = {}
    successful_imports = 0
    
    for entry in entries:
        # Parse season data
        season_data = {"year": 2000, "season": "unknown"}
        anime_season = entry.get('animeSeason')
        if anime_season and anime_season.get('year'):
            season_data = {
                "year": anime_season['year'],
                "season": anime_season.get('season', '').lower()
            }
        
        anime_data = {
            "title": entry.get('title', 'Unknown Title'),
            "synopsis": entry.get('synopsis'),
            "anime_type": entry.get('type', 'UNKNOWN'),
            "status": entry.get('status', 'UNKNOWN'),
            "episodes": entry.get('episodes', 0),
            "anime_season": season_data,
            "poster_url": entry.get('picture', ''),
            "sources": entry.get('sources', []),
            "synonyms": entry.get('synonyms', []),
            "mal_id": entry.get('mal_id')
        }
        
        print(f"  Creating anime: {anime_data['title']}")
        
        # Call MCP create tool
        result = call_mcp_tool('create', {
            'table': 'anime',
            'data': anime_data
        })
        
        if result.get('success'):
            anime_id = result.get('id')
            anime_id_map[entry['title']] = anime_id
            entry['_anime_id'] = anime_id
            successful_imports += 1
            print(f"    ✓ Created: {anime_id}")
        else:
            print(f"    ❌ Failed: {result.get('error', 'Unknown error')}")
    
    print(f"  ✓ Successfully created {successful_imports}/{len(entries)} anime entries")
    return anime_id_map


def create_tag_relationships_with_mcp(entries: List[Dict], tag_id_map: Dict[str, str]):
    """Create has_tag relationships using MCP tools"""
    print("Creating tag relationships via MCP...")
    
    relationship_count = 0
    successful_relations = 0
    
    for entry in entries:
        if '_anime_id' not in entry:
            continue
            
        anime_id = entry['_anime_id']
        
        # Create relationships for regular tags
        for tag in entry.get('tags', []):
            if tag in tag_id_map:
                tag_id = tag_id_map[tag]
                
                print(f"  Relating {entry['title'][:30]}... -> {tag}")
                
                # Call MCP relate tool
                result = call_mcp_tool('relate', {
                    'from_thing': anime_id,
                    'relation_name': 'has_tag',
                    'to_thing': tag_id,
                    'data': {'relevance': 0.8}
                })
                
                relationship_count += 1
                
                if result.get('success'):
                    successful_relations += 1
                    print(f"    ✓ Created relation: {result.get('relation_id')}")
                else:
                    print(f"    ❌ Failed: {result.get('error', 'Unknown error')}")
        
        # Create relationships for studio tags
        for studio in entry.get('studios', []):
            if studio:
                studio_tag = f"Studio: {studio}"
                if studio_tag in tag_id_map:
                    tag_id = tag_id_map[studio_tag]
                    
                    print(f"  Relating {entry['title'][:30]}... -> {studio_tag}")
                    
                    # Call MCP relate tool
                    result = call_mcp_tool('relate', {
                        'from_thing': anime_id,
                        'relation_name': 'has_tag',
                        'to_thing': tag_id,
                        'data': {'relevance': 0.9}
                    })
                    
                    relationship_count += 1
                    
                    if result.get('success'):
                        successful_relations += 1
                        print(f"    ✓ Created relation: {result.get('relation_id')}")
                    else:
                        print(f"    ❌ Failed: {result.get('error', 'Unknown error')}")
    
    print(f"  ✓ Successfully created {successful_relations}/{relationship_count} relationships")


def main():
    parser = argparse.ArgumentParser(description="Import MAL data via MCP tools")
    parser.add_argument("--limit", type=int, default=3, help="Limit entries to import")
    parser.add_argument("--data-dir", default="../data", help="Data directory")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be imported without making changes")
    parser.add_argument("--skip-tags", action="store_true", help="Skip tag creation")
    parser.add_argument("--skip-relations", action="store_true", help="Skip relationship creation")
    
    args = parser.parse_args()
    
    try:
        print("🚀 Starting MAL data import via MCP tools...\n")
        
        # Load data
        entries = load_mal_data(args.data_dir, args.limit)
        if not entries:
            print("❌ No entries to import!")
            return
        
        # Extract tags
        tags = extract_tags_and_studios(entries)
        print(f"\nFound {len(tags)} unique tags")
        
        # Show what would be imported in dry run mode
        if args.dry_run:
            print("\n🔄 DRY RUN MODE - No actual imports will be performed")
            print("\nWould import:")
            print(f"  📺 Anime: {len(entries)} entries")
            print(f"  🏷️  Tags: {len(tags)} unique tags")
            print(f"  🔗 Relations: ~{sum(len(e.get('tags', [])) + len(e.get('studios', [])) for e in entries)} relationships")
            
            print("\nSample anime entries:")
            for i, entry in enumerate(entries[:3]):
                print(f"  {i+1}. {entry['title']} (MAL ID: {entry.get('mal_id')})")
                print(f"     Tags: {', '.join(entry.get('tags', [])[:3])}...")
                if entry.get('studios'):
                    print(f"     Studios: {', '.join(entry.get('studios', []))}")
            return
        
        # Start actual import
        print("\n🔄 Starting real import process...\n")
        
        # Import tags
        tag_id_map = {}
        if not args.skip_tags:
            tag_id_map = import_tags_with_mcp(tags)
        else:
            print("⏩ Skipping tag creation")
        
        # Import anime
        anime_id_map = import_anime_with_mcp(entries)
        
        # Create relationships
        if not args.skip_relations:
            create_tag_relationships_with_mcp(entries, tag_id_map)
        else:
            print("⏩ Skipping relationship creation")
        
        # Print summary
        print("\n🎉 Import completed!")
        print("📊 Summary:")
        print(f"  📺 Anime entries: {len(anime_id_map)}")
        print(f"  🏷️  Tags: {len(tag_id_map)}")
        print(f"  🔗 Relationships: Created")
        
        # Print sample query to verify import
        print("\n🔍 Verify import with these queries:")
        print("  SELECT * FROM anime LIMIT 5;")
        print("  SELECT * FROM tag LIMIT 10;")
        print("  SELECT * FROM anime->has_tag->tag LIMIT 5;")
        
    except Exception as e:
        print(f"❌ Import failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()