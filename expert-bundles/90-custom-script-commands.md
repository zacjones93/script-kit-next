# Expert Bundle #90: Custom Script Commands

## Overview

Custom script commands extend Script Kit's functionality through user-created scripts. This bundle covers patterns for building scripts that integrate seamlessly with the app, including custom prompts, background processes, script composition, and advanced SDK usage.

## Architecture

### Script Types

```rust
// src/scripts/types.rs

/// Categories of scripts based on behavior
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptType {
    /// Interactive scripts with UI prompts
    Interactive,
    /// Background scripts that run silently
    Background,
    /// System integration scripts (clipboard, notifications)
    System,
    /// Automation scripts triggered by events
    Automation,
    /// Template scripts for generating content
    Template,
    /// Tool scripts for developer utilities
    Tool,
}

/// Script execution modes
#[derive(Clone, Copy, Debug)]
pub enum ExecutionMode {
    /// Run once and exit
    OneShot,
    /// Keep running in background
    Persistent,
    /// Run on schedule
    Scheduled { cron: String },
    /// Run on file system changes
    Watched { paths: Vec<PathBuf> },
    /// Run on clipboard changes
    ClipboardTrigger,
}
```

### Script Metadata

```typescript
// SDK type definitions for script metadata
export interface ScriptMetadata {
  // Display
  name: string;
  description?: string;
  icon?: string;
  
  // Triggers
  shortcut?: string;
  trigger?: string;        // Custom trigger word
  schedule?: string;       // Cron expression
  watch?: string[];        // File paths to watch
  
  // Behavior
  background?: boolean;
  cache?: boolean;         // Cache results
  timeout?: number;        // Max execution time
  
  // Categories
  category?: string;
  tags?: string[];
  
  // Author
  author?: string;
  version?: string;
  repository?: string;
}

// Example usage in script
export const metadata: ScriptMetadata = {
  name: "Quick Calculator",
  description: "Evaluate math expressions",
  shortcut: "cmd+shift+=",
  icon: "calculator",
  tags: ["math", "utility"]
};
```

## Common Patterns

### Interactive Prompts

```typescript
// scripts/examples/interactive-prompt.ts
import '@scriptkit/sdk';

export const metadata = {
  name: "File Finder",
  description: "Search and open files",
  shortcut: "cmd+shift+f"
};

// Multi-step prompt flow
const directory = await path({
  hint: "Select directory to search",
  startPath: home()
});

const pattern = await arg({
  placeholder: "Search pattern...",
  hint: `Searching in ${directory}`
});

const files = await glob(`${directory}/**/*${pattern}*`);

if (files.length === 0) {
  await notify("No files found");
  exit(0);
}

const selected = await arg({
  placeholder: "Select file to open",
  choices: files.map(f => ({
    name: basename(f),
    description: dirname(f),
    value: f
  }))
});

await open(selected);
```

### Background Process

```typescript
// scripts/examples/background-watcher.ts
import '@scriptkit/sdk';

export const metadata = {
  name: "Download Organizer",
  description: "Auto-organize downloads folder",
  background: true,
  watch: ["~/Downloads"]
};

// File organization rules
const rules: Record<string, string> = {
  '.pdf': 'Documents',
  '.doc': 'Documents',
  '.docx': 'Documents',
  '.jpg': 'Images',
  '.png': 'Images',
  '.mp4': 'Videos',
  '.zip': 'Archives',
  '.dmg': 'Installers'
};

// Watch for new files
onFileChange(async (event) => {
  if (event.type !== 'add') return;
  
  const ext = extname(event.path).toLowerCase();
  const targetFolder = rules[ext];
  
  if (targetFolder) {
    const destDir = home(`Downloads/${targetFolder}`);
    await ensureDir(destDir);
    
    const destPath = join(destDir, basename(event.path));
    await move(event.path, destPath);
    
    await notify({
      title: "File Organized",
      body: `Moved to ${targetFolder}`
    });
  }
});

// Keep running
await new Promise(() => {});
```

### Custom Prompt Widget

