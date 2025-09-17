#!/usr/bin/env python3
"""
Simple script to import anime data into the backend API
"""
import json
import requests
import sys

def main():
    # Read the anime database
    with open('/Users/kennethsylvain/WKV/Enterprise/kensho/.data/anime-offline-database.json', 'r') as f:
        data = json.load(f)
    
    anime_entries = data['data']
    print(f"Found {len(anime_entries)} anime entries")
    
    # Process first 10 entries
    limit = 10
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
    
    print(f"\nImporting {limit} anime entries...")
    success_count = 0
    failed_count = 0
    
    for i, entry in enumerate(anime_entries[:limit]):
        # Convert anime-offline-database format to our API format
        anime_data = {
            "title": entry.get("title", "Unknown"),
            "synonyms": entry.get("synonyms", []),
            "sources": entry.get("sources", []),
            "episodes": entry.get("episodes", 0),
            "status": entry.get("status", "UNKNOWN"),
            "anime_type": entry.get("type", "UNKNOWN"),
            "anime_season": {
                # Convert season to lowercase for API compatibility
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
                print(f"✓ {i+1}. Created: {anime_data['title']} (ID: {anime_id})")
                
                # Create episodes for this anime
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
                            for ep in range(1, min(anime_data['episodes'] + 1, 6))  # Limit to 5 episodes for testing
                        ]
                    }
                    
                    ep_response = requests.post(
                        f"{base_url}/api/anime/{anime_id}/episodes",
                        json=episodes_data,
                        headers={"Content-Type": "application/json"}
                    )
                    
                    if ep_response.status_code in [201, 206]:
                        print(f"  → Created {len(episodes_data['episodes'])} episodes")
                    
                success_count += 1
            else:
                print(f"✗ {i+1}. Failed: {anime_data['title']}")
                print(f"  Error: {response.status_code} - {response.text[:100]}")
                failed_count += 1
        except Exception as e:
            print(f"✗ {i+1}. Error: {anime_data['title']} - {str(e)}")
            failed_count += 1
    
    print(f"\n✓ Successfully imported {success_count} anime entries")
    if failed_count > 0:
        print(f"✗ Failed to import {failed_count} anime entries")
    print(f"\nTotal processed: {success_count + failed_count}/{limit}")

if __name__ == "__main__":
    main()