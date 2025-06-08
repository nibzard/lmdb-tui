#!/usr/bin/env python3
"""
AI-ready TUI snapshot analyzer for lmdb-tui testing.

This script processes captured TUI snapshots and generates comprehensive
analysis reports suitable for AI review and regression detection.
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Any
from datetime import datetime
import argparse


def load_snapshots(snapshots_dir: Path) -> List[Dict[str, Any]]:
    """Load all JSON snapshots from the snapshots directory."""
    snapshots = []
    
    for json_file in snapshots_dir.glob("*.json"):
        if json_file.name.endswith("_report.json"):
            continue  # Skip report files
            
        try:
            with open(json_file, 'r') as f:
                snapshot = json.load(f)
                snapshots.append(snapshot)
        except Exception as e:
            print(f"Warning: Could not load {json_file}: {e}")
    
    return sorted(snapshots, key=lambda x: x['timestamp'])


def analyze_ui_consistency(snapshots: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Analyze UI consistency across snapshots."""
    analysis = {
        "total_snapshots": len(snapshots),
        "unique_dimensions": set(),
        "views_tested": set(),
        "state_transitions": [],
        "potential_issues": []
    }
    
    prev_snapshot = None
    for snapshot in snapshots:
        # Track dimensions
        dims = tuple(snapshot['dimensions'])
        analysis["unique_dimensions"].add(dims)
        
        # Track views
        view = snapshot['app_state']['current_view']
        analysis["views_tested"].add(view)
        
        # Track state transitions
        if prev_snapshot:
            prev_view = prev_snapshot['app_state']['current_view']
            if prev_view != view:
                analysis["state_transitions"].append({
                    "from": prev_view,
                    "to": view,
                    "test": snapshot['test_name'],
                    "description": snapshot['metadata'].get('description', 'unknown')
                })
        
        # Check for potential issues
        content = snapshot['content']
        if '???' in content or '□' in content:
            analysis["potential_issues"].append({
                "type": "missing_characters",
                "snapshot": snapshot['snapshot_id'],
                "description": "Potential missing or unsupported characters"
            })
        
        if len(content.split('\n')) != snapshot['dimensions'][1]:
            analysis["potential_issues"].append({
                "type": "dimension_mismatch",
                "snapshot": snapshot['snapshot_id'],
                "description": "Content height doesn't match terminal height"
            })
            
        prev_snapshot = snapshot
    
    # Convert sets to lists for JSON serialization
    analysis["unique_dimensions"] = list(analysis["unique_dimensions"])
    analysis["views_tested"] = list(analysis["views_tested"])
    
    return analysis


