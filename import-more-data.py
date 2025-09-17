#!/usr/bin/env python3
"""
Import a larger dataset of anime into the backend
"""
import json
import requests
import sys
import time

def main():
    # Read the anime database
    with open('/Users/kennethsylvain/WKV/Enterprise/kensho/.data/anime-offline-database.json', 'r') as f:
        data = json.load(f)
    
    anime_entries = data['data']
    print(f"Found {len(anime_entries)} anime entries")
    
    # Import 100 anime entries, skipping the first 10 we already imported
    start_index = 10
    limit = 100
    base_url = "http://localhost:3000"
    
    # Check if backend is running
    try:
        health = requests.get(f"{base_url}/api/health")
        if health.status_code != 200:
            print("Backend is not healthy!")
            sys.exit(1)
    except:
        print("Backend is not running! Start it with: cargo run --bin backend-server")
        sys.exit(1)
    
    print(f"\nImporting anime entries {start_index+1} to {start_index+limit}...")
    success_count = 0
    failed_count = 0
    
    for i, entry in enumerate(anime_entries[start_index:start_index+limit], start=start_index+1):
        # Convert anime-offline-database format to our API format
        anime_data = {
            "title": entry.get("title", "Unknown"),
            "synonyms": entry.get("synonyms", []),
            "sources": entry.get("sources", []),
            "episodes": entry.get("episodes", 0),
            "status": entry.get("status", "UNKNOWN"),
            "anime_type": entry.get("type", "UNKNOWN"),
            "anime_season": {
                "season": entry.get("animeSeason", {}).get("season", "spring").lower(),
                "year": entry.get("animeSeason", {}).get("year", 2024)
            },
            "synopsis": entry.get("synopsis", ""),
            "poster_url": entry.get("picture", ""),
            "tags": entry.get("tags", [])[:10]  # Limit tags to 10
        }
        
        # Send POST request to create anime
        try:
            response = requests.post(
                f"{base_url}/api/anime",
                json=anime_data,
                headers={"Content-Type": "application/json"}
            )
            
            if response.status_code == 201:
                created_anime = response.json()
                anime_id = created_anime.get("id")
                
                # Create a limited number of episodes for performance
                if anime_data['episodes'] > 0:
                    episodes_data = {
                        "episodes": [
                            {
                                "episode_number": ep,
                                "title": f"Episode {ep}",
                                "duration": None,
                                "air_date": None,
                                "synopsis": None,
                                "thumbnail_url": None
                            }
                            for ep in range(1, min(anime_data['episodes'] + 1, 4))  # Max 3 episodes per anime
                        ]
                    }
                    
                    requests.post(
                        f"{base_url}/api/anime/{anime_id}/episodes",
                        json=episodes_data,
                        headers={"Content-Type": "application/json"}
                    )
                    
                success_count += 1
                if success_count % 10 == 0:
                    print(f"  Progress: {success_count}/{limit} anime imported")
            else:
                failed_count += 1
                if failed_count <= 5:  # Only show first 5 errors
                    print(f"  ✗ Failed: {anime_data['title'][:50]}")
                    
        except Exception as e:
            failed_count += 1
            if failed_count <= 5:
                print(f"  ✗ Error: {anime_data['title'][:50]} - {str(e)[:50]}")
        
        # Small delay to avoid overwhelming the server
        if (i % 20) == 0:
            time.sleep(0.1)
    
    print(f"\n✅ Import complete!")
    print(f"  • Successfully imported: {success_count} anime")
    if failed_count > 0:
        print(f"  • Failed imports: {failed_count} anime")
    print(f"  • Total in database: ~{success_count + 11} anime (including previous imports)")

if __name__ == "__main__":
    main()