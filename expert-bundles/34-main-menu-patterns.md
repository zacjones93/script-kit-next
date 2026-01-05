# Feature Bundle 34: Main Menu Patterns & UX

## Goal

Ensure the main menu patterns are clear, obvious, and intuitive. Users should immediately understand how items are organized and how to find what they need.

## Current Implementation

### Grouping Strategy
```rust
// Empty filter → frecency-grouped sections
// With filter → flat fuzzy search results

enum GroupedListItem {
    SectionHeader(String),  // "RECENT", "SUGGESTED", "MAIN"
    Item(Arc<Script>),
}
```

### Section Hierarchy
1. **RECENT** - Scripts used today (time-based)
2. **SUGGESTED** - Frecency score > threshold (max 10)
3. **MAIN** - All remaining scripts alphabetically
4. **SCRIPTLETS** - Markdown-based snippets
5. **BUILT-INS** - Clipboard, App Launcher, Window Switcher

### Frecency Algorithm
```rust
// Exponential decay: score = 2^(-days/half_life)
// Default half_life: 7 days
// Min threshold for SUGGESTED: 0.1
score = base_score * 2.0_f64.powf(-days_elapsed / half_life) + 1.0
```

### Visual Layout
- Section headers: 24px height, uppercase, muted color
- Script items: 48px height, icon + name + description
- Selected item: accent background, focus ring
- Keyboard hint: shows shortcut if assigned

## Pain Points & Improvements

### 1. Section Names Are Generic
Current: "RECENT", "SUGGESTED", "MAIN"
- Not immediately clear what "SUGGESTED" means
- "MAIN" is vague

**Proposal**:
- "TODAY" (used today)
- "FREQUENTLY USED" (frecency)
- "ALL SCRIPTS" (everything else)
- Or: Show count like "SCRIPTS (42)"

### 2. No Visual Grouping Cues
All sections look the same except for header text.

**Proposal**:
- Subtle background color per section
- Divider lines between sections
- Collapse/expand sections
- Section icons

### 3. Frecency Not Transparent
Users don't know why something is in "SUGGESTED".

**Proposal**:
- Show "Used 5 times this week" hint on hover
- "Last used: 2 hours ago" metadata
- Option to pin items to top

### 4. No User-Defined Groups
Can't create custom folders/tags.

**Proposal**:
- Allow `// Group: Work` metadata in scripts
- Support folders in `~/.sk/kit/scripts/work/`
- Tag system: `// Tags: git, dev`

### 5. Search UX Issues
- No search history
- No "recent searches"
- No search suggestions

**Proposal**:
- Show recent searches on focus
- "Press ↓ to see recent" hint
- Fuzzy match preview as you type

### 6. Built-ins Mixed with Scripts
Built-in commands appear alongside user scripts.

**Proposal**:
- Separate "TOOLS" section for built-ins
- Or: Only show built-ins when relevant filter
- Option to hide specific built-ins

## Proposed Section Hierarchy

```
┌─────────────────────────────────────────┐
│ 🔍 Search scripts...                    │
├─────────────────────────────────────────┤
│ 📍 PINNED                               │
│   ⭐ My Favorite Script                 │
│   ⭐ Daily Standup                      │
├─────────────────────────────────────────┤
│ ⏱️ TODAY                                │
│   git-commit.ts     (2 hours ago)       │
│   quick-note.ts     (4 hours ago)       │
├─────────────────────────────────────────┤
│ 🔥 FREQUENTLY USED                      │
│   clipboard-manager.ts  (12 uses)       │
│   api-test.ts           (8 uses)        │
├─────────────────────────────────────────┤
│ 📁 WORK (folder)                        │
│   jira-ticket.ts                        │
│   slack-status.ts                       │
├─────────────────────────────────────────┤
│ 📜 ALL SCRIPTS (47)                     │
│   A-script.ts                           │
│   another-script.ts                     │
│   ...                                   │
├─────────────────────────────────────────┤
│ 🧰 TOOLS                                │
│   Clipboard History        ⌘⇧V         │
│   App Launcher             ⌘⇧A         │
│   Window Switcher          ⌘⇧W         │
└─────────────────────────────────────────┘
```

## Key Questions

1. **Section Collapsibility**: Should sections be collapsible? Remember state?

2. **Pinning**: Should users be able to pin scripts to top? Stored where?

3. **Folders vs Tags**: Should we support file system folders, metadata tags, or both?

4. **Frecency Visibility**: Show usage stats on hover, or keep it invisible?

5. **Built-in Placement**: Separate section, inline with scripts, or hidden by default?

6. **Empty State**: What to show when no scripts match filter?

## Implementation Checklist

- [ ] Rename sections to be more descriptive
- [ ] Add section icons
- [ ] Show usage metadata on hover
- [ ] Implement pinning system
- [ ] Support folder-based grouping
- [ ] Add tag support in script metadata
- [ ] Create separate TOOLS section
- [ ] Add search history/suggestions
- [ ] Improve empty state UX
- [ ] Make sections collapsible (optional)

