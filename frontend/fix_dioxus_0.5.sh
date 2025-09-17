#!/bin/bash

# Script to fix Dioxus 0.5 compatibility issues
echo "🔧 Fixing Dioxus 0.5 compatibility issues..."

# Fix component signatures - remove cx: Scope parameter
echo "📝 Updating component signatures..."

# Update all component functions
find src -name "*.rs" -type f | while read file; do
    # Remove cx: Scope from function signatures
    sed -i '' 's/pub fn \([A-Za-z_]*\)(cx: Scope)/pub fn \1()/g' "$file"
    sed -i '' 's/pub fn \([A-Za-z_]*\)(cx: Scope, /pub fn \1(/g' "$file"
    
    # Update use_state to use_signal
    sed -i '' 's/use_state(cx, /use_signal(/g' "$file"
    sed -i '' 's/use_state(/use_signal(/g' "$file"
    
    # Update use_shared_state to use_context
    sed -i '' 's/use_shared_state::/use_context::/g' "$file"
    sed -i '' 's/use_shared_state(/use_context(/g' "$file"
    
    # Update use_router/use_navigator
    sed -i '' 's/use_router(cx)/navigator()/g' "$file"
    sed -i '' 's/use_navigator(cx)/navigator()/g' "$file"
    sed -i '' 's/let router = use_navigator/let nav = navigator/g' "$file"
    
    # Update render! to rsx!
    sed -i '' 's/render! {/rsx! {/g' "$file"
    
    # Update use_effect
    sed -i '' 's/use_effect(cx, ()/use_effect(/g' "$file"
    sed -i '' 's/use_effect(cx, |/use_effect(|/g' "$file"
    
    # Update to_owned! macro
    sed -i '' 's/to_owned!\[/let /g' "$file"
    
    # Fix router navigation
    sed -i '' 's/router\.push/nav.push/g' "$file"
    sed -i '' 's/router\.navigate_to/nav.push/g' "$file"
done

echo "✅ Script complete! Now manually review and fix:"
echo "  1. Component props that need #[props] attribute"
echo "  2. use_effect closures"
echo "  3. Signal read/write syntax"
echo "  4. Any custom hooks"