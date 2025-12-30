// Test the md() function's comprehensive markdown support
import '../../scripts/kit-sdk';

// Test markdown with all supported elements
const testMarkdown = `# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6

This is **bold** and *italic* and ~~strikethrough~~ text.

Here is \`inline code\` in a sentence.

\`\`\`typescript
const hello = "world";
console.log(hello);
\`\`\`

> This is a blockquote
>> This is nested

---

***

- Unordered item 1
- Unordered item 2

1. Ordered item 1
2. Ordered item 2

[Link text](https://example.com)
![Image alt](https://example.com/image.png)

Line with trailing spaces  
should have a break.
`;

console.error('[MD-TEST] Testing md() function...');

const html = md(testMarkdown);

console.error('[MD-TEST] Generated HTML:');
console.error(html);

// Verify key elements are present
const checks = [
  { name: 'h1', pattern: /<h1>Heading 1<\/h1>/ },
  { name: 'h4', pattern: /<h4>Heading 4<\/h4>/ },
  { name: 'h6', pattern: /<h6>Heading 6<\/h6>/ },
  { name: 'strong', pattern: /<strong>bold<\/strong>/ },
  { name: 'em', pattern: /<em>italic<\/em>/ },
  { name: 'del', pattern: /<del>strikethrough<\/del>/ },
  { name: 'inline code', pattern: /<code>inline code<\/code>/ },
  { name: 'fenced code', pattern: /<pre><code class="typescript">/ },
  { name: 'blockquote', pattern: /<blockquote>This is a blockquote<\/blockquote>/ },
  { name: 'nested blockquote', pattern: /<blockquote><blockquote>This is nested<\/blockquote><\/blockquote>/ },
  { name: 'hr (---)', pattern: /<hr>/ },
  { name: 'ul', pattern: /<ul>.*<li>Unordered item 1<\/li>.*<\/ul>/s },
  { name: 'ol', pattern: /<ol>.*<li>Ordered item 1<\/li>.*<\/ol>/s },
  { name: 'link', pattern: /<a href="https:\/\/example\.com">Link text<\/a>/ },
  { name: 'image', pattern: /<img alt="Image alt" src="https:\/\/example\.com\/image\.png">/ },
  { name: 'line break', pattern: /<br>/ },
];

let passed = 0;
let failed = 0;

for (const check of checks) {
  if (check.pattern.test(html)) {
    console.error(`[MD-TEST] PASS: ${check.name}`);
    passed++;
  } else {
    console.error(`[MD-TEST] FAIL: ${check.name} - pattern not found`);
    failed++;
  }
}

console.error(`[MD-TEST] Results: ${passed} passed, ${failed} failed`);

if (failed === 0) {
  console.error('[MD-TEST] All checks passed!');
} else {
  console.error('[MD-TEST] Some checks failed.');
}

process.exit(0);