def analyze_performance_patterns(snapshots: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Analyze performance and response patterns."""
    analysis = {
        "render_complexity": {},
        "content_length_stats": {},
        "query_response_patterns": []
    }
    
    content_lengths = []
    query_snapshots = []
    
    for snapshot in snapshots:
        content_len = len(snapshot['content'])
        content_lengths.append(content_len)
        
        # Analyze query patterns
        app_state = snapshot['app_state']
        if app_state['current_view'] == 'Query':
            query_snapshots.append({
                "query": app_state['query'],
                "result_count": app_state['entry_count'],
                "content_length": content_len
            })
    
    if content_lengths:
        analysis["content_length_stats"] = {
            "min": min(content_lengths),
            "max": max(content_lengths),
            "avg": sum(content_lengths) / len(content_lengths)
        }
    
    analysis["query_response_patterns"] = query_snapshots
    
    return analysis


def generate_ai_analysis_prompt(analysis_results: Dict[str, Any]) -> str:
    """Generate a structured prompt for AI analysis."""
    
    prompt = f"""
# TUI Testing Analysis Report

## Test Session Summary
- **Total Snapshots**: {analysis_results['ui_consistency']['total_snapshots']}
- **Views Tested**: {', '.join(analysis_results['ui_consistency']['views_tested'])}
- **Terminal Sizes**: {analysis_results['ui_consistency']['unique_dimensions']}

## State Transitions
{json.dumps(analysis_results['ui_consistency']['state_transitions'], indent=2)}

## Potential Issues Found
{json.dumps(analysis_results['ui_consistency']['potential_issues'], indent=2)}

## Performance Patterns
{json.dumps(analysis_results['performance'], indent=2)}

## Analysis Instructions for AI

Please analyze this TUI testing data and provide feedback on:

1. **UI Consistency**: Are there any inconsistencies in layout, spacing, or visual hierarchy?
2. **User Experience**: Do the state transitions make sense? Are there confusing flows?
3. **Visual Design**: Is the information density appropriate? Are there readability issues?
4. **Responsive Design**: How well does the UI adapt to different terminal sizes?
5. **Error Handling**: Are error states and edge cases handled gracefully?
6. **Performance**: Are there any patterns suggesting performance issues?

## Recommendations

Based on your analysis, please provide:
- Specific UI/UX improvements
- Potential bug reports
- Accessibility considerations
- Code quality suggestions related to the UI layer

## Test Data Context

This data comes from automated testing of lmdb-tui, a terminal-based database browser.
Key features being tested:
- Database navigation
- Query interface
- Help system
- Responsive layout

Focus on practical improvements that would enhance the user experience.
"""
    
    return prompt


def main():
    parser = argparse.ArgumentParser(description="Analyze TUI test snapshots")
    parser.add_argument("--snapshots-dir", "-d", type=Path, default="test_snapshots",
                       help="Directory containing snapshot files")
    parser.add_argument("--output", "-o", type=Path, default="tui_analysis_report.json",
                       help="Output file for analysis report")
    parser.add_argument("--ai-prompt", "-p", type=Path, default="ai_analysis_prompt.md",
                       help="Output file for AI analysis prompt")
    
    args = parser.parse_args()
    
    if not args.snapshots_dir.exists():
        print(f"Error: Snapshots directory {args.snapshots_dir} not found")
        sys.exit(1)
    
    print(f"🔍 Analyzing snapshots in {args.snapshots_dir}")
    
    # Load snapshots
    snapshots = load_snapshots(args.snapshots_dir)
    if not snapshots:
        print("No snapshots found to analyze")
        sys.exit(1)
    
    print(f"📊 Found {len(snapshots)} snapshots to analyze")
    
    # Perform analysis
    analysis_results = {
        "generated_at": datetime.now().isoformat(),
        "snapshots_analyzed": len(snapshots),
        "ui_consistency": analyze_ui_consistency(snapshots),
        "performance": analyze_performance_patterns(snapshots),
        "raw_snapshots": snapshots  # Include for detailed AI analysis
    }
    
    # Save analysis report
    with open(args.output, 'w') as f:
        json.dump(analysis_results, f, indent=2, default=str)
    print(f"💾 Analysis report saved to {args.output}")
    
    # Generate AI prompt
    ai_prompt = generate_ai_analysis_prompt(analysis_results)
    with open(args.ai_prompt, 'w') as f:
        f.write(ai_prompt)
    print(f"🤖 AI analysis prompt saved to {args.ai_prompt}")
    
    # Print summary
    print("\n📋 Analysis Summary:")
    print(f"  - Views tested: {', '.join(analysis_results['ui_consistency']['views_tested'])}")
    print(f"  - State transitions: {len(analysis_results['ui_consistency']['state_transitions'])}")
    print(f"  - Potential issues: {len(analysis_results['ui_consistency']['potential_issues'])}")
    print(f"  - Terminal sizes: {len(analysis_results['ui_consistency']['unique_dimensions'])}")
    
    if analysis_results['ui_consistency']['potential_issues']:
        print("\n⚠️  Issues detected:")
        for issue in analysis_results['ui_consistency']['potential_issues']:
            print(f"  - {issue['type']}: {issue['description']}")


if __name__ == "__main__":
    main()