```typescript
// scripts/examples/custom-widget.ts
import '@scriptkit/sdk';

export const metadata = {
  name: "Color Picker",
  description: "Pick and copy colors"
};

// Custom HTML prompt
const color = await div(`
  <div class="p-6 flex flex-col items-center gap-4">
    <div class="text-lg font-semibold text-white">Pick a Color</div>
    
    <input 
      type="color" 
      id="colorPicker"
      value="#f59e0b"
      class="w-32 h-32 rounded-lg cursor-pointer"
    />
    
    <div id="colorValue" class="text-2xl font-mono text-amber-400">#f59e0b</div>
    
    <div class="flex gap-2">
      <button 
        onclick="submit(document.getElementById('colorPicker').value)"
        class="px-4 py-2 bg-amber-500 text-black rounded-md font-medium"
      >
        Copy HEX
      </button>
      <button 
        onclick="submit(hexToRgb(document.getElementById('colorPicker').value))"
        class="px-4 py-2 bg-zinc-700 text-white rounded-md"
      >
        Copy RGB
      </button>
    </div>
  </div>
  
  <script>
    const picker = document.getElementById('colorPicker');
    const display = document.getElementById('colorValue');
    
    picker.addEventListener('input', (e) => {
      display.textContent = e.target.value;
    });
    
    function hexToRgb(hex) {
      const r = parseInt(hex.slice(1, 3), 16);
      const g = parseInt(hex.slice(3, 5), 16);
      const b = parseInt(hex.slice(5, 7), 16);
      return \`rgb(\${r}, \${g}, \${b})\`;
    }
  </script>
`);

await clipboard.writeText(color);
await notify(`Copied: ${color}`);
```

### Script Composition

```typescript
// scripts/examples/composed-script.ts
import '@scriptkit/sdk';

export const metadata = {
  name: "Project Setup",
  description: "Initialize a new project"
};

// Import other scripts as functions
const { createGitRepo } = await import('./helpers/git');
const { setupPackageJson } = await import('./helpers/npm');
const { generateReadme } = await import('./helpers/docs');

// Step 1: Get project details
const projectName = await arg("Project name");
const projectType = await arg({
  placeholder: "Project type",
  choices: ["node", "python", "rust", "go"]
});

const features = await checklist({
  message: "Select features",
  choices: [
    { name: "Git repository", value: "git", checked: true },
    { name: "README.md", value: "readme", checked: true },
    { name: "License", value: "license" },
    { name: "CI/CD", value: "ci" },
    { name: "Tests", value: "tests" }
  ]
});

// Step 2: Create project
const projectDir = home(`Projects/${projectName}`);
await ensureDir(projectDir);

// Step 3: Run setup functions based on selections
await setHint("Setting up project...");

if (features.includes("git")) {
  await createGitRepo(projectDir);
}

if (projectType === "node") {
  await setupPackageJson(projectDir, projectName);
}

if (features.includes("readme")) {
  await generateReadme(projectDir, {
    name: projectName,
    type: projectType
  });
}

// Step 4: Open in editor
const openNow = await confirm("Open in VS Code?");
if (openNow) {
  await exec(`code ${projectDir}`);
}

await notify(`Project "${projectName}" created!`);
```

### API Integration

```typescript
// scripts/examples/api-integration.ts
import '@scriptkit/sdk';

export const metadata = {
  name: "GitHub Quick Actions",
  description: "Common GitHub operations",
  shortcut: "cmd+shift+g"
};

// Get GitHub token from env or prompt
let token = env.GITHUB_TOKEN;
if (!token) {
  token = await arg({
    placeholder: "Enter GitHub token",
    secret: true
  });
  await env.set("GITHUB_TOKEN", token);
}

const gh = async (endpoint: string, options?: RequestInit) => {
  const response = await fetch(`https://api.github.com${endpoint}`, {
    ...options,
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/vnd.github.v3+json",
      ...options?.headers
    }
  });
  return response.json();
};

// Action selection
const action = await arg({
  placeholder: "Select action",
  choices: [
    { name: "List my repos", value: "repos" },
    { name: "Create repo", value: "create" },
    { name: "Search repos", value: "search" },
    { name: "View notifications", value: "notifications" }
  ]
});

switch (action) {
  case "repos": {
    const repos = await gh("/user/repos?sort=updated&per_page=20");
    const selected = await arg({
      placeholder: "Select repo",
      choices: repos.map((r: any) => ({
        name: r.name,
        description: r.description || "No description",
        value: r.html_url
      }))
    });
    await open(selected);
    break;
  }
  
  case "create": {
    const name = await arg("Repository name");
    const isPrivate = await confirm("Private repository?");
    
    await gh("/user/repos", {
      method: "POST",
      body: JSON.stringify({
        name,
        private: isPrivate,
        auto_init: true
      })
    });
    
    await notify(`Created ${name}!`);
    break;
  }
  
  case "search": {
    const query = await arg("Search query");
    const results = await gh(`/search/repositories?q=${encodeURIComponent(query)}`);
    
    const selected = await arg({
      placeholder: "Select repo",
      choices: results.items.slice(0, 20).map((r: any) => ({
        name: r.full_name,
        description: `⭐ ${r.stargazers_count} - ${r.description || ""}`,
        value: r.html_url
      }))
    });
    
    await open(selected);
    break;
  }
  
  case "notifications": {
    const notifications = await gh("/notifications");
    
    for (const n of notifications.slice(0, 10)) {
      console.log(`• ${n.subject.title}`);
    }
    
    if (notifications.length === 0) {
      await notify("No notifications");
    }
    break;
  }
}
```

### Clipboard Automation

```typescript
// scripts/examples/clipboard-transform.ts
import '@scriptkit/sdk';

export const metadata = {
  name: "Clipboard Transformer",
  description: "Transform clipboard content"
};

const content = await clipboard.readText();

if (!content) {
  await notify("Clipboard is empty");
  exit(0);
}

const transform = await arg({
  placeholder: "Select transformation",
  choices: [
    { name: "UPPERCASE", value: "upper" },
    { name: "lowercase", value: "lower" },
    { name: "Title Case", value: "title" },
    { name: "camelCase", value: "camel" },
    { name: "snake_case", value: "snake" },
    { name: "kebab-case", value: "kebab" },
    { name: "Reverse", value: "reverse" },
    { name: "Base64 Encode", value: "base64" },
    { name: "Base64 Decode", value: "base64decode" },
    { name: "URL Encode", value: "urlencode" },
    { name: "URL Decode", value: "urldecode" },
    { name: "JSON Pretty", value: "jsonpretty" },
    { name: "JSON Minify", value: "jsonminify" }
  ]
});

let result: string;

switch (transform) {
  case "upper":
    result = content.toUpperCase();
    break;
  case "lower":
    result = content.toLowerCase();
    break;
  case "title":
    result = content.replace(/\w\S*/g, (txt) => 
      txt.charAt(0).toUpperCase() + txt.substr(1).toLowerCase()
    );
    break;
  case "camel":
    result = content.toLowerCase()
      .replace(/[^a-zA-Z0-9]+(.)/g, (_, chr) => chr.toUpperCase());
    break;
  case "snake":
    result = content
      .replace(/\s+/g, '_')
      .replace(/([a-z])([A-Z])/g, '$1_$2')
      .toLowerCase();
    break;
  case "kebab":
    result = content
      .replace(/\s+/g, '-')
      .replace(/([a-z])([A-Z])/g, '$1-$2')
      .toLowerCase();
    break;
  case "reverse":
    result = content.split('').reverse().join('');
    break;
  case "base64":
    result = Buffer.from(content).toString('base64');
    break;
  case "base64decode":
    result = Buffer.from(content, 'base64').toString('utf8');
    break;
  case "urlencode":
    result = encodeURIComponent(content);
    break;
  case "urldecode":
    result = decodeURIComponent(content);
    break;
  case "jsonpretty":
    result = JSON.stringify(JSON.parse(content), null, 2);
    break;
  case "jsonminify":
    result = JSON.stringify(JSON.parse(content));
    break;
  default:
    result = content;
}

await clipboard.writeText(result);
await notify("Transformed and copied!");
```

## Testing Scripts

```typescript
// tests/smoke/test-custom-commands.ts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Test 1: Script list with custom icons
await div(`
  <div class="flex flex-col">
    <div class="h-[52px] px-4 flex items-center gap-3 bg-zinc-800">
      <div class="w-6 h-6 rounded-md bg-amber-500/20 flex items-center justify-center">
        <span class="text-xs">🧮</span>
      </div>
      <div class="flex-1">
        <div class="text-sm text-white">Quick Calculator</div>
        <div class="text-xs text-zinc-500">Evaluate math expressions</div>
      </div>
      <span class="text-xs text-zinc-500 font-mono">⌘⇧=</span>
    </div>
    <div class="h-[52px] px-4 flex items-center gap-3 hover:bg-zinc-800/50">
      <div class="w-6 h-6 rounded-md bg-blue-500/20 flex items-center justify-center">
        <span class="text-xs">🔍</span>
      </div>
      <div class="flex-1">
        <div class="text-sm text-white">File Finder</div>
        <div class="text-xs text-zinc-500">Search and open files</div>
      </div>
      <span class="text-xs text-zinc-500 font-mono">⌘⇧F</span>
    </div>
    <div class="h-[52px] px-4 flex items-center gap-3 hover:bg-zinc-800/50">
      <div class="w-6 h-6 rounded-md bg-green-500/20 flex items-center justify-center">
        <span class="text-xs">🔄</span>
      </div>
      <div class="flex-1">
        <div class="text-sm text-white">Clipboard Transformer</div>
        <div class="text-xs text-zinc-500">Transform clipboard content</div>
      </div>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot1 = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, 'custom-scripts-list.png'), Buffer.from(shot1.data, 'base64'));

// Test 2: Custom widget prompt
await div(`
  <div class="p-6 flex flex-col items-center gap-4 bg-zinc-900 rounded-lg">
    <div class="text-lg font-semibold text-white">Pick a Color</div>
    
    <div class="w-32 h-32 rounded-lg bg-amber-500"></div>
    
    <div class="text-2xl font-mono text-amber-400">#f59e0b</div>
    
    <div class="flex gap-2">
      <button class="px-4 py-2 bg-amber-500 text-black rounded-md font-medium">
        Copy HEX
      </button>
      <button class="px-4 py-2 bg-zinc-700 text-white rounded-md">
        Copy RGB
      </button>
    </div>
  </div>
`);

await new Promise(r => setTimeout(r, 500));
const shot2 = await captureScreenshot();
writeFileSync(join(dir, 'custom-widget.png'), Buffer.from(shot2.data, 'base64'));

console.error('[CUSTOM COMMANDS] Test screenshots saved');
process.exit(0);
```

## Best Practices

```typescript
// Best practices for custom scripts

// 1. Always provide metadata
export const metadata = {
  name: "My Script",           // Required: display name
  description: "What it does", // Recommended: helps users
  shortcut: "cmd+shift+m",     // Optional: quick access
  icon: "zap",                 // Optional: visual identity
};

// 2. Handle errors gracefully
try {
  const result = await riskyOperation();
} catch (error) {
  await notify({
    title: "Error",
    body: error.message
  });
  exit(1);
}

// 3. Provide progress feedback
await setHint("Processing...");
// ... long operation
await setHint("Almost done...");

// 4. Validate user input
const email = await arg({
  placeholder: "Email address",
  validate: (value) => {
    if (!value.includes("@")) {
      return "Please enter a valid email";
    }
    return true;
  }
});

// 5. Use environment variables for secrets
const apiKey = env.MY_API_KEY || await arg({
  placeholder: "API Key",
  secret: true
});

// 6. Clean up resources
process.on("exit", () => {
  // Clean up temp files, close connections
});

// 7. Document your scripts
/**
 * @name Project Generator
 * @description Creates a new project with boilerplate
 * @param {string} name - Project name
 * @param {string} type - Project type (node, python, rust)
 */
```

## Related Bundles

- Bundle #67: SDK Preload Architecture - SDK internals
- Bundle #61: Script Execution Lifecycle - How scripts run
- Bundle #66: Scriptlet System - Template scripts
- Bundle #72: Actions System - Script actions
